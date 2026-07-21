export const OpEnd = 0x00;
export const OpText = 0x01;
export const OpOut = 0x02;
export const OpCondition = 0x03;
export const OpIterate = 0x04;
export const OpJump = 0x05;
export const OpPopScope = 0x06;

export const OpCall = 0x10;
export const OpPushConst = 0x11;
export const OpLookup = 0x12;
export const OpLookupOut = 0x13;

export const OpEq = 0x20;
export const OpNeq = 0x21;
export const OpGt = 0x22;
export const OpGte = 0x23;
export const OpLt = 0x24;
export const OpLte = 0x25;
export const OpNot = 0x26;
export const OpAnd = 0x27;
export const OpOr = 0x28;
export const OpEmpty = 0x29;
export const OpNotEmpty = 0x2a;
export const OpLength = 0x2b;
export const OpConcat = 0x2c;

export const LiteralString = 0x30;
export const LiteralFloat = 0x31;
export const LiteralInteger = 0x32;
export const LiteralBool = 0x33;
export const LiteralNull = 0x34;
