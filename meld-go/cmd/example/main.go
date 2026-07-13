package main

import (
	"encoding/binary"
	"encoding/json"
	"fmt"
	"log"
	"os"

	"github.com/aglis-lab/meld-go/runtime"
)

type Content []byte

func (content *Content) length() uint32 {
	return uint32(len(*content))
}

func main() {
	// Example 1: Basic template rendering
	fmt.Println("=== Example 1: Basic Template ===")
	basicExample()

	// Example 2: Variable interpolation
	fmt.Println("\n=== Example 2: Variable Interpolation ===")
	interpolationExample()

	fmt.Println("\n=== Example 3: Conditionals ===")
	conditionalExample()

	// Example 4: Loops
	fmt.Println("\n=== Example 4: Loops ===")
	loopExample()

	// Example 5: Helper functions
	fmt.Println("\n=== Example 5: Helper Functions ===")
	helperExample()

	// Example 6: Read Compiled File
	fmt.Println("\n=== Example 6: Read Compiled File ===")
	compiledExample()
}

// compiledExample
func compiledExample() {
	inputFile := "../templates/comprehensive.bhtml"
	outputFile := "../templates/comprehensive.out.html"
	content, err := os.ReadFile(inputFile)
	if err != nil {
		log.Fatal(err)
	}

	jsonContent, err := os.ReadFile("../templates/comprehensive.json")
	if err != nil {
		log.Fatal(err)
	}

	program, err := runtime.NewProgram(content)
	if err != nil {
		log.Fatal(err)
	}

	payload, err := runtime.FromJSON(jsonContent)
	if err != nil {
		log.Fatal(err)
	}

	eval := runtime.NewRuntime(program, runtime.NewRuntimeConfig())
	err = eval.Run(payload)
	if err != nil {
		log.Fatal(err)
	}

	err = os.WriteFile(outputFile, []byte(eval.Output()), 0644)
	if err != nil {
		log.Fatal(err)
	}
}

// basicExample demonstrates basic template rendering
func basicExample() {
	// Create simple bytecode: TEXT "Hello, World!" then END
	var instructions []byte
	var content []byte

	// TEXT opcode with offset and length
	instructions = append(instructions, runtime.OpText)
	instructions = append(instructions, encodeU32(uint32(len(content)))...)         // offset
	instructions = append(instructions, encodeU32(uint32(len("Hello, World!")))...) // length
	content = append(content, []byte("Hello, World!")...)

	// END opcode
	instructions = append(instructions, runtime.OpEnd)

	bytecode := createBytecode(instructions, content)
	program, err := runtime.NewProgram(bytecode)
	if err != nil {
		log.Fatal(err)
	}

	r := runtime.NewRuntime(program, runtime.NewRuntimeConfig())
	err = r.Run(make(map[string]runtime.Value))
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Output:", r.Output())
}

// interpolationExample demonstrates variable interpolation
func interpolationExample() {
	data := map[string]interface{}{
		"name": "Alice",
		"age":  30,
	}

	jsonData, _ := json.Marshal(data)
	payload, _ := runtime.FromJSON(jsonData)

	// Create bytecode: TEXT "Name: " LOOKUP_OUT "name" TEXT ", Age: " LOOKUP_OUT "age" END
	var instructions []byte
	var content Content

	// TEXT "Name: "
	instructions = append(instructions, runtime.OpText)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+6)...)
	content = append(content, []byte("Name: ")...)

	// LOOKUP_OUT "name"
	instructions = append(instructions, runtime.OpLookupOut)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+4)...)
	content = append(content, []byte("name")...)

	// TEXT ", Age: "
	instructions = append(instructions, runtime.OpText)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+7)...)
	content = append(content, []byte(", Age: ")...)

	// LOOKUP_OUT "age"
	instructions = append(instructions, runtime.OpLookupOut)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+3)...)
	content = append(content, []byte("age")...)

	// END
	instructions = append(instructions, runtime.OpEnd)

	bytecode := createBytecode(instructions, content)
	program, _ := runtime.NewProgram(bytecode)
	r := runtime.NewRuntime(program, runtime.NewRuntimeConfig())
	r.Run(payload)

	fmt.Println("Output:", r.Output())
}

// conditionalExample demonstrates conditional rendering
func conditionalExample() {
	data := map[string]interface{}{
		"admin": true,
	}

	jsonData, _ := json.Marshal(data)
	payload, _ := runtime.FromJSON(jsonData)

	// Create bytecode: LOOKUP "admin" CONDITION (jump to else) TEXT "Admin Panel" JUMP (to end) TEXT "User Panel" END
	var instructions []byte
	var content Content

	// LOOKUP "admin"
	instructions = append(instructions, runtime.OpLookup)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+5)...)
	content = append(content, []byte("admin")...)

	// CONDITION (jump to offset 21 if false)
	elseOffset := uint32(len(instructions)) + 5 + 16 // condition (5) + text opcode (1) + text args (8) + text content (2)
	instructions = append(instructions, runtime.OpCondition)
	instructions = append(instructions, encodeU32(elseOffset)...)

	// TEXT "Admin"
	instructions = append(instructions, runtime.OpText)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+5)...)
	content = append(content, []byte("Admin")...)

	// JUMP to end
	endOffset := uint32(len(instructions)) + 5 + 9 + 4 // jump (5) + text opcode (1) + text args (8) + text content (4)
	instructions = append(instructions, runtime.OpJump)
	instructions = append(instructions, encodeU32(endOffset)...)

	// TEXT "User"
	instructions = append(instructions, runtime.OpText)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+4)...)
	content = append(content, []byte("User")...)

	// END
	instructions = append(instructions, runtime.OpEnd)

	bytecode := createBytecode(instructions, content)
	program, _ := runtime.NewProgram(bytecode)
	r := runtime.NewRuntime(program, runtime.NewRuntimeConfig())
	r.Run(payload)

	fmt.Println("Output:", r.Output())
}

// loopExample demonstrates iteration
func loopExample() {
	data := map[string]interface{}{
		"items": []interface{}{"apple", "banana", "cherry"},
	}

	jsonData, _ := json.Marshal(data)
	payload, _ := runtime.FromJSON(jsonData)

	// Create bytecode: LOOKUP "items" ITERATE ... TEXT ", " LOOKUP_OUT "item" POP_SCOPE ... END
	var instructions []byte
	var content Content

	// LOOKUP "items"
	instructions = append(instructions, runtime.OpLookup)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+5)...)
	content = append(content, []byte("items")...)

	// ITERATE
	loopStart := uint32(len(instructions))
	instructions = append(instructions, runtime.OpIterate)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+4)...)
	content = append(content, []byte("item")...)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+5)...)
	content = append(content, []byte("index")...)
	loopBodyEnd := uint32(len(instructions)) + 4 + 9 + 9 + 1 // offset (4) + loop body text (9) + lookup_out (9) + pop_scope (1)
	instructions = append(instructions, encodeU32(loopBodyEnd)...)

	// TEXT ", "
	instructions = append(instructions, runtime.OpText)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+2)...)
	content = append(content, []byte(", ")...)

	// LOOKUP_OUT "item"
	instructions = append(instructions, runtime.OpLookupOut)
	instructions = append(instructions, encodeU32(content.length())...)
	instructions = append(instructions, encodeU32(content.length()+4)...)
	content = append(content, []byte("item")...)

	// POP_SCOPE
	instructions = append(instructions, runtime.OpPopScope)

	// JUMP back to loop
	instructions = append(instructions, runtime.OpJump)
	instructions = append(instructions, encodeU32(loopStart)...)

	// END
	instructions = append(instructions, runtime.OpEnd)

	bytecode := createBytecode(instructions, content)
	program, _ := runtime.NewProgram(bytecode)
	r := runtime.NewRuntime(program, runtime.NewRuntimeConfig())
	r.Run(payload)

	fmt.Println("Output:", r.Output())
}

// helperExample demonstrates helper functions
func helperExample() {
	data := map[string]interface{}{
		"items": []interface{}{"a", "b", "c"},
	}

	jsonData, _ := json.Marshal(data)
	payload, _ := runtime.FromJSON(jsonData)

	// Create bytecode: LOOKUP "items" CALL "length" OUT END
	var instructions []byte
	var content []byte

	// LOOKUP "items"
	instructions = append(instructions, runtime.OpLookup)
	instructions = append(instructions, encodeU32(uint32(len(content)))...)
	instructions = append(instructions, encodeU32(uint32(len(content)+5))...)
	content = append(content, []byte("items")...)

	// CALL "length"
	instructions = append(instructions, runtime.OpCall)
	instructions = append(instructions, encodeU32(uint32(len(content)))...)
	instructions = append(instructions, encodeU32(uint32(len(content)+6))...)
	content = append(content, []byte("length")...)
	instructions = append(instructions, 1) // arg count

	// OUT
	instructions = append(instructions, runtime.OpOut)

	// END
	instructions = append(instructions, runtime.OpEnd)

	bytecode := createBytecode(instructions, content)
	program, _ := runtime.NewProgram(bytecode)
	r := runtime.NewRuntime(program, runtime.NewRuntimeConfig())
	r.Run(payload)

	fmt.Println("Items count:", r.Output())
}

// Helper functions
func createBytecode(instructions, content []byte) []byte {
	header := make([]byte, 42)
	binary.LittleEndian.PutUint16(header[0:2], 1)
	binary.LittleEndian.PutUint32(header[2:6], uint32(len(instructions)))
	binary.LittleEndian.PutUint32(header[6:10], uint32(len(content)))
	return append(append(header, instructions...), content...)
}

func encodeU32(val uint32) []byte {
	return []byte{byte(val), byte(val >> 8), byte(val >> 16), byte(val >> 24)}
}
