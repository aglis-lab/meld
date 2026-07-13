package runtime

// Render format opcodes
const (
	OpEnd       = 0x00
	OpText      = 0x01
	OpOut       = 0x02
	OpCondition = 0x03
	OpIterate   = 0x04
	OpJump      = 0x05
	OpPopScope  = 0x06
)

// Expression instruction opcodes
const (
	OpCall      = 0x10
	OpPushConst = 0x11
	OpLookup    = 0x12
	OpLookupOut = 0x13
)

// Logic and comparison operators
const (
	OpEq       = 0x20
	OpNeq      = 0x21
	OpGt       = 0x22
	OpGte      = 0x23
	OpLt       = 0x24
	OpLte      = 0x25
	OpNot      = 0x26
	OpAnd      = 0x27
	OpOr       = 0x28
	OpEmpty    = 0x29
	OpNotEmpty = 0x2A
	OpLength   = 0x2B
	OpConcat   = 0x2C
)

// Literal type codes
const (
	LiteralString  = 0x30
	LiteralFloat   = 0x31
	LiteralInteger = 0x32
	LiteralBool    = 0x33
	LiteralNull    = 0x34
)
