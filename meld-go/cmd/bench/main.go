package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"math"
	"os"
	"text/template"
	"time"

	meldruntime "github.com/aglis-lab/meld-go/runtime"
	"github.com/nikolalohinski/gonja/v2"
	"github.com/nikolalohinski/gonja/v2/exec"
)

const SAMPLE_SIZE = 20

var BENCH_ITERATIONS = []uint64{
	10_000,
	20_000,
	30_000,
	40_000,
	50_000,
	60_000,
	70_000,
	80_000,
	90_000,
	100_000,
}

func main() {
	// Handlebars isn't benchmarked because it's have bad support at the moment
	// handlebarsBenchmark()

	// htmlBenchmark("stats/html.csv")
	// gonjaBenchmark("stats/gonja.csv")
	meldBenchmark("stats/meld.csv")
}

// html template
func htmlBenchmark(csvPath string) {
	file, err := os.OpenFile(csvPath, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0666)
	if err != nil {
		log.Fatalf("Failed to open CSV file: %v\n", err)
	}
	defer file.Close()

	jsonPath := "../templates/meld.json"
	templatePath := "../templates/html.html"

	// Read JSON payload
	jsonData, err := os.ReadFile(jsonPath)
	if err != nil {
		log.Fatalf("Failed to read JSON file: %v\n", err)
	}

	var payload map[string]any
	if err := json.Unmarshal(jsonData, &payload); err != nil {
		log.Fatalf("Failed to unmarshal JSON: %v\n", err)
	}

	tmpl, err := template.ParseFiles(templatePath)
	if err != nil {
		log.Fatalf("Failed to parse template: %v\n", err)
	}

	// Print header
	fmt.Println("Html/Template Engine Benchmark")
	fmt.Println("================================")
	fmt.Printf("Template: %s\n", templatePath)
	fmt.Printf("Payload: %s\n", jsonPath)
	fmt.Printf("Sample Size: %d\n\n", SAMPLE_SIZE)
	fmt.Fprintln(file, "n,duration_ns,std_ns,throughput")

	// Run benchmarks
	for _, iterations := range BENCH_ITERATIONS {
		times := make([]float64, SAMPLE_SIZE)

		// Run SAMPLE_SIZE iterations
		for i := range SAMPLE_SIZE {

			start := time.Now()

			// Run `iterations` executions inside the timed region —
			for range iterations {
				output := bytes.Buffer{}
				if err := tmpl.Execute(&output, payload); err != nil {
					log.Fatalf("Failed to execute template: %v\n", err)
				}
				_ = output.String()
			}

			elapsed := time.Since(start).Nanoseconds()
			times[i] = float64(elapsed)
		}

		// Calculate statistics
		mean := calculateMean(times)
		stdDev := calculateStdDev(times, mean)

		// mean = avg total ns for `iterations` runs, so:
		perOpNs := mean / float64(iterations)
		opsPerSec := 1.0 / (perOpNs / 1e9) // == float64(iterations) / (mean/1e9)
		throughput := opsPerSec

		// stdDev is currently in "batch total ns" units (spread across whole
		// batches of `iterations` runs); normalize it too if you want it
		// comparable to Rust's per-op std_ns column:
		stdDevPerOp := stdDev / float64(iterations)

		fmt.Fprintf(file, "%d,%.2f,%.2f,%.3f\n", iterations, mean, stdDevPerOp, throughput)
	}
}

// gonja (jinja go implementation)
func gonjaBenchmark(csvPath string) {
	file, err := os.OpenFile(csvPath, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0666)
	if err != nil {
		log.Fatalf("Failed to open CSV file: %v\n", err)
	}
	defer file.Close()

	jsonPath := "../templates/meld.json"
	templatePath := "../templates/gonja.html"

	// Read JSON payload
	jsonData, err := os.ReadFile(jsonPath)
	if err != nil {
		log.Fatalf("Failed to read JSON file: %v\n", err)
	}

	var payload map[string]any
	if err := json.Unmarshal(jsonData, &payload); err != nil {
		log.Fatalf("Failed to unmarshal JSON: %v\n", err)
	}

	// Parse the Gonja template
	template, err := gonja.FromFile(templatePath)
	if err != nil {
		log.Fatalf("Failed to read template file: %v\n", err)
	}

	// Print header
	fmt.Println("Gonja Template Engine Benchmark")
	fmt.Println("================================")
	fmt.Printf("Template: %s\n", templatePath)
	fmt.Printf("Payload: %s\n", jsonPath)
	fmt.Printf("Sample Size: %d\n\n", SAMPLE_SIZE)
	fmt.Fprintln(file, "n,duration_ns,std_ns,throughput")

	// Run benchmarks
	for _, iterations := range BENCH_ITERATIONS {
		times := make([]float64, SAMPLE_SIZE)

		// Run SAMPLE_SIZE iterations
		for i := range SAMPLE_SIZE {
			start := time.Now()

			// Run `iterations` executions inside the timed region —
			for range iterations {
				output := bytes.Buffer{}
				if err = template.Execute(&output, exec.NewContext(payload)); err != nil { // Prints: Hello Bob!
					log.Fatalf("Failed to execute template: %v\n", err)
				}

				_ = output.String()
			}

			elapsed := time.Since(start).Nanoseconds()
			times[i] = float64(elapsed)
		}

		// Calculate statistics
		mean := calculateMean(times)
		stdDev := calculateStdDev(times, mean)

		// mean = avg total ns for `iterations` runs, so:
		perOpNs := mean / float64(iterations)
		opsPerSec := 1.0 / (perOpNs / 1e9) // == float64(iterations) / (mean/1e9)
		throughput := opsPerSec

		// stdDev is currently in "batch total ns" units (spread across whole
		// batches of `iterations` runs); normalize it too if you want it
		// comparable to Rust's per-op std_ns column:
		stdDevPerOp := stdDev / float64(iterations)

		fmt.Fprintf(file, "%d,%.2f,%.2f,%.3f\n", iterations, mean, stdDevPerOp, throughput)
	}
}

// meld
func meldBenchmark(csvPath string) {
	file, err := os.OpenFile(csvPath, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0666)
	if err != nil {
		log.Fatalf("Failed to open CSV file: %v\n", err)
	}
	defer file.Close()

	jsonPath := "../templates/meld.json"
	templatePath := "../templates/meld.bhtml"

	// Bytecode Template
	content, err := os.ReadFile(templatePath)
	if err != nil {
		log.Fatalf("Failed to read template file: %v\n", err)
	}

	// Read JSON payload
	jsonData, err := os.ReadFile(jsonPath)
	if err != nil {
		log.Fatalf("Failed to read JSON file: %v\n", err)
	}

	// Parse JSON
	var payload interface{}
	if err := json.Unmarshal(jsonData, &payload); err != nil {
		log.Fatalf("Failed to parse JSON: %v\n", err)
	}

	// Program
	program, err := meldruntime.NewProgram(content)
	if err != nil {
		log.Fatalf("Failed to compile template: %v\n", err)
	}

	// Runtime
	rt := meldruntime.NewRuntime(program, meldruntime.RuntimeConfig{
		IgnoreMissingVariables: true,
	})

	// Print header
	fmt.Println("Meld Template Engine Benchmark")
	fmt.Println("================================")
	fmt.Printf("Template: %s\n", templatePath)
	fmt.Printf("Payload: %s\n", jsonPath)
	fmt.Printf("Sample Size: %d\n\n", SAMPLE_SIZE)
	fmt.Fprintln(file, "n,duration_ns,std_ns,throughput")

	// Run benchmarks
	for _, iterations := range BENCH_ITERATIONS {
		times := make([]float64, SAMPLE_SIZE)

		for i := range SAMPLE_SIZE {

			start := time.Now()

			// Run `iterations` executions inside the timed region —
			for range iterations {
				err := rt.Run(payload)
				if err != nil {
					log.Fatalf("Runtime execution failed: %v\n", err)
				}

				_ = rt.Output()
			}

			elapsed := time.Since(start).Nanoseconds()
			times[i] = float64(elapsed) // total ns for `iterations` runs
		}

		// times[i] is now a BATCH time (iterations runs), not a single-op time
		mean := calculateMean(times)
		stdDev := calculateStdDev(times, mean)

		// mean = avg total ns for `iterations` runs, so:
		perOpNs := mean / float64(iterations)
		opsPerSec := 1.0 / (perOpNs / 1e9) // == float64(iterations) / (mean/1e9)
		throughput := opsPerSec

		// stdDev is currently in "batch total ns" units (spread across whole
		// batches of `iterations` runs); normalize it too if you want it
		// comparable to Rust's per-op std_ns column:
		stdDevPerOp := stdDev / float64(iterations)

		fmt.Fprintf(file, "%d,%.2f,%.2f,%.3f\n", iterations, mean, stdDevPerOp, throughput)
	}
}

// calculateMean calculates the arithmetic mean of a slice of floats
func calculateMean(values []float64) float64 {
	if len(values) == 0 {
		return 0
	}
	sum := 0.0
	for _, v := range values {
		sum += v
	}
	return sum / float64(len(values))
}

// calculateStdDev calculates the standard deviation of a slice of floats
func calculateStdDev(values []float64, mean float64) float64 {
	if len(values) == 0 {
		return 0
	}
	sumSquaredDiff := 0.0
	for _, v := range values {
		diff := v - mean
		sumSquaredDiff += diff * diff
	}
	variance := sumSquaredDiff / float64(len(values))
	return math.Sqrt(variance)
}

// handlebars
// func handlebarsBenchmark() {
// 	jsonPath := "../templates/meld.json"
// 	templatePath := "../templates/handlebars.html"

// 	// Read JSON payload
// 	jsonData, err := os.ReadFile(jsonPath)
// 	if err != nil {
// 		log.Fatalf("Failed to read JSON file: %v\n", err)
// 	}

// 	var payload map[string]any
// 	if err := json.Unmarshal(jsonData, &payload); err != nil {
// 		log.Fatalf("Failed to unmarshal JSON: %v\n", err)
// 	}

// 	// Compile the template
// 	template, err := raymond.ParseFile(templatePath)
// 	if err != nil {
// 		log.Fatalf("Failed to parse template: %v\n", err)
// 	}

// 	// Register helpers
// 	raymond.RegisterHelper("gt", helperGt)
// 	raymond.RegisterHelper("gte", helperGte)
// 	raymond.RegisterHelper("and", helperAnd)
// 	raymond.RegisterHelper("or", helperOr)
// 	raymond.RegisterHelper("concat", helperConcat)
// 	raymond.RegisterHelper("length", helperLength)
// 	raymond.RegisterHelper("coalesce", helperCoalesce)

// 	output, err := template.Exec(payload)
// 	if err != nil {
// 		log.Fatalf("Failed to execute template: %v\n", err)
// 	}

// 	err = os.WriteFile("../templates/handlebars.out.html", []byte(output), 0666)
// 	if err != nil {
// 		log.Fatalf("Failed to write output file: %v\n", err)
// 	}
// }

// // Helper functions for Handlebars
// func helperGt(left, right any) bool {
// 	leftNum := toFloat64(left)
// 	rightNum := toFloat64(right)
// 	return leftNum > rightNum
// }

// func helperGte(left, right any) bool {
// 	leftNum := toFloat64(left)
// 	rightNum := toFloat64(right)
// 	return leftNum >= rightNum
// }

// func helperAnd(left, right any) bool {
// 	return toBool(left) && toBool(right)
// }

// func helperOr(left, right any) bool {
// 	return toBool(left) || toBool(right)
// }

// func helperConcat(options *raymond.Options) string {
// 	var result string
// 	for _, param := range options.Params() {
// 		result += toString(param)
// 	}
// 	return result
// }

// func helperLength(val any) int {
// 	switch v := val.(type) {
// 	case []any:
// 		return len(v)
// 	case string:
// 		return len(v)
// 	case map[string]any:
// 		return len(v)
// 	default:
// 		return 0
// 	}
// }

// func helperCoalesce(val, fallback any) any {
// 	if val == nil {
// 		return fallback
// 	}
// 	if str, ok := val.(string); ok && str == "" {
// 		return fallback
// 	}
// 	return val
// }

// // Helper conversion functions
// func toFloat64(val any) float64 {
// 	switch v := val.(type) {
// 	case float64:
// 		return v
// 	case float32:
// 		return float64(v)
// 	case int:
// 		return float64(v)
// 	case int64:
// 		return float64(v)
// 	case string:
// 		var f float64
// 		fmt.Sscanf(v, "%f", &f)
// 		return f
// 	default:
// 		return 0
// 	}
// }

// func toBool(val any) bool {
// 	switch v := val.(type) {
// 	case bool:
// 		return v
// 	case float64:
// 		return v != 0
// 	case int:
// 		return v != 0
// 	case string:
// 		return v != ""
// 	case []any:
// 		return len(v) > 0
// 	case map[string]any:
// 		return len(v) > 0
// 	case nil:
// 		return false
// 	default:
// 		return true
// 	}
// }

// func toString(val any) string {
// 	switch v := val.(type) {
// 	case string:
// 		return v
// 	case float64:
// 		return fmt.Sprintf("%g", v)
// 	case bool:
// 		if v {
// 			return "true"
// 		}
// 		return "false"
// 	case nil:
// 		return ""
// 	default:
// 		return fmt.Sprintf("%v", v)
// 	}
// }
