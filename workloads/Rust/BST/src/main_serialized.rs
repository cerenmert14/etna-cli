use crate::implementation::Tree;

pub mod implementation;
pub mod spec;
pub mod strategies;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        eprintln!("Usage: {} <tests> <property>", args[0]);
        eprintln!("Tests should be an s-expression that is a list of test cases.");
        eprintln!(
            "For available properties, check https://github.com/alpaylan/etna-cli/blob/main/docs/workloads/bst.md"
        );
        return;
    }
    let tests = args[1].as_str();
    let property = args[2].as_str();

    let num_tests = 200_000;
    let mut qc = quickcheck::QuickCheck::new()
        .tests(num_tests)
        .max_tests(num_tests * 2);

    match property {
        "insert_valid" => {
            let tests: Vec<(Tree, i32, i32)> = serde_sexpr::from_str(tests).unwrap_or_else(|_| {
                eprintln!("Failed to parse tests: '{}'", tests);
                return vec![];
            });

            for (i, (t, k, v)) in tests.into_iter().enumerate() {
                if !spec::prop_insert_valid(t.clone(), k, v) {
                    eprintln!(
                        "Test {} failed for insert_valid: ({}, {}, {})",
                        i,
                        serde_sexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k,
                        v
                    );
                }
            }
        }
        "delete_valid" => qc.quickcheck(spec::prop_delete_valid as fn(Tree, i32) -> bool),
        "union_valid" => qc.quickcheck(spec::prop_union_valid as fn(Tree, Tree) -> bool),
        "insert_post" => qc.quickcheck(spec::prop_insert_post as fn(Tree, i32, i32, i32) -> bool),
        "delete_post" => qc.quickcheck(spec::prop_delete_post as fn(Tree, i32, i32) -> bool),
        "union_post" => qc.quickcheck(spec::prop_union_post as fn(Tree, Tree, i32) -> bool),
        "insert_model" => qc.quickcheck(spec::prop_insert_model as fn(Tree, i32, i32) -> bool),
        "delete_model" => qc.quickcheck(spec::prop_delete_model as fn(Tree, i32) -> bool),
        "union_model" => qc.quickcheck(spec::prop_union_model as fn(Tree, Tree) -> bool),
        _ => {
            eprintln!("Unknown property: {}", property);
        }
    }
}
