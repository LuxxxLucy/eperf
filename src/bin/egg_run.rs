// Equality Saturation with 'egg'
// ===========================================
//
// Author: Jialin Lu
// Contact: luxxxlucy@gmail.com
//
// Implements equality saturation using 'egg'.
// The essence here is to define rewrite rules and apply them to an initial
// expression, optimizing it without changing its meaning.
//
// I used a personal fork of the fork of the 'egg' library.
// The fork includes additional tracing for debugging and analysis,
// without messing with the core logic and should be found with a soft link under the repo
// directory.
//
// Usage:
// Run the program would save the traces in a file.
// ```
//      egg_run > tmp.data
// ```
// It applies predefined rules to the expression and optimizes it.
// Check the output and trace logs if you've enabled them in the fork.

use epost::prelude::*;

use egg::{rewrite as rw, *};

fn main() -> Result<()> {
    // Define rewrite rules
    let rules: &[Rewrite<SymbolLang, ()>] = &[
        rw!("cancel-multiply"; "(/ (* ?a ?b) ?b)" => "?a"),
        rw!("commute-mul"; "(* ?x ?y)" => "(* ?y ?x)"),
        rw!("multi-shift"; "(* ?x 2)" => "(<< ?x 1)"),
        // rw!("multi-div-assoc"; "(/ (* ?x ?y) ?z)" => "(* ?x (/ ?y ?z))"),

        // rw!("cancel-denominator"; "(* (/ ?a ?b) ?b)" => "?a"),
        // rw!("commute-add"; "(+ ?x ?y)" => "(+ ?y ?x)"),
        // rw!("add-0"; "(+ ?x 0)" => "?x"),
        // rw!("mul-0"; "(* ?x 0)" => "0"),
        // rw!("mul-1"; "(* ?x 1)" => "?x"),
    ];

    let start = "(/ (* a 2) 2)"
        .parse()
        .map_err(|_| Error::Generic("egg parse init expression failed.".to_string()))?;

    // Run equality saturation
    let runner = Runner::default().with_expr(&start).run(rules);

    // Use AstSize as the cost function for extraction
    let extractor = Extractor::new(&runner.egraph, AstSize);

    // Extract the best expression from the initial e-class
    let (best_cost, best_expr) = extractor.find_best(runner.roots[0]);

    // Assertions to validate the results
    assert_eq!(
        best_expr,
        "a".parse().unwrap(),
        "Optimized expr should be 'a'"
    );
    assert_eq!(best_cost, 1, "Cost should be 1");

    // Indicate success
    println!("Success: Best expression found is '{}'", best_expr);

    Ok(())
}
