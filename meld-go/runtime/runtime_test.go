package runtime

import (
	"encoding/binary"
	"testing"
)

// TestBasicText tests basic text rendering
func TestBasicText(t *testing.T) {
	bytecode := createTestBytecode([]byte{OpText, 0, 0, 0, 0, 5, 0, 0, 0, OpEnd}, []byte("Hello"))
	program, err := NewProgram(bytecode)
	if err != nil {
		t.Fatalf("Failed to create program: %v", err)
	}

	runtime := NewRuntime(program, NewRuntimeConfig())
	payload := make(map[string]interface{})

	err = runtime.Run(payload)
	if err != nil {
		t.Fatalf("Runtime error: %v", err)
	}

	output := runtime.Output()
	if output != "Hello" {
		t.Errorf("Expected 'Hello', got '%s'", output)
	}
}

// TestLookupOut tests variable lookup and output
func TestLookupOut(t *testing.T) {
	obj := make(map[string]interface{})
	obj["name"] = "World"
	payload := obj

	// OpLookupOut: lookup "name" variable
	bytecode := createTestBytecode(
		[]byte{OpLookupOut, 0, 0, 0, 0, 4, 0, 0, 0, OpEnd},
		[]byte("name"),
	)

	program, err := NewProgram(bytecode)
	if err != nil {
		t.Fatalf("Failed to create program: %v", err)
	}

	runtimes := NewRuntime(program, NewRuntimeConfig())
	err = runtimes.Run(payload)
	if err != nil {
		t.Fatalf("Runtime error: %v", err)
	}

	output := runtimes.Output()
	if output != "World" {
		t.Errorf("Expected 'World', got '%s'", output)
	}
}

// TestComparison tests number comparison operations
func TestComparison(t *testing.T) {
	// Create bytecode: Push 5, Push 3, Compare GT, Output result
	var instructions []byte
	var content []byte

	// Push 5 (integer)
	instructions = append(instructions, OpPushConst, LiteralInteger)
	instructions = append(instructions, encodeU32(uint32(len(content)))...)
	instructions = append(instructions, encodeU32(1)...)
	content = append(content, '5')

	// Push 3 (integer)
	instructions = append(instructions, OpPushConst, LiteralInteger)
	instructions = append(instructions, encodeU32(uint32(len(content)))...)
	instructions = append(instructions, encodeU32(1)...)
	content = append(content, '3')

	// GT comparison
	instructions = append(instructions, OpGt)

	// Output result
	instructions = append(instructions, OpOut)

	// End
	instructions = append(instructions, OpEnd)

	bytecode := createTestBytecode(instructions, content)
	program, err := NewProgram(bytecode)
	if err != nil {
		t.Fatalf("Failed to create program: %v", err)
	}

	rt := NewRuntime(program, NewRuntimeConfig())
	payload := make(map[string]interface{})

	err = rt.Run(payload)
	if err != nil {
		t.Fatalf("Runtime error: %v", err)
	}

	output := rt.Output()
	if output != "true" {
		t.Errorf("Expected 'true', got '%s'", output)
	}
}

// TestHelper tests helper functions
func TestHelperLength(t *testing.T) {
	obj := make(map[string]interface{})
	obj["items"] = []interface{}{
		"a",
		"b",
		"c",
	}
	payload := obj

	// Create bytecode: Lookup "items", call length, output result
	var instructions []byte
	var content []byte

	// Lookup "items"
	instructions = append(instructions, OpLookup)
	instructions = append(instructions, encodeU32(uint32(len(content)))...)
	instructions = append(instructions, encodeU32(5)...)
	content = append(content, []byte("items")...)

	// Call "length" with 1 arg
	instructions = append(instructions, OpCall)
	instructions = append(instructions, encodeU32(uint32(len(content)))...)
	instructions = append(instructions, encodeU32(6)...)
	content = append(content, []byte("length")...)
	instructions = append(instructions, 1) // arg count

	// Output result
	instructions = append(instructions, OpOut)

	// End
	instructions = append(instructions, OpEnd)

	bytecode := createTestBytecode(instructions, content)
	program, err := NewProgram(bytecode)
	if err != nil {
		t.Fatalf("Failed to create program: %v", err)
	}

	rt := NewRuntime(program, NewRuntimeConfig())
	err = rt.Run(payload)
	if err != nil {
		t.Fatalf("Runtime error: %v", err)
	}

	output := rt.Output()
	if output != "3" {
		t.Errorf("Expected '3', got '%s'", output)
	}
}

// encodeU32 encodes a uint32 in little-endian format
func encodeU32(val uint32) []byte {
	return []byte{
		byte(val),
		byte(val >> 8),
		byte(val >> 16),
		byte(val >> 24),
	}
}

// createTestBytecode creates a valid TEF bytecode for testing
func createTestBytecode(instructions []byte, content []byte) []byte {
	// Header: version (2) + instr_len (4) + content_len (4) + checksum (32)
	header := make([]byte, 42)
	binary.LittleEndian.PutUint16(header[0:2], 1)                         // version
	binary.LittleEndian.PutUint32(header[2:6], uint32(len(instructions))) // instruction length
	binary.LittleEndian.PutUint32(header[6:10], uint32(len(content)))     // content length
	// Skip checksum (bytes 10-42)

	return append(append(header, instructions...), content...)
}

// BenchmarkRuntime benchmarks the runtime performance
func BenchmarkRuntime(b *testing.B) {
	obj := make(map[string]interface{})
	obj["name"] = "Alice"
	obj["age"] = float64(30)
	obj["items"] = []interface{}{
		"item1",
		"item2",
		"item3",
	}
	payload := obj

	// Create a simple bytecode that outputs a variable
	bytecode := createTestBytecode(
		[]byte{OpLookupOut, 0, 0, 0, 0, 4, 0, 0, 0, OpEnd},
		[]byte("name"),
	)

	program, _ := NewProgram(bytecode)
	config := NewRuntimeConfig()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		runtime := NewRuntime(program, config)
		_ = runtime.Run(payload)
	}
}
