# FormCalc - Formula Calculator Engine

A Rust implementation of the Formula Calculator Engine, providing a powerful formula evaluation system with dependency management.

## Features

- **Formula Parsing**: Parse and evaluate complex formulas with arithmetic, logical, and comparison operations
- **Dependency Management**: Automatically resolve and execute formulas in the correct order based on dependencies
- **Parallel Execution**: Formulas in the same dependency layer are executed in parallel for maximum performance
- **Built-in Functions**: Support for mathematical, string, and date functions
- **Custom Functions**: Register custom functions to extend functionality
- **Variables**: Support for variables in formulas
- **Type System**: Strong typing with support for numbers, strings, and booleans
- **Error Handling**: Comprehensive error reporting with detailed messages

## Formula Syntax

### Basic Expressions

```
return 2 + 2              // Arithmetic
return 'Hello' + ' World' // String concatenation
return 5 > 3              // Comparison
return true and false     // Logical operations
```

### Conditional Statements

```
if (x > 10) then
    return 'High'
else if (x > 5) then
    return 'Medium'
else
    return 'Low'
end
```

### Built-in Functions

#### Mathematical Functions
- `max(a, b)` - Maximum of two numbers
- `min(a, b)` - Minimum of two numbers
- `rnd(value, decimals)` - Round to specified decimal places
- `ceil(value)` - Round up to nearest integer
- `floor(value)` - Round down to nearest integer
- `exp(value)` - Exponential function
- `mod` - Modulo operator

#### Date Functions
- `year(date)` - Extract year from date string
- `month(date)` - Extract month from date string
- `day(date)` - Extract day from date string
- `add_days(date, days)` - Add days to a date
- `get_diff_days(date1, date2)` - Get difference between dates in days
- `get_diff_months(date1, date2)` - Get difference in months

#### String Functions
- `substr(string, start, length)` - Extract substring
- `padded_string(string, width)` - Pad string with zeros

#### Formula Functions
- `get_output_from('formula_name')` - Get result from another formula

## Usage Examples

### Basic Calculation

```rust
use formcalc::{Engine, Formula, Value};

let mut engine = Engine::new();
let formula = Formula::new("calculation", "return 2 + 2 * 3");

engine.execute(vec![formula]).unwrap();

let result = engine.get_result("calculation").unwrap();
assert_eq!(result, Value::Number(8.0));
```

### Using Variables

```rust
use formcalc::{Engine, Formula, Value};

let mut engine = Engine::new();
engine.set_variable("price".to_string(), Value::Number(100.0));
engine.set_variable("tax_rate".to_string(), Value::Number(0.2));

let formula = Formula::new("total", "return price * (1 + tax_rate)");
engine.execute(vec![formula]).unwrap();

let result = engine.get_result("total").unwrap();
assert_eq!(result, Value::Number(120.0));
```

### Formula Dependencies

```rust
use formcalc::{Engine, Formula, Value};

let mut engine = Engine::new();

let formula1 = Formula::new("base_price", "return 100");
let formula2 = Formula::new("with_tax", "return get_output_from('base_price') * 1.2");
let formula3 = Formula::new("final_price", "return get_output_from('with_tax') + 10");

// The engine automatically resolves dependencies and executes in correct order
engine.execute(vec![formula1, formula2, formula3]).unwrap();

let result = engine.get_result("final_price").unwrap();
assert_eq!(result, Value::Number(130.0));
```

### Custom Functions

```rust
use formcalc::{Engine, Formula, Function, Value, Result, CalculatorError};
use std::sync::Arc;

// Define a custom function
struct DoubleFunction;

impl Function for DoubleFunction {
    fn name(&self) -> &str {
        "double"
    }

    fn num_args(&self) -> usize {
        1
    }

    fn execute(&self, params: &[Value]) -> Result<Value> {
        match params[0] {
            Value::Number(n) => Ok(Value::Number(n * 2.0)),
            _ => Err(CalculatorError::TypeError("Expected number".to_string())),
        }
    }
}

let mut engine = Engine::new();
engine.register_function(Arc::new(DoubleFunction));

let formula = Formula::new("test", "return double(21)");
engine.execute(vec![formula]).unwrap();

let result = engine.get_result("test").unwrap();
assert_eq!(result, Value::Number(42.0));
```

### Conditional Logic

```rust
use formcalc::{Engine, Formula, Value};

let mut engine = Engine::new();
engine.set_variable("score".to_string(), Value::Number(85.0));

let formula = Formula::new("grade", r#"
    if (score >= 90) then
        return 'A'
    else if (score >= 80) then
        return 'B'
    else if (score >= 70) then
        return 'C'
    else
        return 'F'
    end
"#);

engine.execute(vec![formula]).unwrap();

let result = engine.get_result("grade").unwrap();
assert_eq!(result, Value::String("B".to_string()));
```

## Supported Operators

### Arithmetic
- `+` - Addition (also string concatenation)
- `-` - Subtraction
- `*` - Multiplication
- `/` - Division
- `^` - Power
- `mod` - Modulo

### Comparison
- `=` - Equal
- `<>` - Not equal
- `<` - Less than
- `>` - Greater than
- `<=` - Less than or equal
- `>=` - Greater than or equal

### Logical
- `and` - Logical AND
- `or` - Logical OR
- `!` - Logical NOT

## Error Handling

```rust
use formcalc::{Engine, Formula};

let mut engine = Engine::new();
let formula = Formula::new("error_test", "return 1 / 0");

engine.execute(vec![formula]).unwrap();

// Check for errors
if let Some(error) = engine.get_errors().get("error_test") {
    println!("Error: {}", error);
}
```

## Architecture

The engine follows the architecture:

1. **Lexer** - Tokenizes the input formula
2. **Parser** - Builds an Abstract Syntax Tree (AST)
3. **Evaluator** - Evaluates the AST using the visitor pattern
4. **DAG** - Manages formula dependencies using a directed acyclic graph
5. **Engine** - Orchestrates parsing, dependency resolution, and execution

## Performance Considerations

- **Parallel Execution**: Formulas in the same dependency layer are executed in parallel using Rayon
- Results are cached to avoid re-computation
- Function results are cached per execution
- Layer-by-layer execution ensures dependencies are resolved correctly

## Contributing

We welcome contributions to FormCalc! Here's how you can help:

### Reporting Issues

- Use the GitHub issue tracker to report bugs
- Describe the issue clearly with steps to reproduce
- Include sample formulas and expected vs actual behavior

### Submitting Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with clear, descriptive commit messages
4. Add tests for any new functionality
5. Ensure all tests pass (`cargo test`)
6. Run the formatter (`cargo fmt`)
7. Run the linter (`cargo clippy`)
8. Submit a pull request

### Development Guidelines

- Follow Rust naming conventions and idioms
- Write unit tests for new features
- Update documentation for API changes
- Keep commits focused and atomic

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

