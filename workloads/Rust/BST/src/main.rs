use crate::implementation::Tree;

pub mod implementation;
pub mod spec;
pub mod strategies;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Available tools: quickcheck");
        eprintln!(
            "For available properties, check https://github.com/alpaylan/etna-cli/blob/main/docs/workloads/bst.md"
        );
        return;
    }
    let tool = args[1].as_str();
    let property = args[2].as_str();

    let num_tests = 200_000;
    let mut qc = quickcheck::QuickCheck::new()
        .tests(num_tests)
        .max_tests(num_tests * 2);

    let result = match (tool, property) {
        ("quickcheck", "insert_valid") => {
            qc.quicktest(spec::prop_insert_valid as fn(Tree, i32, i32) -> bool)
        }
        ("quickcheck", "delete_valid") => {
            qc.quicktest(spec::prop_delete_valid as fn(Tree, i32) -> bool)
        }
        ("quickcheck", "union_valid") => {
            qc.quicktest(spec::prop_union_valid as fn(Tree, Tree) -> bool)
        }
        ("quickcheck", "insert_post") => {
            qc.quicktest(spec::prop_insert_post as fn(Tree, i32, i32, i32) -> bool)
        }
        ("quickcheck", "delete_post") => {
            qc.quicktest(spec::prop_delete_post as fn(Tree, i32, i32) -> bool)
        }
        ("quickcheck", "union_post") => {
            qc.quicktest(spec::prop_union_post as fn(Tree, Tree, i32) -> bool)
        }
        ("quickcheck", "insert_model") => {
            qc.quicktest(spec::prop_insert_model as fn(Tree, i32, i32) -> bool)
        }
        ("quickcheck", "delete_model") => {
            qc.quicktest(spec::prop_delete_model as fn(Tree, i32) -> bool)
        }
        ("quickcheck", "union_model") => {
            qc.quicktest(spec::prop_union_model as fn(Tree, Tree) -> bool)
        }
        _ => {
            panic!("Unknown tool or property: {} {}", tool, property)
        }
    };


    match result {
        Ok(n) => println!("{} tests passed successfully", n),
        Err(e) => eprintln!("Test failed: {}\nExecution times: {:?}", e.args(), e.execution_time),
    }
}
