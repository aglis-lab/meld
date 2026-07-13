// Render opcodes
pub const END: u8 = 0x00;
pub const TEXT: u8 = 0x01;
pub const OUT: u8 = 0x02;
pub const CONDITION: u8 = 0x03;
pub const ITERATE: u8 = 0x04;
pub const JUMP: u8 = 0x05;
pub const POP_SCOPE: u8 = 0x06;

// Expression opcodes
pub const CALL: u8 = 0x10;
pub const PUSH_CONST: u8 = 0x11;
pub const LOOKUP: u8 = 0x12;
pub const LOOKUP_OUT: u8 = 0x13;

// logic and string operators
pub const EQ: u8 = 0x20;
pub const NEQ: u8 = 0x21;
pub const GT: u8 = 0x22;
pub const GTE: u8 = 0x23;
pub const LT: u8 = 0x24;
pub const LTE: u8 = 0x25;
pub const NOT: u8 = 0x26;
pub const AND: u8 = 0x27;
pub const OR: u8 = 0x28;
pub const EMPTY: u8 = 0x29;
pub const NOT_EMPTY: u8 = 0x2A;
pub const LENGTH: u8 = 0x2B;
pub const CONCAT: u8 = 0x2C;

// Math operators
// Add, Subtract, Multiply, Divide, Modulus
// TODO: Add support for math operators

// Literal opcodes
pub const LITERAL_STRING: u8 = 0x30;
pub const LITERAL_FLOAT: u8 = 0x31;
pub const LITERAL_INT: u8 = 0x32;
pub const LITERAL_BOOL: u8 = 0x33;
pub const LITERAL_NULL: u8 = 0x34;
