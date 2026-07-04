#[derive(Debug)]
pub enum LiteralValue {
    Float(f64),
    Integer(i64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug)]
pub enum Expression {
    Unary {
        operator: u8,
        expr: Box<Expression>,
    },
    Comparison {
        left: Box<Expression>,
        operator: u8,
        right: Box<Expression>,
    },
    Logical {
        left: Box<Expression>,
        operator: u8,
        right: Box<Expression>,
    },
    Call {
        name: String,
        args: Vec<Expression>,
    },
    Variable(String),
    Literal(LiteralValue),
}

#[derive(Debug)]
pub enum Instruction {
    Text(String),
    LookupOut(String),
    Out(Expression),
    If(Expression),
    ElseIf(Expression),
    Else,
    EndIf,
    Each {
        collection: Expression,
        item_name: String,
        index_name: String,
    },
    EndEach,
    End,
}

pub type Value = serde_json::Value;
pub type MapValue = serde_json::Map<String, Value>;
