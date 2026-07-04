use std::{ops::Range, vec};

use super::instruction::{Expression, Instruction, LiteralValue};
use crate::{opcode, utils};

#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("Unmatched open brace at position {0}")]
    UnmatchedOpenBrace(usize),
    #[error("Invalid variable name in range {0:?}")]
    InvalidVariableName(Range<usize>),
    #[error("Invalid conditional structure: {0}")]
    InvalidConditionalStructure(&'static str),
}

#[derive(Debug, PartialEq)]
enum TokenType {
    Variable, // {{
    If,       // {if
    ElseIf,   // {else if
    Else,     // {else
    EndIf,    // {/if}
    Each,     // {each
    EndEach,  // {/each}
    None,     // no token
}

struct ExpressionParser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    index: usize,
}

struct ConditionalState {
    false_jump_pos: usize,
    end_jump_positions: vec::Vec<usize>,
    saw_else: bool,
}

struct LoopState {
    loop_start_pc: usize,
    loop_continue_marker: usize,
}

pub struct Builder {
    version: u16,
    instructions: vec::Vec<Instruction>,
}

// TODO: Optimize output by using a pool of buffers
impl Builder {
    pub fn new() -> Self {
        Self {
            instructions: vec::Vec::new(),
            version: utils::tef_version().expect("Failed to get TEF version"),
        }
    }

    // TODO: Optimize looping through the data by using a more efficient algorithm
    pub fn build(&mut self, data: &vec::Vec<u8>) -> Result<(), BuilderError> {
        let mut prev_index = 0;
        let mut index = 0;

        while index < data.len() {
            match data[index] {
                b'{' => {
                    let token_type = self.identify_token(index, data);
                    match token_type {
                        TokenType::Variable => {
                            let (expr_str, new_index) = self.lookup(index + 2, data)?;
                            if prev_index < index {
                                self.instructions.push(Instruction::Text(
                                    String::from_utf8_lossy(&data[prev_index..index]).to_string(),
                                ));
                            }

                            // Parse as expression to support function calls and complex expressions
                            let expr = self.parse_expression(expr_str.trim().into());
                            match expr {
                                Expression::Variable(ref name) => {
                                    // Simple variable - use optimized LookupOut
                                    self.instructions.push(Instruction::LookupOut(name.clone()));
                                }
                                _ => {
                                    // Complex expression (function call, etc.) - use Out
                                    self.instructions.push(Instruction::Out(expr));
                                }
                            }

                            prev_index = new_index;
                            index = new_index;
                        }
                        TokenType::If => {
                            let (expr, new_index) = self.parse_condition(index + 3, data)?;
                            if prev_index < index {
                                self.instructions.push(Instruction::Text(
                                    String::from_utf8_lossy(&data[prev_index..index]).to_string(),
                                ));
                            }
                            self.instructions.push(Instruction::If(expr));
                            prev_index = new_index;
                            index = new_index;
                        }
                        TokenType::ElseIf => {
                            let (expr, new_index) = self.parse_condition(index + 8, data)?;
                            if prev_index < index {
                                self.instructions.push(Instruction::Text(
                                    String::from_utf8_lossy(&data[prev_index..index]).to_string(),
                                ));
                            }
                            self.instructions.push(Instruction::ElseIf(expr));
                            prev_index = new_index;
                            index = new_index;
                        }
                        TokenType::Else => {
                            if prev_index < index {
                                self.instructions.push(Instruction::Text(
                                    String::from_utf8_lossy(&data[prev_index..index]).to_string(),
                                ));
                            }
                            self.instructions.push(Instruction::Else);
                            prev_index = index + 6; // len("{else}") = 6
                            index = index + 6;
                        }
                        TokenType::EndIf => {
                            if prev_index < index {
                                self.instructions.push(Instruction::Text(
                                    String::from_utf8_lossy(&data[prev_index..index]).to_string(),
                                ));
                            }
                            self.instructions.push(Instruction::EndIf);
                            prev_index = index + 5; // len("{/if}") = 5
                            index = index + 5;
                        }
                        TokenType::Each => {
                            let (collection, item_name, index_name, new_index) =
                                self.parse_each(index + 5, data)?;
                            if prev_index < index {
                                self.instructions.push(Instruction::Text(
                                    String::from_utf8_lossy(&data[prev_index..index]).to_string(),
                                ));
                            }
                            self.instructions.push(Instruction::Each {
                                collection,
                                item_name,
                                index_name,
                            });
                            prev_index = new_index;
                            index = new_index;
                        }
                        TokenType::EndEach => {
                            if prev_index < index {
                                self.instructions.push(Instruction::Text(
                                    String::from_utf8_lossy(&data[prev_index..index]).to_string(),
                                ));
                            }
                            self.instructions.push(Instruction::EndEach);
                            prev_index = index + 7; // len("{/each}") = 7
                            index = index + 7;
                        }
                        TokenType::None => index += 1,
                    }
                }
                b'<' => {
                    if data.get(index..index + 4) == Some(b"<!--") {
                        let new_index = self.skip_comment(index + 4, data);
                        if prev_index < index {
                            self.instructions.push(Instruction::Text(
                                String::from_utf8_lossy(&data[prev_index..index]).to_string(),
                            ));
                        }
                        prev_index = new_index;
                        index = new_index;
                    } else {
                        index += 1;
                    }
                }
                _ => index += 1,
            }
        }

        if prev_index < data.len() {
            self.instructions.push(Instruction::Text(
                String::from_utf8_lossy(&data[prev_index..]).to_string(),
            ));
        }

        self.instructions.push(Instruction::End);

        Ok(())
    }

    // Layout
    // Header | TEF | SYMBOLS | CONTENT
    pub fn compile(&self) -> Result<vec::Vec<u8>, BuilderError> {
        let mut tef_body = vec::Vec::new();
        let mut content = vec::Vec::new();
        let mut symbol = vec::Vec::new();
        let symbol_length = self.calculate_symbol_length() as u32;
        let mut conditional_stack: vec::Vec<ConditionalState> = vec::Vec::new();
        let mut loop_stack: vec::Vec<LoopState> = vec::Vec::new();

        for instruction in &self.instructions {
            match instruction {
                Instruction::End => tef_body.push(0x00),
                Instruction::Text(val) => {
                    tef_body.push(opcode::TEXT);
                    tef_body
                        .extend_from_slice(&(content.len() as u32 + symbol_length).to_le_bytes());
                    tef_body.extend_from_slice(
                        &(content.len() as u32 + symbol_length + val.len() as u32).to_le_bytes(),
                    );

                    content.extend_from_slice(val.as_bytes());
                }
                Instruction::LookupOut(val) => {
                    tef_body.push(opcode::LOOKUP_OUT);
                    tef_body.extend_from_slice(&(symbol.len() as u32).to_le_bytes());
                    tef_body
                        .extend_from_slice(&(symbol.len() as u32 + val.len() as u32).to_le_bytes());

                    symbol.extend_from_slice(val.as_bytes());
                }
                Instruction::Out(expr) => {
                    // Emit expression and then OUT
                    self.emit_expression(
                        expr,
                        &mut tef_body,
                        &mut content,
                        &mut symbol,
                        symbol_length,
                    );
                    tef_body.push(opcode::OUT);
                }
                Instruction::If(expr) => {
                    self.emit_expression(
                        expr,
                        &mut tef_body,
                        &mut content,
                        &mut symbol,
                        symbol_length,
                    );
                    tef_body.push(opcode::CONDITION);
                    let false_jump_pos = tef_body.len();
                    tef_body.extend_from_slice(&0u32.to_le_bytes());

                    conditional_stack.push(ConditionalState {
                        false_jump_pos,
                        end_jump_positions: vec::Vec::new(),
                        saw_else: false,
                    });
                }
                Instruction::ElseIf(expr) => {
                    let state = conditional_stack.last_mut().ok_or(
                        BuilderError::InvalidConditionalStructure("elseif without matching if"),
                    )?;

                    if state.saw_else {
                        return Err(BuilderError::InvalidConditionalStructure(
                            "elseif after else",
                        ));
                    }

                    let jump_pos = self.emit_jump_placeholder(&mut tef_body);
                    state.end_jump_positions.push(jump_pos);

                    let branch_start = tef_body.len() as u32;
                    Self::patch_u32(&mut tef_body, state.false_jump_pos, branch_start);

                    self.emit_expression(
                        expr,
                        &mut tef_body,
                        &mut content,
                        &mut symbol,
                        symbol_length,
                    );
                    tef_body.push(opcode::CONDITION);
                    state.false_jump_pos = tef_body.len();
                    tef_body.extend_from_slice(&0u32.to_le_bytes());
                }
                Instruction::Else => {
                    let state = conditional_stack.last_mut().ok_or(
                        BuilderError::InvalidConditionalStructure("else without matching if"),
                    )?;

                    if state.saw_else {
                        return Err(BuilderError::InvalidConditionalStructure("duplicate else"));
                    }

                    let jump_pos = self.emit_jump_placeholder(&mut tef_body);
                    state.end_jump_positions.push(jump_pos);

                    let else_start = tef_body.len() as u32;
                    Self::patch_u32(&mut tef_body, state.false_jump_pos, else_start);
                    state.saw_else = true;
                }
                Instruction::EndIf => {
                    let state = conditional_stack.pop().ok_or(
                        BuilderError::InvalidConditionalStructure("endif without matching if"),
                    )?;

                    let end_target = tef_body.len() as u32;
                    if !state.saw_else {
                        Self::patch_u32(&mut tef_body, state.false_jump_pos, end_target);
                    }

                    for jump_pos in state.end_jump_positions {
                        Self::patch_u32(&mut tef_body, jump_pos, end_target);
                    }
                }
                Instruction::Each {
                    collection,
                    item_name,
                    index_name,
                } => {
                    // Emit collection expression
                    self.emit_expression(
                        collection,
                        &mut tef_body,
                        &mut content,
                        &mut symbol,
                        symbol_length,
                    );
                    // Emit ITERATE instruction
                    tef_body.push(opcode::ITERATE);
                    // Item name: offset & length
                    tef_body.extend_from_slice(&(symbol.len() as u32).to_le_bytes());
                    tef_body.extend_from_slice(
                        &(symbol.len() as u32 + item_name.len() as u32).to_le_bytes(),
                    );
                    symbol.extend_from_slice(item_name.as_bytes());
                    // Index name: offset & length
                    tef_body.extend_from_slice(&(symbol.len() as u32).to_le_bytes());
                    tef_body.extend_from_slice(
                        &(symbol.len() as u32 + index_name.len() as u32).to_le_bytes(),
                    );
                    symbol.extend_from_slice(index_name.as_bytes());
                    // Done target placeholder
                    let loop_continue_marker = tef_body.len();
                    tef_body.extend_from_slice(&0u32.to_le_bytes());

                    loop_stack.push(LoopState {
                        loop_start_pc: tef_body.len() - 21,
                        loop_continue_marker,
                    });
                }
                Instruction::EndEach => {
                    let loop_state =
                        loop_stack
                            .pop()
                            .ok_or(BuilderError::InvalidConditionalStructure(
                                "end each without each",
                            ))?;

                    // Emit JUMP back to loop start
                    tef_body.push(opcode::JUMP);
                    tef_body.extend_from_slice(&(loop_state.loop_start_pc as u32).to_le_bytes());

                    // Patch the done target
                    let end_target = tef_body.len() as u32;
                    Self::patch_u32(&mut tef_body, loop_state.loop_continue_marker, end_target);

                    // ITERATE already cleans up the loop scope when done, so no POP_SCOPE needed
                }
            }
        }

        if !conditional_stack.is_empty() {
            return Err(BuilderError::InvalidConditionalStructure(
                "unclosed if block",
            ));
        }

        if !loop_stack.is_empty() {
            return Err(BuilderError::InvalidConditionalStructure(
                "unclosed each block",
            ));
        }

        let version = self.version;
        let tef_body_length = tef_body.len() as u32;
        let symbol_length = symbol.len() as u32;
        let content_length = content.len() as u32 + symbol_length;
        let content = [symbol, content].concat();
        let checksum = hex::decode(sha256::digest(&content)).unwrap();

        let mut output = vec::Vec::new();
        output.extend_from_slice(&version.to_le_bytes());
        output.extend_from_slice(&tef_body_length.to_le_bytes());
        output.extend_from_slice(&content_length.to_le_bytes());
        output.extend_from_slice(&checksum);
        output.extend_from_slice(&tef_body);
        output.extend_from_slice(&content);

        Ok(output)
    }

    fn emit_expression(
        &self,
        expr: &Expression,
        tef_body: &mut vec::Vec<u8>,
        content: &mut vec::Vec<u8>,
        symbol: &mut vec::Vec<u8>,
        symbol_length: u32,
    ) {
        match expr {
            Expression::Literal(value) => {
                self.emit_literal(value, tef_body, content, symbol_length)
            }
            Expression::Variable(name) => self.emit_lookup(name, tef_body, symbol),
            Expression::Call { name, args } => {
                // Emit arguments first (they go on the eval stack)
                for arg in args {
                    self.emit_expression(arg, tef_body, content, symbol, symbol_length);
                }
                // Emit CALL instruction
                tef_body.push(opcode::CALL);
                tef_body.extend_from_slice(&(symbol.len() as u32).to_le_bytes());
                tef_body
                    .extend_from_slice(&(symbol.len() as u32 + name.len() as u32).to_le_bytes());
                tef_body.push(args.len() as u8);

                symbol.extend_from_slice(name.as_bytes());
            }
            Expression::Unary { operator, expr } => {
                self.emit_expression(expr, tef_body, content, symbol, symbol_length);
                tef_body.push(self.map_unary_operator(*operator));
            }
            Expression::Comparison {
                left,
                operator,
                right,
            } => {
                self.emit_expression(left, tef_body, content, symbol, symbol_length);
                self.emit_expression(right, tef_body, content, symbol, symbol_length);
                tef_body.push(self.map_comparison_operator(*operator));
            }
            Expression::Logical {
                left,
                operator,
                right,
            } => {
                self.emit_expression(left, tef_body, content, symbol, symbol_length);
                self.emit_expression(right, tef_body, content, symbol, symbol_length);
                tef_body.push(self.map_logical_operator(*operator));
            }
        }
    }

    fn emit_literal(
        &self,
        value: &LiteralValue,
        tef_body: &mut vec::Vec<u8>,
        content: &mut vec::Vec<u8>,
        symbol_length: u32,
    ) {
        let (literal_type, raw_value): (u8, String) = match value {
            LiteralValue::Float(val) => (opcode::LITERAL_FLOAT, val.to_string()),
            LiteralValue::Integer(val) => (opcode::LITERAL_INT, val.to_string()),
            LiteralValue::String(val) => (opcode::LITERAL_STRING, val.clone()),
            LiteralValue::Bool(val) => (opcode::LITERAL_BOOL, val.to_string()),
            LiteralValue::Null => (opcode::LITERAL_NULL, String::new()),
        };

        tef_body.push(opcode::PUSH_CONST);
        tef_body.push(literal_type);

        let start = symbol_length + content.len() as u32;
        let end = start + raw_value.len() as u32;
        tef_body.extend_from_slice(&start.to_le_bytes());
        tef_body.extend_from_slice(&end.to_le_bytes());

        content.extend_from_slice(raw_value.as_bytes());
    }

    fn emit_lookup(&self, name: &str, tef_body: &mut vec::Vec<u8>, symbol: &mut vec::Vec<u8>) {
        tef_body.push(opcode::LOOKUP);
        tef_body.extend_from_slice(&(symbol.len() as u32).to_le_bytes());
        tef_body.extend_from_slice(&(symbol.len() as u32 + name.len() as u32).to_le_bytes());
        symbol.extend_from_slice(name.as_bytes());
    }

    #[inline(always)]
    fn calculate_symbol_length(&self) -> usize {
        let mut total = 0usize;

        for instruction in &self.instructions {
            match instruction {
                Instruction::LookupOut(name) => {
                    total += name.len();
                }
                Instruction::Out(expr) => {
                    total += Self::expression_symbol_length(expr);
                }
                Instruction::If(expr) | Instruction::ElseIf(expr) => {
                    total += Self::expression_symbol_length(expr);
                }
                Instruction::Each {
                    collection,
                    item_name,
                    index_name,
                } => {
                    total += Self::expression_symbol_length(collection);
                    total += item_name.len();
                    total += index_name.len();
                }
                _ => {}
            }
        }

        total
    }

    fn expression_symbol_length(expr: &Expression) -> usize {
        match expr {
            Expression::Variable(name) => name.len(),
            Expression::Literal(_) => 0,
            Expression::Call { name, args } => {
                name.len()
                    + args
                        .iter()
                        .map(Self::expression_symbol_length)
                        .sum::<usize>()
            }
            Expression::Unary { expr, .. } => Self::expression_symbol_length(expr),
            Expression::Comparison { left, right, .. }
            | Expression::Logical { left, right, .. } => {
                Self::expression_symbol_length(left) + Self::expression_symbol_length(right)
            }
        }
    }

    fn emit_jump_placeholder(&self, tef_body: &mut vec::Vec<u8>) -> usize {
        tef_body.push(opcode::JUMP);
        let operand_pos = tef_body.len();
        tef_body.extend_from_slice(&0u32.to_le_bytes());
        operand_pos
    }

    fn patch_u32(tef_body: &mut vec::Vec<u8>, at: usize, value: u32) {
        tef_body[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn map_comparison_operator(&self, operator: u8) -> u8 {
        match operator {
            b'=' => opcode::EQ,
            b'!' => opcode::NEQ,
            b'<' => opcode::LT,
            b'>' => opcode::GT,
            b'L' => opcode::LTE,
            b'G' => opcode::GTE,
            _ => opcode::EQ,
        }
    }

    fn map_logical_operator(&self, operator: u8) -> u8 {
        match operator {
            b'&' => opcode::AND,
            b'|' => opcode::OR,
            _ => opcode::AND,
        }
    }

    fn map_unary_operator(&self, operator: u8) -> u8 {
        match operator {
            b'!' => opcode::NOT,
            _ => opcode::NOT,
        }
    }

    #[inline(always)]
    fn identify_token(&self, index: usize, data: &vec::Vec<u8>) -> TokenType {
        if data.get(index..index + 2) == Some(b"{{") {
            TokenType::Variable
        } else if data.get(index..index + 8) == Some(b"{else if") {
            TokenType::ElseIf
        } else if data.get(index..index + 7) == Some(b"{/each}") {
            TokenType::EndEach
        } else if data.get(index..index + 5) == Some(b"{each") {
            TokenType::Each
        } else if data.get(index..index + 5) == Some(b"{else") {
            TokenType::Else
        } else if data.get(index..index + 3) == Some(b"{if") {
            TokenType::If
        } else if data.get(index..index + 5) == Some(b"{/if}") {
            TokenType::EndIf
        } else {
            TokenType::None
        }
    }

    #[inline(always)]
    fn lookup(
        &self,
        mut index: usize,
        data: &vec::Vec<u8>,
    ) -> Result<(String, usize), BuilderError> {
        let left = index;
        while index < data.len() - 1 {
            let chs = &data.get(index..index + 2).unwrap();
            if chs == b"}}" {
                let content = String::from_utf8_lossy(&data[left..index].trim_ascii());
                return Ok((content.to_string(), index + 2));
            }

            index += 1;
        }

        Err(BuilderError::UnmatchedOpenBrace(left))
    }

    #[inline(always)]
    fn skip_comment(&self, mut index: usize, data: &vec::Vec<u8>) -> usize {
        let mut end_comment = false;
        while index < data.len() - 3 {
            let chs = &data.get(index..index + 3).unwrap();
            if chs == b"-->" {
                index += 3;
                end_comment = true;
                break;
            }

            index += 1;
        }

        if !end_comment {
            index = data.len();
        }

        index
    }

    #[inline(always)]
    fn parse_condition(
        &self,
        mut index: usize,
        data: &vec::Vec<u8>,
    ) -> Result<(Expression, usize), BuilderError> {
        let left = index;
        while index < data.len() {
            if data.get(index..index + 1) == Some(b"}") {
                let expr = String::from_utf8_lossy(&data[left..index].trim_ascii());
                if expr.is_empty() {
                    return Err(BuilderError::UnmatchedOpenBrace(left));
                }

                let expr = self.parse_expression(expr);

                return Ok((expr, index + 1));
            }

            index += 1;
        }

        Err(BuilderError::UnmatchedOpenBrace(left))
    }

    #[inline(always)]
    fn parse_each(
        &self,
        mut index: usize,
        data: &vec::Vec<u8>,
    ) -> Result<(Expression, String, String, usize), BuilderError> {
        let left = index;
        while index < data.len() {
            if data.get(index..index + 1) == Some(b"}") {
                let content = String::from_utf8_lossy(&data[left..index].trim_ascii());
                if content.is_empty() {
                    return Err(BuilderError::UnmatchedOpenBrace(left));
                }

                // Parse: "collection as item, index"
                let parts: Vec<&str> = content.split(" as ").collect();
                if parts.len() != 2 {
                    return Err(BuilderError::UnmatchedOpenBrace(left));
                }

                let collection_expr = self.parse_expression(parts[0].trim().into());
                let vars: Vec<&str> = parts[1].split(',').collect();
                if vars.len() < 1 {
                    return Err(BuilderError::UnmatchedOpenBrace(left));
                }

                let item_name = vars[0].trim().to_string();
                let index_name = if vars.len() > 1 {
                    vars[1].trim().to_string()
                } else {
                    "index".to_string()
                };

                return Ok((collection_expr, item_name, index_name, index + 1));
            }

            index += 1;
        }

        Err(BuilderError::UnmatchedOpenBrace(left))
    }

    fn parse_expression(&self, expr: std::borrow::Cow<str>) -> Expression {
        let mut parser = ExpressionParser::new(expr.trim());
        parser.parse_expression()
    }
}

impl<'a> ExpressionParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            index: 0,
        }
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Expression {
        let mut left = self.parse_logical_and();

        loop {
            self.skip_whitespace();
            if self.consume("||") {
                let right = self.parse_logical_and();
                left = Expression::Logical {
                    left: Box::new(left),
                    operator: b'|',
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        left
    }

    fn parse_logical_and(&mut self) -> Expression {
        let mut left = self.parse_comparison();

        loop {
            self.skip_whitespace();
            if self.consume("&&") {
                let right = self.parse_comparison();
                left = Expression::Logical {
                    left: Box::new(left),
                    operator: b'&',
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        left
    }

    fn parse_comparison(&mut self) -> Expression {
        let left = self.parse_primary();
        self.skip_whitespace();

        let operator = if self.consume("==") {
            Some(b'=')
        } else if self.consume("!=") {
            Some(b'!')
        } else if self.consume("<=") {
            Some(b'L')
        } else if self.consume(">=") {
            Some(b'G')
        } else if self.consume("<") {
            Some(b'<')
        } else if self.consume(">") {
            Some(b'>')
        } else {
            None
        };

        if let Some(operator) = operator {
            let right = self.parse_primary();
            Expression::Comparison {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }
        } else {
            left
        }
    }

    fn parse_primary(&mut self) -> Expression {
        self.skip_whitespace();

        if self.consume("!") {
            return Expression::Unary {
                operator: b'!',
                expr: Box::new(self.parse_primary()),
            };
        }

        if self.consume("(") {
            let expr = self.parse_expression();
            self.skip_whitespace();
            let _ = self.consume(")");
            return expr;
        }

        if self.consume_keyword("true") {
            return Expression::Literal(LiteralValue::Bool(true));
        }

        if self.consume_keyword("false") {
            return Expression::Literal(LiteralValue::Bool(false));
        }

        if self.consume_keyword("null") {
            return Expression::Literal(LiteralValue::Null);
        }

        if self.peek() == Some(b'"') {
            return self.parse_string_literal();
        }

        if let Some(expression) = self.parse_number_literal() {
            return expression;
        }

        self.parse_variable_or_call()
    }

    fn parse_variable_or_call(&mut self) -> Expression {
        let start = self.index;

        while let Some(ch) = self.peek() {
            match ch {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'.' => self.index += 1,
                _ => break,
            }
        }

        let name = self.input[start..self.index].trim().to_string();

        self.skip_whitespace();
        if self.consume("(") {
            // This is a function call
            let mut args = vec![];
            self.skip_whitespace();

            if !self.starts_with(")") {
                loop {
                    args.push(self.parse_expression());
                    self.skip_whitespace();

                    if self.consume(",") {
                        self.skip_whitespace();
                    } else {
                        break;
                    }
                }
            }

            self.skip_whitespace();
            let _ = self.consume(")");

            Expression::Call { name, args }
        } else {
            Expression::Variable(name)
        }
    }

    fn parse_string_literal(&mut self) -> Expression {
        self.index += 1;
        let start = self.index;

        while let Some(ch) = self.peek() {
            if ch == b'"' {
                let value = self.input[start..self.index].to_string();
                self.index += 1;
                return Expression::Literal(LiteralValue::String(value));
            }
            self.index += 1;
        }

        Expression::Literal(LiteralValue::String(self.input[start..].to_string()))
    }

    fn parse_number_literal(&mut self) -> Option<Expression> {
        let start = self.index;
        let mut has_digits = false;
        let mut has_dot = false;

        if self.peek() == Some(b'-') {
            self.index += 1;
        }

        while let Some(ch) = self.peek() {
            match ch {
                b'0'..=b'9' => {
                    has_digits = true;
                    self.index += 1;
                }
                b'.' if !has_dot => {
                    has_dot = true;
                    self.index += 1;
                }
                _ => break,
            }
        }

        if !has_digits {
            self.index = start;
            return None;
        }

        let value = &self.input[start..self.index];
        if has_dot {
            value
                .parse::<f64>()
                .ok()
                .map(|value| Expression::Literal(LiteralValue::Float(value)))
        } else {
            value
                .parse::<i64>()
                .ok()
                .map(|value| Expression::Literal(LiteralValue::Integer(value)))
        }
    }

    fn starts_with(&self, token: &str) -> bool {
        self.input[self.index..].starts_with(token)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.index += 1;
        }
    }

    fn consume(&mut self, token: &str) -> bool {
        if self.input[self.index..].starts_with(token) {
            self.index += token.len();
            true
        } else {
            false
        }
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        if self.input[self.index..].starts_with(keyword) {
            let next_index = self.index + keyword.len();
            if next_index >= self.input.len() {
                self.index = next_index;
                return true;
            }

            let next = self.input.as_bytes()[next_index];
            if !matches!(next, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'.') {
                self.index = next_index;
                return true;
            }
        }

        false
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.index).copied()
    }
}
