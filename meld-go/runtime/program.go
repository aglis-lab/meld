package runtime

import (
	"encoding/binary"
	"fmt"
)

// Program represents compiled template bytecode
type Program struct {
	Version      uint16
	Instructions []byte
	Content      []byte
}

// NewProgram creates a program from bytecode
func NewProgram(bytecode []byte) (*Program, error) {
	if len(bytecode) < 42 {
		return nil, fmt.Errorf("bytecode too short: minimum 42 bytes required")
	}

	// Read header
	version := binary.LittleEndian.Uint16(bytecode[0:2])
	instructionLen := binary.LittleEndian.Uint32(bytecode[2:6])
	contentLen := binary.LittleEndian.Uint32(bytecode[6:10])
	// checksum := bytecode[10:42] // TODO: validate checksum if needed

	expectedLen := 42 + instructionLen + contentLen
	if uint32(len(bytecode)) < expectedLen {
		return nil, fmt.Errorf("bytecode too short: expected %d bytes, got %d", expectedLen, len(bytecode))
	}

	instructions := bytecode[42 : 42+instructionLen]
	content := bytecode[42+instructionLen : 42+instructionLen+contentLen]

	return &Program{
		Version:      version,
		Instructions: instructions,
		Content:      content,
	}, nil
}

// GetOp gets a single opcode byte at pc
func (p *Program) GetOp(pc uint32) (byte, error) {
	if pc >= uint32(len(p.Instructions)) {
		return 0, fmt.Errorf("pc out of bounds: %d", pc)
	}
	return p.Instructions[pc], nil
}

// GetOpRange gets a range from instructions body (offset + length pair at pc+offset)
func (p *Program) GetOpRange(pc uint32) (uint32, uint32, error) {
	if pc+8 > uint32(len(p.Instructions)) {
		return 0, 0, fmt.Errorf("pc out of bounds for range: %d", pc)
	}
	start := binary.LittleEndian.Uint32(p.Instructions[pc : pc+4])
	end := binary.LittleEndian.Uint32(p.Instructions[pc+4 : pc+8])
	return start, end, nil
}

// GetOpU32 gets a 4-byte little-endian uint32 at pc
func (p *Program) GetOpU32(pc uint32) (uint32, error) {
	if pc+4 > uint32(len(p.Instructions)) {
		return 0, fmt.Errorf("pc out of bounds for u32: %d", pc)
	}
	return binary.LittleEndian.Uint32(p.Instructions[pc : pc+4]), nil
}

// GetContent gets content bytes from start and end
func (p *Program) GetContent(start, end uint32) ([]byte, error) {
	if end > uint32(len(p.Content)) {
		return nil, fmt.Errorf("content out of bounds: start=%d, end=%d, total=%d", start, end, len(p.Content))
	}
	return p.Content[start:end], nil
}

// GetContentString gets content as UTF-8 string
func (p *Program) GetContentString(start, end uint32) (string, error) {
	content, err := p.GetContent(start, end)
	if err != nil {
		return "", err
	}
	return string(content), nil
}
