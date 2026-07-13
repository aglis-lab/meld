package runtime

import (
	"encoding/json"
	"fmt"
	"math"
	"strconv"
	"strings"
)

// Value is any JSON-like value (uses Go's any type for dynamic typing)
type Value = any

// FromJSON converts json.RawMessage to Value
func FromJSON(data json.RawMessage) (Value, error) {
	var result any
	if err := json.Unmarshal(data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// ValueString returns string representation of a value
func ValueString(v Value) string {
	switch val := v.(type) {
	case nil:
		return "null"
	case bool:
		if val {
			return "true"
		}
		return "false"
	case float64:
		if val == math.Floor(val) {
			return strconv.FormatInt(int64(val), 10)
		}
		return strconv.FormatFloat(val, 'g', -1, 64)
	case string:
		return val
	case []any:
		strs := make([]string, len(val))
		for i, item := range val {
			strs[i] = ValueString(item)
		}
		return "[" + strings.Join(strs, ",") + "]"
	case map[string]any:
		strs := make([]string, 0, len(val))
		for k, item := range val {
			strs = append(strs, fmt.Sprintf("%q:%s", k, ValueString(item)))
		}
		return "{" + strings.Join(strs, ",") + "}"
	default:
		return ""
	}
}

// IsTruthy checks if a value is truthy
func IsTruthy(v Value) bool {
	switch val := v.(type) {
	case nil:
		return false
	case bool:
		return val
	case float64:
		return !math.IsNaN(val) && val != 0
	case string:
		return len(val) > 0
	case []any:
		return len(val) > 0
	case map[string]any:
		return len(val) > 0
	default:
		return false
	}
}

// IsEmpty checks if a value is empty
func IsEmpty(v Value) bool {
	switch val := v.(type) {
	case nil:
		return true
	case string:
		return len(val) == 0
	case []any:
		return len(val) == 0
	case map[string]any:
		return len(val) == 0
	default:
		return false
	}
}

// Length returns the length of a value
func Length(v Value) int64 {
	switch val := v.(type) {
	case string:
		return int64(len(val))
	case []any:
		return int64(len(val))
	case map[string]any:
		return int64(len(val))
	default:
		return 0
	}
}

// AppendValue appends a value to a strings.Builder
func AppendValue(sb *strings.Builder, val Value) {
	if str, ok := val.(string); ok {
		sb.WriteString(str)
	} else {
		sb.WriteString(ValueString(val))
	}
}

// ValuesEqual checks if two values are equal
func ValuesEqual(left, right Value) bool {
	if left == nil && right == nil {
		return true
	}
	if left == nil || right == nil {
		return false
	}

	switch l := left.(type) {
	case bool:
		r, ok := right.(bool)
		return ok && l == r
	case float64:
		r, ok := right.(float64)
		return ok && l == r
	case string:
		r, ok := right.(string)
		return ok && l == r
	default:
		return false
	}
}

// Compare compares two values for ordering. Returns -1, 0, or 1, or false if not comparable.
func Compare(left, right Value) (int, bool) {
	lType := typeOf(left)
	rType := typeOf(right)

	if lType != rType {
		// Fallback to string comparison
		return strings.Compare(ValueString(left), ValueString(right)), true
	}

	if left == nil && right == nil {
		return 0, true
	}

	switch l := left.(type) {
	case float64:
		r := right.(float64)
		if math.IsNaN(l) || math.IsNaN(r) {
			return 0, false
		}
		if l < r {
			return -1, true
		} else if l > r {
			return 1, true
		}
		return 0, true
	case string:
		r := right.(string)
		return strings.Compare(l, r), true
	case bool:
		r := right.(bool)
		if l == r {
			return 0, true
		}
		if !l && r {
			return -1, true
		}
		return 1, true
	default:
		return 0, false
	}
}

// typeOf returns a string representation of the value's type
func typeOf(v Value) string {
	if v == nil {
		return "null"
	}

	switch v.(type) {
	case bool:
		return "bool"
	case float64:
		return "number"
	case string:
		return "string"
	case []any:
		return "array"
	case map[string]any:
		return "object"
	default:
		return "unknown"
	}
}
