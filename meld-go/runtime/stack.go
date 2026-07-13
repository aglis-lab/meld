package runtime

import "strings"

const defaultStackCapacity = 64

// Stack manages a stack of Values for scope or evaluation
type Stack struct {
	values []Value
}

// NewStack creates a new stack
func NewStack() *Stack {
	return &Stack{
		values: make([]Value, 0, defaultStackCapacity), // Pre-allocate reasonable capacity
	}
}

// Push adds a value to the stack
func (s *Stack) Push(v Value) {
	s.values = append(s.values, v)
}

// Pop removes and returns the top value. Returns (nil, false) if empty.
func (s *Stack) Pop() (Value, bool) {
	if len(s.values) == 0 {
		return nil, false
	}
	val := s.values[len(s.values)-1]
	s.values = s.values[:len(s.values)-1]
	return val, true
}

// Peek returns the top value without removing it. Returns nil if empty.
func (s *Stack) Peek() Value {
	if len(s.values) == 0 {
		return nil
	}
	return s.values[len(s.values)-1]
}

// Len returns the number of values on the stack
func (s *Stack) Len() int {
	return len(s.values)
}

// Clear removes all values from the stack
func (s *Stack) Clear() {
	s.values = s.values[:0]
}

// Get retrieves a value at a specific position from the bottom of the stack
func (s *Stack) Get(index int) Value {
	if index < 0 || index >= len(s.values) {
		return nil
	}
	return s.values[index]
}

// DrainRange returns and removes values in the specified range [start, end)
func (s *Stack) DrainRange(start, end int) ([]Value, bool) {
	if start < 0 || end > len(s.values) || start > end {
		return nil, false
	}
	result := make([]Value, end-start)
	copy(result, s.values[start:end])
	s.values = append(s.values[:start], s.values[end:]...)
	return result, true
}

// ScopeStack manages a stack of scope objects for variable lookups
type ScopeStack struct {
	scopes Stack
}

// NewScopeStack creates a new scope stack
func NewScopeStack() *ScopeStack {
	return &ScopeStack{
		scopes: Stack{
			values: make([]Value, 0, defaultStackCapacity),
		},
	}
}

// Push adds a new scope
func (ss *ScopeStack) Push(scope Value) {
	ss.scopes.Push(scope)
}

// Pop removes the current scope
func (ss *ScopeStack) Pop() Value {
	val, _ := ss.scopes.Pop()
	return val
}

// Len returns the number of scopes
func (ss *ScopeStack) Len() int {
	return ss.scopes.Len()
}

// Clear removes all scopes
func (ss *ScopeStack) Clear() {
	ss.scopes.Clear()
}

// Get looks up a variable in the scope stack, traversing from top to bottom
// Supports dot notation for property access (e.g., "item.name")
func (ss *ScopeStack) Get(key string) (Value, bool) {
	// Search from top to bottom for the key
	for i := len(ss.scopes.values) - 1; i >= 0; i-- {
		scope := ss.scopes.values[i]

		// Try to traverse the key parts (handles "item.name" style lookups)
		var currentValue Value = scope
		parts := strings.Split(key, ".")
		matchedCount := 0

		for _, part := range parts {
			if objMap, ok := currentValue.(map[string]interface{}); ok {
				if val, exists := objMap[part]; exists {
					currentValue = val
					matchedCount++
				} else {
					break
				}
			} else {
				break
			}
		}

		// Full match - found all parts in current scope
		if matchedCount == len(parts) {
			return currentValue, true
		}

		// Partial match - don't check outer scopes
		if matchedCount > 0 {
			break
		}
	}
	return nil, false
}

// CleanupToDepth removes all scopes beyond the specified depth
func (ss *ScopeStack) CleanupToDepth(depth int) {
	if depth < 0 {
		ss.scopes.Clear()
		return
	}
	if depth < len(ss.scopes.values) {
		ss.scopes.values = ss.scopes.values[:depth]
	}
}
