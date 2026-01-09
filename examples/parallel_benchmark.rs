use formcalc::{Engine, Formula};
use std::time::Instant;

fn main() {
    println!("=== Parallel Execution Benchmark ===\n");

    // Test 1: Many independent formulas (best case for parallelism)
    benchmark_independent_formulas();

    // Test 2: Layered dependencies
    benchmark_layered_dependencies();

    // Test 3: Complex formulas
    benchmark_complex_formulas();
}

fn benchmark_independent_formulas() {
    println!("Test 1: Independent Formulas");
    println!("-----------------------------");

    let mut engine = Engine::new();

    // Create 100 independent formulas
    let formulas: Vec<Formula> = (0..100)
        .map(|i| {
            Formula::new(
                format!("formula_{}", i),
                format!("return {} + {} * {} - {}", i, i + 1, i + 2, i + 3),
            )
        })
        .collect();

    let start = Instant::now();
    engine.execute(formulas).unwrap();
    let duration = start.elapsed();

    println!("Executed 100 independent formulas in {:?}", duration);
    println!("All formulas executed in parallel (single layer)\n");
}

fn benchmark_layered_dependencies() {
    println!("Test 2: Layered Dependencies");
    println!("-----------------------------");

    let mut engine = Engine::new();

    // Create a dependency tree:
    // Layer 0: 20 base formulas
    // Layer 1: 20 formulas depending on layer 0
    // Layer 2: 20 formulas depending on layer 1
    let mut formulas = Vec::new();

    // Layer 0: base formulas
    for i in 0..20 {
        formulas.push(Formula::new(
            format!("base_{}", i),
            format!("return {}", i * 10),
        ));
    }

    // Layer 1: depends on layer 0
    for i in 0..20 {
        let dep_idx = i % 20;
        formulas.push(Formula::new(
            format!("layer1_{}", i),
            format!("return get_output_from('base_{}') + 5", dep_idx),
        ));
    }

    // Layer 2: depends on layer 1
    for i in 0..20 {
        let dep_idx = i % 20;
        formulas.push(Formula::new(
            format!("layer2_{}", i),
            format!("return get_output_from('layer1_{}') * 2", dep_idx),
        ));
    }

    let start = Instant::now();
    engine.execute(formulas).unwrap();
    let duration = start.elapsed();

    println!("Executed 60 formulas in 3 layers in {:?}", duration);
    println!("20 formulas per layer executed in parallel\n");
}

fn benchmark_complex_formulas() {
    println!("Test 3: Complex Formulas");
    println!("------------------------");

    let mut engine = Engine::new();

    // Create formulas with more complex calculations
    let formulas: Vec<Formula> = (0..50)
        .map(|i| {
            Formula::new(
                format!("complex_{}", i),
                format!(
                    "if ({} > 25) then return max({}, {}) * rnd(3.14159, 2) else return min({}, {}) + floor(2.7) end",
                    i, i, i + 1, i, i + 1
                ),
            )
        })
        .collect();

    let start = Instant::now();
    engine.execute(formulas).unwrap();
    let duration = start.elapsed();

    println!("Executed 50 complex formulas in {:?}", duration);
    println!("All formulas executed in parallel with conditional logic\n");
}
