#[derive(Debug)]
pub(super) enum LiteralValue {
    Float(f64),
    Integer(i64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug)]
pub(super) enum Expression {
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

pub(super) struct ExpressionParser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    index: usize,
}

impl<'a> ExpressionParser<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            index: 0,
        }
    }

    pub(super) fn parse_expression(&mut self) -> Expression {
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
