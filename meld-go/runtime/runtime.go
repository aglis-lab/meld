package runtime

import (
	"fmt"
	"strconv"
	"strings"
)

// RuntimeConfig controls runtime behavior
type RuntimeConfig struct {
	IgnoreMissingVariables bool
}

type Callee func(args ...Value) (Value, error)

// NewRuntimeConfig creates a default config
func NewRuntimeConfig() RuntimeConfig {
	return RuntimeConfig{
		IgnoreMissingVariables: true,
	}
}

// Runtime executes template bytecode
type Runtime struct {
	program         *Program
	config          RuntimeConfig
	output          strings.Builder
	scopeStack      *ScopeStack
	scopeMarks      map[uint32]int
	iterateIndices  map[uint32]int
	evaluationStack *Stack
	calleeFunc      map[string]Callee
}

// NewRuntime creates a new runtime
func NewRuntime(program *Program, config RuntimeConfig) *Runtime {
	return &Runtime{
		program:         program,
		config:          config,
		output:          strings.Builder{},
		scopeStack:      NewScopeStack(),
		scopeMarks:      make(map[uint32]int, defaultStackCapacity),
		iterateIndices:  make(map[uint32]int, defaultStackCapacity),
		evaluationStack: NewStack(),
		calleeFunc:      make(map[string]Callee),
	}
}

// Run executes the program with the given payload
func (r *Runtime) Run(payload Value) error {
	r.clear()

	if _, ok := payload.(map[string]any); ok {
		r.scopeStack.Push(payload)
	}

	var pc uint32
	for {
		opcode, err := r.program.GetOp(pc)
		if err != nil {
			return err
		}

		var step uint32
		var stepErr error

		switch opcode {
		case OpEnd:
			return nil
		case OpText:
			step, stepErr = r.text(pc)
		case OpOut:
			step, stepErr = r.out()
		case OpLookup:
			step, stepErr = r.lookup(pc)
		case OpLookupOut:
			step, stepErr = r.lookupOut(pc)
		case OpCall:
			step, stepErr = r.call(pc)
		case OpPushConst:
			step, stepErr = r.pushConst(pc)
		case OpEq:
			step, stepErr = r.compare(func(left, right Value) bool {
				return ValuesEqual(left, right)
			})
		case OpNeq:
			step, stepErr = r.compare(func(left, right Value) bool {
				return !ValuesEqual(left, right)
			})
		case OpGt:
			step, stepErr = r.compareOrdered(func(cmp int) bool { return cmp > 0 })
		case OpGte:
			step, stepErr = r.compareOrdered(func(cmp int) bool { return cmp >= 0 })
		case OpLt:
			step, stepErr = r.compareOrdered(func(cmp int) bool { return cmp < 0 })
		case OpLte:
			step, stepErr = r.compareOrdered(func(cmp int) bool { return cmp <= 0 })
		case OpNot:
			step, stepErr = r.not()
		case OpAnd:
			step, stepErr = r.logic(func(left, right bool) bool { return left && right })
		case OpOr:
			step, stepErr = r.logic(func(left, right bool) bool { return left || right })
		case OpEmpty:
			step, stepErr = r.empty(true)
		case OpNotEmpty:
			step, stepErr = r.empty(false)
		case OpLength:
			step, stepErr = r.length()
		case OpConcat:
			step, stepErr = r.concat()
		case OpCondition:
			var newPc uint32
			newPc, stepErr = r.condition(pc)
			if stepErr != nil {
				return stepErr
			}
			pc = newPc
			continue
		case OpJump:
			var newPc uint32
			newPc, stepErr = r.jump(pc)
			if stepErr != nil {
				return stepErr
			}
			pc = newPc
			continue
		case OpPopScope:
			r.scopeStack.Pop()
			step = 1
		case OpIterate:
			var newPc uint32
			newPc, stepErr = r.iterate(pc)
			if stepErr != nil {
				return stepErr
			}
			pc = newPc
			continue
		default:
			return fmt.Errorf("unknown opcode: %d at pc %d", opcode, pc)
		}

		if stepErr != nil {
			return stepErr
		}

		pc += step
	}
}

// Output returns the rendered template output
func (r *Runtime) Output() string {
	return r.output.String()
}

func (r *Runtime) clear() {
	r.output.Reset()
	r.scopeStack.Clear()
	r.evaluationStack.Clear()
	clear(r.scopeMarks)
	clear(r.iterateIndices)
}

// text handles OpText: output raw static text (9 bytes)
func (r *Runtime) text(pc uint32) (uint32, error) {
	start, end, err := r.program.GetOpRange(pc + 1)
	if err != nil {
		return 0, err
	}
	content, err := r.program.GetContent(start, end)
	if err != nil {
		return 0, err
	}
	r.output.Write(content)
	return 9, nil
}

// out handles OpOut: pop evaluation stack and output the value (1 byte)
func (r *Runtime) out() (uint32, error) {
	val, ok := r.evaluationStack.Pop()
	if !ok {
		return 0, fmt.Errorf("evaluation stack is empty")
	}
	AppendValue(&r.output, val)
	return 1, nil
}

// lookup handles OpLookup: lookup variable and push to evaluation stack (9 bytes)
func (r *Runtime) lookup(pc uint32) (uint32, error) {
	start, end, err := r.program.GetOpRange(pc + 1)
	if err != nil {
		return 0, err
	}
	key, err := r.program.GetContentString(start, end)
	if err != nil {
		return 0, err
	}

	val, found := r.scopeStack.Get(key)
	if !found {
		if !r.config.IgnoreMissingVariables {
			return 0, fmt.Errorf("can't lookup variable %s", key)
		}
		val = nil
	}

	r.evaluationStack.Push(val)
	return 9, nil
}

// lookupOut handles OpLookupOut: lookup variable and output directly (9 bytes)
func (r *Runtime) lookupOut(pc uint32) (uint32, error) {
	start, end, err := r.program.GetOpRange(pc + 1)
	if err != nil {
		return 0, err
	}
	key, err := r.program.GetContentString(start, end)
	if err != nil {
		return 0, err
	}

	val, found := r.scopeStack.Get(key)
	if !found {
		if !r.config.IgnoreMissingVariables {
			return 0, fmt.Errorf("can't lookup variable %s", key)
		}
		return 9, nil
	}

	AppendValue(&r.output, val)
	return 9, nil
}

// pushConst handles OpPushConst: push literal value to evaluation stack (10 bytes)
func (r *Runtime) pushConst(pc uint32) (uint32, error) {
	literalType, err := r.program.GetOp(pc + 1)
	if err != nil {
		return 0, err
	}
	offset, length, err := r.program.GetOpRange(pc + 2)
	if err != nil {
		return 0, err
	}
	content, err := r.program.GetContent(offset, length)
	if err != nil {
		return 0, err
	}

	var val Value
	switch literalType {
	case LiteralString:
		val = string(content)
	case LiteralFloat:
		num, err := strconv.ParseFloat(string(content), 64)
		if err != nil {
			return 0, fmt.Errorf("invalid float literal: %s", string(content))
		}
		val = num
	case LiteralInteger:
		num, err := strconv.ParseInt(string(content), 10, 64)
		if err != nil {
			return 0, fmt.Errorf("invalid integer literal: %s", string(content))
		}
		val = float64(num)
	case LiteralBool:
		val = string(content) == "true"
	case LiteralNull:
		val = nil
	default:
		return 0, fmt.Errorf("unknown literal type: %d", literalType)
	}

	r.evaluationStack.Push(val)
	return 10, nil
}

// call handles OpCall: call helper functions (10 bytes)
func (r *Runtime) call(pc uint32) (uint32, error) {
	start, end, err := r.program.GetOpRange(pc + 1)
	if err != nil {
		return 0, err
	}
	helperName, err := r.program.GetContentString(start, end)
	if err != nil {
		return 0, err
	}

	argCount, err := r.program.GetOp(pc + 9)
	if err != nil {
		return 0, err
	}

	// Get arguments from evaluation stack
	length := r.evaluationStack.Len()
	args, ok := r.evaluationStack.DrainRange(length-int(argCount), length)
	if !ok {
		return 0, fmt.Errorf("not enough arguments on evaluation stack")
	}

	var result Value
	switch helperName {
	case "length":
		if len(args) != 1 {
			return 0, fmt.Errorf("length expects 1 argument, got %d", len(args))
		}
		result = float64(Length(args[0]))

	case "empty":
		if len(args) != 1 {
			return 0, fmt.Errorf("empty expects 1 argument, got %d", len(args))
		}
		result = IsEmpty(args[0])

	case "not_empty":
		if len(args) != 1 {
			return 0, fmt.Errorf("not_empty expects 1 argument, got %d", len(args))
		}
		result = !IsEmpty(args[0])

	case "concat":
		var sb strings.Builder
		for _, arg := range args {
			AppendValue(&sb, arg)
		}
		result = sb.String()

	case "coalesce":
		result = nil
		for _, arg := range args {
			if arg != nil {
				result = arg
				break
			}
		}

	default:
		callee, ok := r.calleeFunc[helperName]
		if !ok {
			return 0, fmt.Errorf("unknown helper: %s", helperName)
		}

		result, err = callee(args...)
		if err != nil {
			return 0, err
		}
	}

	r.evaluationStack.Push(result)
	return 10, nil
}

// iterate handles OpIterate: iterate over arrays (21 bytes)
func (r *Runtime) iterate(pc uint32) (uint32, error) {
	itemStart, itemEnd, err := r.program.GetOpRange(pc + 1)
	if err != nil {
		return 0, err
	}
	indexStart, indexEnd, err := r.program.GetOpRange(pc + 9)
	if err != nil {
		return 0, err
	}
	doneTarget, err := r.program.GetOpU32(pc + 17)
	if err != nil {
		return 0, err
	}

	itemName, err := r.program.GetContentString(itemStart, itemEnd)
	if err != nil {
		return 0, err
	}
	indexName, err := r.program.GetContentString(indexStart, indexEnd)
	if err != nil {
		return 0, err
	}

	// Set base depth if not already set
	baseDepth, exists := r.scopeMarks[pc]
	if !exists {
		baseDepth = r.scopeStack.Len()
		r.scopeMarks[pc] = baseDepth
	}

	// Cleanup scope to base depth
	r.scopeStack.CleanupToDepth(baseDepth)

	// Get collection from evaluation stack
	collection := r.evaluationStack.Peek()
	if collection == nil {
		return 0, fmt.Errorf("iterate expects a collection on evaluation stack")
	}

	if collection == nil {
		// Skip iteration for null values
		delete(r.iterateIndices, pc)
		delete(r.scopeMarks, pc)
		r.evaluationStack.Pop()
		return doneTarget, nil
	}

	arr, ok := collection.([]interface{})
	if !ok {
		return 0, fmt.Errorf("iterate requires array collection")
	}

	// Get next index
	nextIndex := r.iterateIndices[pc]
	if nextIndex >= len(arr) {
		delete(r.iterateIndices, pc)
		delete(r.scopeMarks, pc)
		r.scopeStack.CleanupToDepth(baseDepth)
		r.evaluationStack.Pop()
		return doneTarget, nil
	}

	// Create scope with item and index
	scope := make(map[string]interface{})
	scope[itemName] = arr[nextIndex]
	scope[indexName] = float64(nextIndex)
	r.scopeStack.Push(scope)

	r.iterateIndices[pc] = nextIndex + 1
	return pc + 21, nil
}

// condition handles OpCondition: conditional jump (5 bytes)
func (r *Runtime) condition(pc uint32) (uint32, error) {
	falseTarget, err := r.program.GetOpU32(pc + 1)
	if err != nil {
		return 0, err
	}

	cond, ok := r.evaluationStack.Pop()
	if !ok {
		return 0, fmt.Errorf("evaluation stack is empty")
	}

	if IsTruthy(cond) {
		return pc + 5, nil
	}
	return falseTarget, nil
}

// jump handles OpJump: unconditional jump (5 bytes)
func (r *Runtime) jump(pc uint32) (uint32, error) {
	target, err := r.program.GetOpU32(pc + 1)
	if err != nil {
		return 0, err
	}
	return target, nil
}

// compare handles comparison operators (1 byte each)
func (r *Runtime) compare(predicate func(Value, Value) bool) (uint32, error) {
	right, ok1 := r.evaluationStack.Pop()
	left, ok2 := r.evaluationStack.Pop()
	if !ok1 || !ok2 {
		return 0, fmt.Errorf("evaluation stack is empty")
	}

	result := predicate(left, right)
	r.evaluationStack.Push(result)
	return 1, nil
}

// compareOrdered handles ordered comparison operators (1 byte each)
func (r *Runtime) compareOrdered(predicate func(int) bool) (uint32, error) {
	right, ok1 := r.evaluationStack.Pop()
	left, ok2 := r.evaluationStack.Pop()
	if !ok1 || !ok2 {
		return 0, fmt.Errorf("evaluation stack is empty")
	}

	cmp, ok := Compare(left, right)
	if !ok {
		return 0, fmt.Errorf("values are not comparable")
	}

	result := predicate(cmp)
	r.evaluationStack.Push(result)
	return 1, nil
}

// not handles OpNot (1 byte)
func (r *Runtime) not() (uint32, error) {
	val, ok := r.evaluationStack.Pop()
	if !ok {
		return 0, fmt.Errorf("evaluation stack is empty")
	}

	result := !IsTruthy(val)
	r.evaluationStack.Push(result)
	return 1, nil
}

// logic handles logical operators (1 byte each)
func (r *Runtime) logic(predicate func(bool, bool) bool) (uint32, error) {
	right, ok1 := r.evaluationStack.Pop()
	left, ok2 := r.evaluationStack.Pop()
	if !ok1 || !ok2 {
		return 0, fmt.Errorf("evaluation stack is empty")
	}

	result := predicate(IsTruthy(left), IsTruthy(right))
	r.evaluationStack.Push(result)
	return 1, nil
}

// empty handles OpEmpty and OpNotEmpty (1 byte each)
func (r *Runtime) empty(expectEmpty bool) (uint32, error) {
	val, ok := r.evaluationStack.Pop()
	if !ok {
		return 0, fmt.Errorf("evaluation stack is empty")
	}

	isEmpty := IsEmpty(val)
	if !expectEmpty {
		isEmpty = !isEmpty
	}

	r.evaluationStack.Push(isEmpty)
	return 1, nil
}

// length handles OpLength (1 byte)
func (r *Runtime) length() (uint32, error) {
	val, ok := r.evaluationStack.Pop()
	if !ok {
		return 0, fmt.Errorf("evaluation stack is empty")
	}

	result := float64(Length(val))
	r.evaluationStack.Push(result)
	return 1, nil
}

// concat handles OpConcat (1 byte)
func (r *Runtime) concat() (uint32, error) {
	right, ok1 := r.evaluationStack.Pop()
	left, ok2 := r.evaluationStack.Pop()
	if !ok1 || !ok2 {
		return 0, fmt.Errorf("evaluation stack is empty")
	}

	var sb strings.Builder
	AppendValue(&sb, left)
	AppendValue(&sb, right)

	result := sb.String()
	r.evaluationStack.Push(result)
	return 1, nil
}
