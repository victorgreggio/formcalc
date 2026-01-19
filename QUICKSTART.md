# Quick Start Guide

Get started with the FormCalc engine in 5 minutes.

## Key Features

- 🚀 **Parallel Execution**: Formulas execute in parallel within each dependency layer
- 📊 **Dependency Resolution**: Automatic topological sorting
- 🔧 **Extensible**: Add custom functions easily
- 🛡️ **Type Safe**: Strong typing with Rust's guarantees

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
formcalc = { path = "../formcalc" }
```

## Basic Usage

### 1. Simple Calculation

```rust
use formcalc::{Engine, Formula};

let mut engine = Engine::new();
let formula = Formula::new("result", "return 2 + 2");

engine.execute(vec![formula]).unwrap();

println!("{}", engine.get_result("result").unwrap()); // Prints: 4
```

### 2. Using Variables

```rust
use formcalc::{Engine, Formula, Value};

let mut engine = Engine::new();

// Set variables
engine.set_variable("x".to_string(), Value::Number(10.0));
engine.set_variable("y".to_string(), Value::Number(5.0));

// Create formula using variables
let formula = Formula::new("sum", "return x + y");

engine.execute(vec![formula]).unwrap();

println!("{}", engine.get_result("sum").unwrap()); // Prints: 15
```

### 3. Conditional Logic

```rust
use formcalc::{Engine, Formula, Value};

let mut engine = Engine::new();
engine.set_variable("age".to_string(), Value::Number(25.0));

let formula = Formula::new("status", r#"
    if (age >= 18) then
        return 'Adult'
    else
        return 'Minor'
    end
"#);

engine.execute(vec![formula]).unwrap();

println!("{}", engine.get_result("status").unwrap()); // Prints: Adult
```

### 4. Formula Dependencies

```rust
use formcalc::{Engine, Formula};

let mut engine = Engine::new();

// Formulas can depend on each other
let price = Formula::new("price", "return 100");
let tax = Formula::new("tax", "return get_output_from('price') * 0.1");
let total = Formula::new("total", "return get_output_from('price') + get_output_from('tax')");

// Engine automatically resolves dependencies
engine.execute(vec![price, tax, total]).unwrap();

println!("Total: {}", engine.get_result("total").unwrap()); // Prints: 110
```

### 5. Built-in Functions

```rust
use formcalc::{Engine, Formula};

let mut engine = Engine::new();

let formulas = vec![
    Formula::new("max_val", "return max(10, 20)"),
    Formula::new("rounded", "return rnd(3.14159, 2)"),
    Formula::new("power", "return 2 ^ 10"),
];

engine.execute(formulas).unwrap();

println!("Max: {}", engine.get_result("max_val").unwrap());    // Prints: 20
println!("Rounded: {}", engine.get_result("rounded").unwrap()); // Prints: 3.14
println!("Power: {}", engine.get_result("power").unwrap());    // Prints: 1024
```

### 6. String Operations

```rust
use formcalc::{Engine, Formula, Value};

let mut engine = Engine::new();
engine.set_variable("name".to_string(), Value::String("World".to_string()));

let formula = Formula::new("greeting", "return 'Hello, ' + name + '!'");

engine.execute(vec![formula]).unwrap();

println!("{}", engine.get_result("greeting").unwrap()); // Prints: Hello, World!
```

### 7. Custom Functions

```rust
use formcalc::{Engine, Formula, Function, Value, Result, CalculatorError};
use std::sync::Arc;

// Define a custom function
struct SquareFunction;

impl Function for SquareFunction {
    fn name(&self) -> &str { "square" }
    fn num_args(&self) -> usize { 1 }
    
    fn execute(&self, params: &[Value]) -> Result<Value> {
        match params[0] {
            Value::Number(n) => Ok(Value::Number(n * n)),
            _ => Err(CalculatorError::TypeError("Expected number".to_string())),
        }
    }
}

let mut engine = Engine::new();
engine.register_function(Arc::new(SquareFunction));

let formula = Formula::new("result", "return square(5)");
engine.execute(vec![formula]).unwrap();

println!("{}", engine.get_result("result").unwrap()); // Prints: 25
```

### 8. Parallel Execution

The engine automatically executes independent formulas in parallel:

```rust
use formcalc::{Engine, Formula};
use std::time::Instant;

let mut engine = Engine::new();

// Create 100 independent formulas
let formulas: Vec<Formula> = (0..100)
    .map(|i| Formula::new(
        format!("calc_{}", i),
        format!("return {} * 2 + 1", i)
    ))
    .collect();

let start = Instant::now();
engine.execute(formulas).unwrap();
let duration = start.elapsed();

println!("Executed 100 formulas in {:?}", duration);
// All formulas run in parallel!
```

## Common Patterns

### Error Handling

```rust
use formcalc::{Engine, Formula};

let mut engine = Engine::new();
let formula = Formula::new("bad", "return 1 / 0");

engine.execute(vec![formula]).unwrap();

// Check for errors
if let Some(error) = engine.get_errors().get("bad") {
    eprintln!("Error in formula: {}", error);
}
```

### Multiple Formulas with Dependencies

```rust
use formcalc::{Engine, Formula};

let mut engine = Engine::new();

let formulas = vec![
    Formula::new("a", "return 10"),
    Formula::new("b", "return 20"),
    Formula::new("c", "return get_output_from('a') + get_output_from('b')"),
];

engine.execute(formulas).unwrap();

for name in &["a", "b", "c"] {
    if let Some(result) = engine.get_result(name) {
        println!("{} = {}", name, result);
    }
}
```

### Reusing the Engine

```rust
use formcalc::{Engine, Formula, Value};

let mut engine = Engine::new();

// First calculation
engine.set_variable("x".to_string(), Value::Number(5.0));
let f1 = Formula::new("double", "return x * 2");
engine.execute(vec![f1]).unwrap();
println!("First: {}", engine.get_result("double").unwrap());

// Clear and reuse
engine.clear();

// Second calculation
engine.set_variable("x".to_string(), Value::Number(10.0));
let f2 = Formula::new("triple", "return x * 3");
engine.execute(vec![f2]).unwrap();
println!("Second: {}", engine.get_result("triple").unwrap());
```

## Supported Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+` `-` `*` `/` `^` `mod` |
| Comparison | `=` `<>` `<` `>` `<=` `>=` |
| Logical | `and` `or` `!` |

## Built-in Functions

| Function | Description | Example |
|----------|-------------|---------|
| `max(a, b)` | Maximum of two numbers | `max(10, 20)` → 20 |
| `min(a, b)` | Minimum of two numbers | `min(10, 20)` → 10 |
| `rnd(n, d)` | Round to d decimals | `rnd(3.14159, 2)` → 3.14 |
| `ceil(n)` | Round up | `ceil(4.2)` → 5 |
| `floor(n)` | Round down | `floor(4.8)` → 4 |
| `exp(n)` | Exponential | `exp(1)` → 2.718... |
| `substr(s, start, len)` | Substring | `substr('hello', 0, 3)` → 'hel' |
| `padded_string(s, w)` | Pad with zeros | `padded_string('42', 5)` → '00042' |
| `year(date)` | Extract year | `year('2024-01-15')` → 2024 |
| `month(date)` | Extract month | `month('2024-01-15')` → 1 |
| `day(date)` | Extract day | `day('2024-01-15')` → 15 |
| `add_days(date, n)` | Add days | `add_days('2024-01-15', 5)` |
| `get_diff_days(d1, d2)` | Days between | `get_diff_days('2024-01-20', '2024-01-15')` → 5 |
| `get_diff_months(d1, d2)` | Months between | - |
| `get_output_from('name')` | Get formula result | `get_output_from('price')` |

## Next Steps

- See [README.md](README.md) for detailed documentation
- Check [examples/basic_usage.rs](examples/basic_usage.rs) for more examples

## Running Examples

```bash
cargo run --example basic_usage
```

## Running Tests

```bash
cargo test
```

## Need Help?

- All functions are documented in source code
- Use `cargo doc --open` to view documentation
- Check the test files for usage patterns
