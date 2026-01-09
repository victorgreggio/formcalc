use formcalc::{Engine, Formula, Value};

fn main() {
    println!("=== FormCalc Examples ===\n");

    // Example 1: Simple Calculation
    example_1_simple_calculation();

    // Example 2: Variables
    example_2_variables();

    // Example 3: Conditional Logic
    example_3_conditional();

    // Example 4: Formula Dependencies
    example_4_dependencies();

    // Example 5: Built-in Functions
    example_5_builtin_functions();

    // Example 6: String Operations
    example_6_strings();
}

fn example_1_simple_calculation() {
    println!("Example 1: Simple Calculation");
    println!("------------------------------");

    let mut engine = Engine::new();
    let formula = Formula::new("calc", "return (5 + 3) * 2 - 1");

    engine.execute(vec![formula]).unwrap();

    if let Some(result) = engine.get_result("calc") {
        println!("Formula: (5 + 3) * 2 - 1");
        println!("Result: {}", result);
    }
    println!();
}

fn example_2_variables() {
    println!("Example 2: Using Variables");
    println!("--------------------------");

    let mut engine = Engine::new();

    // Set variables
    engine.set_variable("price".to_string(), Value::Number(100.0));
    engine.set_variable("quantity".to_string(), Value::Number(5.0));
    engine.set_variable("tax_rate".to_string(), Value::Number(0.08));

    let formula = Formula::new("total", "return price * quantity * (1 + tax_rate)");

    engine.execute(vec![formula]).unwrap();

    if let Some(result) = engine.get_result("total") {
        println!("Price: $100, Quantity: 5, Tax: 8%");
        println!("Total: ${}", result);
    }
    println!();
}

fn example_3_conditional() {
    println!("Example 3: Conditional Logic");
    println!("----------------------------");

    let mut engine = Engine::new();

    // Test different scores
    let scores = vec![95.0, 85.0, 75.0, 65.0];

    for score in scores {
        engine.clear();
        engine.set_variable("score".to_string(), Value::Number(score));

        let formula = Formula::new(
            "grade",
            r#"
            if (score >= 90) then
                return 'A'
            else if (score >= 80) then
                return 'B'
            else if (score >= 70) then
                return 'C'
            else if (score >= 60) then
                return 'D'
            else
                return 'F'
            end
        "#,
        );

        engine.execute(vec![formula]).unwrap();

        if let Some(result) = engine.get_result("grade") {
            println!("Score: {} => Grade: {}", score, result);
        }
    }
    println!();
}

fn example_4_dependencies() {
    println!("Example 4: Formula Dependencies");
    println!("-------------------------------");

    let mut engine = Engine::new();

    // Create formulas with dependencies
    let base = Formula::new("base_amount", "return 1000");
    let discount = Formula::new(
        "discount_amount",
        "return get_output_from('base_amount') * 0.1",
    );
    let subtotal = Formula::new(
        "subtotal",
        "return get_output_from('base_amount') - get_output_from('discount_amount')",
    );
    let tax = Formula::new("tax", "return get_output_from('subtotal') * 0.08");
    let total = Formula::new(
        "grand_total",
        "return get_output_from('subtotal') + get_output_from('tax')",
    );

    // Engine automatically resolves dependencies
    engine
        .execute(vec![base, discount, subtotal, tax, total])
        .unwrap();

    println!("Base Amount: {}", engine.get_result("base_amount").unwrap());
    println!(
        "Discount:    {}",
        engine.get_result("discount_amount").unwrap()
    );
    println!("Subtotal:    {}", engine.get_result("subtotal").unwrap());
    println!("Tax:         {}", engine.get_result("tax").unwrap());
    println!("Grand Total: {}", engine.get_result("grand_total").unwrap());
    println!();
}

fn example_5_builtin_functions() {
    println!("Example 5: Built-in Functions");
    println!("-----------------------------");

    let mut engine = Engine::new();

    let formulas = vec![
        Formula::new("max_test", "return max(10, 25)"),
        Formula::new("min_test", "return min(10, 25)"),
        Formula::new("round_test", "return rnd(3.14159, 2)"),
        Formula::new("ceil_test", "return ceil(4.2)"),
        Formula::new("floor_test", "return floor(4.8)"),
        Formula::new("power_test", "return 2 ^ 8"),
    ];

    engine.execute(formulas).unwrap();

    println!(
        "max(10, 25)       = {}",
        engine.get_result("max_test").unwrap()
    );
    println!(
        "min(10, 25)       = {}",
        engine.get_result("min_test").unwrap()
    );
    println!(
        "rnd(3.14159, 2)   = {}",
        engine.get_result("round_test").unwrap()
    );
    println!(
        "ceil(4.2)         = {}",
        engine.get_result("ceil_test").unwrap()
    );
    println!(
        "floor(4.8)        = {}",
        engine.get_result("floor_test").unwrap()
    );
    println!(
        "2 ^ 8             = {}",
        engine.get_result("power_test").unwrap()
    );
    println!();
}

fn example_6_strings() {
    println!("Example 6: String Operations");
    println!("----------------------------");

    let mut engine = Engine::new();

    engine.set_variable("first_name".to_string(), Value::String("John".to_string()));
    engine.set_variable("last_name".to_string(), Value::String("Doe".to_string()));

    let formulas = vec![
        Formula::new("full_name", "return first_name + ' ' + last_name"),
        Formula::new("greeting", "return 'Hello, ' + first_name + '!'"),
        Formula::new("substring", "return substr('Hello World', 0, 5)"),
        Formula::new("padded", "return padded_string('42', 5)"),
    ];

    engine.execute(formulas).unwrap();

    println!("Full Name:   {}", engine.get_result("full_name").unwrap());
    println!("Greeting:    {}", engine.get_result("greeting").unwrap());
    println!("Substring:   {}", engine.get_result("substring").unwrap());
    println!("Padded:      {}", engine.get_result("padded").unwrap());
    println!();
}
