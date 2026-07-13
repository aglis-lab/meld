use std::borrow::Cow;
use std::collections::HashMap;

const DEFAULT_STACK_CAPACITY: usize = 16;
const DEFAULT_SCOPE_MARKS_CAPACITY: usize = 16;
const DEFAULT_ITERATE_INDICES_CAPACITY: usize = 16;

use super::stack::*;
use crate::{
    opcode,
    runtime::Program,
    value::{Number, Value},
};
use anyhow::anyhow;

pub struct EvaluatorConfig {
    pub ignore_missing_variables: bool,
}

impl EvaluatorConfig {
    pub fn new() -> Self {
        Self {
            ignore_missing_variables: true,
        }
    }
}

pub trait Callable {
    fn call(&self, args: &Vec<Value>) -> Value;
}

impl<F> Callable for F
where
    F: Fn(&Vec<Value>) -> Value,
{
    fn call(&self, args: &Vec<Value>) -> Value {
        self(args)
    }
}

pub struct Runtime<'a> {
    program: &'a Program,
    config: EvaluatorConfig,
    output: String,

    // Used for scope management
    scope_stack: Stack<'a>,
    scope_marks: HashMap<usize, usize>,
    iterate_indices: HashMap<usize, usize>,

    // Used by expression
    evaluation_stack: Stack<'a>,

    // Context for helper functions
    context: HashMap<String, Box<dyn Callable>>,
}

impl<'a> Runtime<'a> {
    #[inline(always)]
    pub fn new(program: &'a Program, config: EvaluatorConfig) -> Self {
        Self {
            program,
            config,
            output: String::with_capacity(program.header().content_length() as usize),
            scope_stack: Stack::with_capacity(DEFAULT_STACK_CAPACITY),
            evaluation_stack: Stack::with_capacity(DEFAULT_STACK_CAPACITY),
            scope_marks: HashMap::with_capacity(DEFAULT_SCOPE_MARKS_CAPACITY),
            iterate_indices: HashMap::with_capacity(DEFAULT_ITERATE_INDICES_CAPACITY),
            context: HashMap::new(), // No need to preallocate context, as it will be run before the evaluation or hot path
        }
    }

    pub fn register_callable<F>(&mut self, name: &str, callable: F)
    where
        F: Callable + 'static,
    {
        self.context.insert(name.to_string(), Box::new(callable));
    }

    pub fn register_callable_fn<F>(&mut self, name: &str, callable: F)
    where
        F: Fn(&Vec<Value>) -> Value + 'static,
    {
        self.context.insert(name.to_string(), Box::new(callable));
    }

    pub fn run(&mut self, payload: &'a Value) -> anyhow::Result<()> {
        self.clear();

        if payload.is_object() {
            self.scope_stack.push(Cow::Borrowed(payload));
        }

        let mut pc: usize = 0;
        loop {
            let opcode = self.program.get_op(pc)?;
            let step = match opcode {
                opcode::TEXT => self.text(pc)?,
                opcode::OUT => self.out()?,
                opcode::LOOKUP => self.lookup(pc)?,
                opcode::LOOKUP_OUT => self.lookup_out(pc)?,
                opcode::CALL => self.call(pc)?,
                opcode::PUSH_CONST => self.push_const(pc)?,
                opcode::EQ => self.compare(pc, |left, right| left == right)?,
                opcode::NEQ => self.compare(pc, |left, right| left != right)?,
                opcode::GT => self.compare_ordered(pc, |ordering| ordering.is_gt())?,
                opcode::GTE => self.compare_ordered(pc, |ordering| ordering.is_ge())?,
                opcode::LT => self.compare_ordered(pc, |ordering| ordering.is_lt())?,
                opcode::LTE => self.compare_ordered(pc, |ordering| ordering.is_le())?,
                opcode::NOT => self.not(pc)?,
                opcode::AND => self.logic(pc, |left, right| left && right)?,
                opcode::OR => self.logic(pc, |left, right| left || right)?,
                opcode::EMPTY => self.empty(pc, true)?,
                opcode::NOT_EMPTY => self.empty(pc, false)?,
                opcode::LENGTH => self.length(pc)?,
                opcode::CONCAT => self.concat(pc)?,
                opcode::CONDITION => {
                    pc = self.condition(pc)?;
                    continue;
                }
                opcode::JUMP => {
                    pc = self.jump(pc)?;
                    continue;
                }
                opcode::POP_SCOPE => {
                    self.scope_stack.pop();
                    1
                }
                opcode::ITERATE => {
                    pc = self.iterate(pc)?;
                    continue;
                }
                opcode::END => break,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unknown instruction: {}, pc: {}",
                        opcode,
                        pc
                    ));
                }
            };

            pc += step;
        }

        Ok(())
    }

    // output
    #[inline(always)]
    pub fn output(&self) -> &str {
        &self.output
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.output.clear();
        self.scope_stack.scopes.clear();
        self.evaluation_stack.scopes.clear();
        self.scope_marks.clear();
        self.iterate_indices.clear();
    }

    // 9 bytes
    // 1 byte for opcode, 4 bytes for start, 4 bytes for end
    #[inline(always)]
    fn text(&mut self, pc: usize) -> anyhow::Result<usize> {
        let range = self.program.get_op_range(pc + 1)?;
        let content = self.program.get_content(range)?;
        let text = std::str::from_utf8(content)?;

        self.output.push_str(text);

        Ok(9)
    }

    // 1 byte
    // 1 byte for opcode without any additional data
    // pop the top value from the evaluation stack and append it to the output
    #[inline(always)]
    fn out(&mut self) -> anyhow::Result<usize> {
        let val = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;

        Self::append_value(&mut self.output, &val);

        Ok(1)
    }

    // 9 bytes
    // 1 byte for opcode, 4 bytes for start, 4 bytes for end
    #[inline(always)]
    fn lookup(&mut self, pc: usize) -> anyhow::Result<usize> {
        let range = self.program.get_op_range(pc + 1)?;
        let content = self.program.get_content(range)?;
        let lookup_key = std::str::from_utf8(content)?;
        let scope_value = self.scope_stack.get(lookup_key);
        if let Some(val) = scope_value {
            self.evaluation_stack.push(Cow::Owned(val.clone()));
        } else if !self.config.ignore_missing_variables {
            return Err(anyhow!("can't lookup variable {}", lookup_key));
        } else {
            self.evaluation_stack.push(Cow::Owned(Value::Null));
        }

        Ok(9)
    }

    // 9 bytes
    // 1 byte for opcode, 4 bytes for start, 4 bytes for end
    #[inline(always)]
    fn lookup_out(&mut self, pc: usize) -> anyhow::Result<usize> {
        let range = self.program.get_op_range(pc + 1)?;
        let content = self.program.get_content(range)?;
        let lookup_key = std::str::from_utf8(content)?;
        if let Some(val) = self.scope_stack.get(lookup_key) {
            Self::append_value(&mut self.output, val);
        } else if !self.config.ignore_missing_variables {
            return Err(anyhow!("can't lookup variable {}", lookup_key));
        }

        Ok(9)
    }

    #[inline(always)]
    fn push_const(&mut self, pc: usize) -> anyhow::Result<usize> {
        let literal_type = self.program.get_op(pc + 1)?;
        let range = self.program.get_op_range(pc + 2)?;
        let content = self.program.get_content(range)?;
        let value = match literal_type {
            opcode::LITERAL_STRING => Value::String(std::str::from_utf8(content)?.to_string()),
            opcode::LITERAL_FLOAT => {
                let parsed = std::str::from_utf8(content)?.parse::<f64>()?;
                Value::Number(
                    Number::from_f64(parsed).ok_or_else(|| anyhow!("invalid float literal"))?,
                )
            }
            opcode::LITERAL_INT => {
                let parsed = std::str::from_utf8(content)?.parse::<i64>()?;
                Value::Number(Number::from(parsed))
            }
            opcode::LITERAL_BOOL => Value::Bool(std::str::from_utf8(content)? == "true"),
            opcode::LITERAL_NULL => Value::Null,
            _ => return Err(anyhow!("Unknown literal type: {}", literal_type)),
        };

        self.evaluation_stack.push(Cow::Owned(value));
        Ok(10)
    }

    fn call(&mut self, pc: usize) -> anyhow::Result<usize> {
        let helper_range = self.program.get_op_range(pc + 1)?;
        let helper_bytes = self.program.get_content(helper_range)?;
        let helper_name = std::str::from_utf8(helper_bytes)?;
        let arg_count = self.program.get_op(pc + 9)? as usize;

        let length = self.evaluation_stack.len();
        let args = self
            .evaluation_stack
            .get_drain_range(length - arg_count..length)
            .ok_or_else(|| anyhow!("Not enough arguments on evaluation stack"))?;

        let result = match helper_name {
            "length" => {
                let arg = args
                    .first()
                    .ok_or_else(|| anyhow!("length expects 1 argument"))?;
                let len = match arg.as_ref() {
                    Value::String(val) => val.len() as i64,
                    Value::Array(val) => val.len() as i64,
                    Value::Object(val) => val.len() as i64,
                    _ => 0,
                };
                Value::Number(Number::from(len))
            }
            "empty" => {
                let arg = args
                    .first()
                    .ok_or_else(|| anyhow!("empty expects 1 argument"))?;
                Value::Bool(Self::is_empty(arg.as_ref()))
            }
            "not_empty" => {
                let arg = args
                    .first()
                    .ok_or_else(|| anyhow!("not_empty expects 1 argument"))?;
                Value::Bool(!Self::is_empty(arg.as_ref()))
            }
            "concat" => {
                let mut out = String::new();
                for arg in args {
                    Self::append_value(&mut out, arg.as_ref());
                }
                Value::String(out)
            }
            "coalesce" => {
                let mut selected = Value::Null;
                for arg in args {
                    if !arg.as_ref().is_null() {
                        selected = arg.as_ref().clone();
                        break;
                    }
                }
                selected
            }
            _ => self
                .context
                .get(helper_name)
                .ok_or_else(|| anyhow!("Unknown helper: {}", helper_name))?
                .call(&args.iter().map(|v| v.as_ref().clone()).collect()),
        };

        self.evaluation_stack.push(Cow::Owned(result));
        Ok(10)
    }

    #[inline(always)]
    fn iterate(&mut self, pc: usize) -> anyhow::Result<usize> {
        let item_name_range = self.program.get_op_range(pc + 1)?;
        let index_name_range = self.program.get_op_range(pc + 9)?;
        let done_target = self.program.get_op_u32(pc + 17)? as usize;

        let item_name = std::str::from_utf8(self.program.get_content(item_name_range)?)?;
        let index_name = std::str::from_utf8(self.program.get_content(index_name_range)?)?;

        let base_depth = *self.scope_marks.entry(pc).or_insert(self.scope_stack.len());
        self.cleanup_scope_to_depth(base_depth);

        let collection = self
            .evaluation_stack
            .peek()
            .ok_or_else(|| anyhow!("ITERATE expects a collection on evaluation stack"))?;

        let arr = collection
            .as_ref()
            .as_array()
            .ok_or_else(|| anyhow!("ITERATE requires array collection"))?;

        let next_index = self.iterate_indices.get(&pc).copied().unwrap_or(0);
        if next_index >= arr.len() {
            self.iterate_indices.remove(&pc);
            self.scope_marks.remove(&pc);
            self.cleanup_scope_to_depth(base_depth);
            // Pop the collection from the evaluation stack
            let _ = self.evaluation_stack.pop();
            return Ok(done_target);
        }

        let mut scope = serde_json::Map::new();
        scope.insert(item_name.to_string(), arr[next_index].clone());
        scope.insert(
            index_name.to_string(),
            Value::Number(Number::from(next_index as i64)),
        );
        self.scope_stack.push(Cow::Owned(Value::Object(scope)));

        self.iterate_indices.insert(pc, next_index + 1);
        Ok(pc + 21)
    }

    #[inline(always)]
    fn condition(&mut self, pc: usize) -> anyhow::Result<usize> {
        let false_target = self.program.get_op_u32(pc + 1)? as usize;
        let condition = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;

        if Self::is_truthy(condition.as_ref()) {
            Ok(pc + 5)
        } else {
            Ok(false_target)
        }
    }

    #[inline(always)]
    fn jump(&self, pc: usize) -> anyhow::Result<usize> {
        Ok(self.program.get_op_u32(pc + 1)? as usize)
    }

    #[inline(always)]
    fn compare<F>(&mut self, _pc: usize, predicate: F) -> anyhow::Result<usize>
    where
        F: FnOnce(&Value, &Value) -> bool,
    {
        let right = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;
        let left = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;

        self.evaluation_stack.push(Cow::Owned(Value::Bool(predicate(
            left.as_ref(),
            right.as_ref(),
        ))));

        Ok(1)
    }

    #[inline(always)]
    fn compare_ordered<F>(&mut self, _pc: usize, predicate: F) -> anyhow::Result<usize>
    where
        F: FnOnce(std::cmp::Ordering) -> bool,
    {
        let right = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;
        let left = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;

        let ordering = Self::compare_values(left.as_ref(), right.as_ref())
            .ok_or_else(|| anyhow!("Values are not comparable"))?;

        self.evaluation_stack
            .push(Cow::Owned(Value::Bool(predicate(ordering))));

        Ok(1)
    }

    #[inline(always)]
    fn not(&mut self, _pc: usize) -> anyhow::Result<usize> {
        let value = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;

        self.evaluation_stack
            .push(Cow::Owned(Value::Bool(!Self::is_truthy(value.as_ref()))));

        Ok(1)
    }

    #[inline(always)]
    fn logic<F>(&mut self, _pc: usize, predicate: F) -> anyhow::Result<usize>
    where
        F: FnOnce(bool, bool) -> bool,
    {
        let right = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;
        let left = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;

        self.evaluation_stack.push(Cow::Owned(Value::Bool(predicate(
            Self::is_truthy(left.as_ref()),
            Self::is_truthy(right.as_ref()),
        ))));

        Ok(1)
    }

    #[inline(always)]
    fn empty(&mut self, _pc: usize, expect_empty: bool) -> anyhow::Result<usize> {
        let value = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;

        let is_empty = Self::is_empty(value.as_ref());
        self.evaluation_stack
            .push(Cow::Owned(Value::Bool(if expect_empty {
                is_empty
            } else {
                !is_empty
            })));

        Ok(1)
    }

    #[inline(always)]
    fn length(&mut self, _pc: usize) -> anyhow::Result<usize> {
        let value = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;

        let len = match value.as_ref() {
            Value::String(val) => val.len() as i64,
            Value::Array(val) => val.len() as i64,
            Value::Object(val) => val.len() as i64,
            _ => 0,
        };

        self.evaluation_stack
            .push(Cow::Owned(Value::Number(Number::from(len))));

        Ok(1)
    }

    #[inline(always)]
    fn concat(&mut self, _pc: usize) -> anyhow::Result<usize> {
        let right = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;
        let left = self
            .evaluation_stack
            .pop()
            .ok_or_else(|| anyhow!("Evaluation stack is empty"))?;

        let mut output = String::new();
        Self::append_value(&mut output, left.as_ref());
        Self::append_value(&mut output, right.as_ref());
        self.evaluation_stack
            .push(Cow::Owned(Value::String(output)));

        Ok(1)
    }

    #[inline(always)]
    fn cleanup_scope_to_depth(&mut self, depth: usize) {
        while self.scope_stack.len() > depth {
            let _ = self.scope_stack.pop();
        }
    }

    #[inline(always)]
    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(val) => *val,
            Value::String(val) => !val.is_empty(),
            Value::Array(val) => !val.is_empty(),
            Value::Object(val) => !val.is_empty(),
            Value::Number(val) => val.as_f64().map(|num| num != 0.0).unwrap_or(false),
        }
    }

    #[inline(always)]
    fn is_empty(value: &Value) -> bool {
        match value {
            Value::Null => true,
            Value::String(val) => val.is_empty(),
            Value::Array(val) => val.is_empty(),
            Value::Object(val) => val.is_empty(),
            _ => false,
        }
    }

    #[inline(always)]
    fn compare_values(left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => {
                let left = left.as_f64()?;
                let right = right.as_f64()?;
                left.partial_cmp(&right)
            }
            (Value::String(left), Value::String(right)) => Some(left.cmp(right)),
            (Value::Bool(left), Value::Bool(right)) => Some(left.cmp(right)),
            (Value::Null, Value::Null) => Some(std::cmp::Ordering::Equal),
            _ => {
                let left = left.to_string();
                let right = right.to_string();
                Some(left.cmp(&right))
            }
        }
    }

    #[inline(always)]
    fn append_value(output: &mut String, val: &Value) {
        if val.is_string() {
            output.push_str(val.as_str().unwrap());
        } else {
            output.push_str(&val.to_string());
        }
    }
}
