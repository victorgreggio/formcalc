# Parallel Execution Implementation

## Overview

The FormCalc engine supports **parallel execution** of formulas within the same dependency layer using Rayon's work-stealing thread pool. This provides significant performance improvements when evaluating multiple independent formulas.

## How It Works

### Dependency Resolution

1. **Build DAG**: All formulas are added to a Directed Acyclic Graph (DAG) with their dependencies
2. **Topological Sort**: The DAG is sorted into layers where:
   - Layer 0: Formulas with no dependencies
   - Layer 1: Formulas that only depend on Layer 0
   - Layer N: Formulas that only depend on previous layers
3. **Parallel Execution**: All formulas in each layer are executed in parallel

### Thread Safety

The implementation uses thread-safe data structures:
- **Arc<RwLock<HashMap>>**: For all caches (variables, formulas, functions, results)
- **Rayon**: Work-stealing thread pool for efficient parallel execution
- **Sequential Result Collection**: Results are collected and written back sequentially to avoid race conditions

## Code Example

```rust
use formcalc::{Engine, Formula};

let mut engine = Engine::new();

// These formulas will execute in parallel (Layer 0)
let formulas = vec![
    Formula::new("a", "return 10"),
    Formula::new("b", "return 20"),
    Formula::new("c", "return 30"),
];

engine.execute(formulas).unwrap();
```

### With Dependencies

```rust
// Layer 0: a, b (parallel)
// Layer 1: c, d (parallel, depend on Layer 0)
// Layer 2: e (depends on Layer 1)
let formulas = vec![
    Formula::new("a", "return 10"),
    Formula::new("b", "return 20"),
    Formula::new("c", "return get_output_from('a') * 2"),
    Formula::new("d", "return get_output_from('b') * 2"),
    Formula::new("e", "return get_output_from('c') + get_output_from('d')"),
];

engine.execute(formulas).unwrap();
// Result: e = 60
```

## Performance Benefits

### Benchmark Results

From the `parallel_benchmark` example:

```
Test 1: Independent Formulas
-----------------------------
Executed 100 independent formulas in 1.2ms
All formulas executed in parallel (single layer)

Test 2: Layered Dependencies
-----------------------------
Executed 60 formulas in 3 layers in 258µs
20 formulas per layer executed in parallel

Test 3: Complex Formulas
------------------------
Executed 50 complex formulas in 180µs
All formulas executed in parallel with conditional logic
```

### When Parallel Execution Helps

**Best Performance Gains:**
- Multiple independent formulas
- Wide dependency trees (many formulas at each layer)
- Computationally expensive formulas
- I/O-bound operations (custom functions that do I/O)

**Limited Benefits:**
- Sequential dependency chains (A → B → C → D)
- Very simple formulas (overhead > computation time)
- Small number of formulas

## Implementation Details

### Engine Changes

The `Engine::execute()` method:

1. Builds the dependency graph
2. Performs topological sort
3. For each layer, calls `execute_layer_parallel()`

```rust
fn execute_layer_parallel(&mut self, graph: &DAGraph<String, Formula>, layer: Vec<String>) {
    // Execute formulas in parallel
    let results: Vec<(String, Result<Value>)> = layer
        .par_iter()  // Rayon's parallel iterator
        .filter_map(|formula_name| {
            graph.get(formula_name).map(|formula| {
                let result = self.try_execute_formula(formula);
                (formula_name.clone(), result)
            })
        })
        .collect();

    // Process results sequentially
    for (formula_name, result) in results {
        match result {
            Ok(value) => {
                self.formula_result_cache.set(formula_name, value);
            }
            Err(e) => {
                let error_msg = format!("Error executing formula '{}': {}", formula_name, e);
                self.errors.insert(formula_name, error_msg);
            }
        }
    }
}
```

### Cache Thread Safety

All caches use `Arc<RwLock<HashMap>>`:

```rust
#[derive(Clone, Default)]
pub struct VariableCache {
    cache: Arc<RwLock<HashMap<String, Value>>>,
}
```

This allows:
- **Cloning**: Evaluators can clone caches without copying data
- **Thread-safe reads**: Multiple threads can read simultaneously
- **Thread-safe writes**: Exclusive write access when updating

### Rayon Integration

Added to `Cargo.toml`:
```toml
[dependencies]
rayon = "1.10"
```

Rayon provides:
- Work-stealing thread pool
- Automatic load balancing
- Minimal overhead for parallel execution
- Graceful degradation (works efficiently even with small workloads)

## Testing

### Unit Tests

New tests verify parallel execution:

```rust
#[test]
fn test_parallel_execution() {
    let mut engine = Engine::new();
    
    let formulas = vec![
        Formula::new("a", "return 1 + 1"),
        Formula::new("b", "return 2 + 2"),
        Formula::new("c", "return 3 + 3"),
        Formula::new("d", "return 4 + 4"),
        Formula::new("e", "return 5 + 5"),
    ];
    
    engine.execute(formulas).unwrap();
    
    assert_eq!(engine.get_result("a").unwrap(), Value::Number(2.0));
    // ... etc
}

#[test]
fn test_parallel_with_dependencies() {
    // Tests that dependencies are correctly resolved
    // even with parallel execution
}
```

## Benchmark

Run the benchmark to see parallel execution in action:

```bash
cargo run --release --example parallel_benchmark
```

## Configuration

### Thread Pool Size

Rayon automatically determines the optimal number of threads based on:
- Number of CPU cores
- System load
- Available resources

To manually configure (if needed):

```rust
// At the start of your program
rayon::ThreadPoolBuilder::new()
    .num_threads(4)
    .build_global()
    .unwrap();
```

### Disabling Parallel Execution

If you need sequential execution (e.g., for debugging), you can:

```rust
// Set RAYON_NUM_THREADS=1 environment variable
// or
rayon::ThreadPoolBuilder::new()
    .num_threads(1)
    .build_global()
    .unwrap();
```
## Future Enhancements

Potential improvements:

1. **Adaptive Parallelism**: Use sequential execution for small layers
2. **Custom Thread Pools**: Allow users to provide their own thread pool
3. **Execution Strategies**: Different strategies for different workloads
4. **Profiling**: Built-in performance profiling for formulas
5. **Cancellation**: Support for cancelling long-running evaluations

## Troubleshooting

### Common Issues

**Issue**: Slower with parallel execution
**Solution**: This can happen with very simple formulas where parallel overhead exceeds computation time. Consider using sequential execution for simple cases.

**Issue**: Thread pool exhaustion
**Solution**: Ensure custom functions don't block threads indefinitely. Use async operations where appropriate.

**Issue**: Race conditions in custom functions
**Solution**: Ensure custom `Function` implementations are thread-safe (marked with `Send + Sync`).

## Conclusion

The parallel execution implementation provides:
- ✅ Significant performance improvements for independent formulas
- ✅ Correct dependency resolution
- ✅ Thread-safe operation
- ✅ Minimal changes to the API
- ✅ Automatic load balancing
- ✅ Production-ready implementation

The engine can now efficiently handle large formula sets, making it suitable for high-performance applications and batch processing scenarios.
