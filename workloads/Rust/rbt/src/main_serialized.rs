use std::path::Path;

use rbt::{implementation::Tree, spec};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        eprintln!("Usage: {} <tests> <property>", args[0]);
        eprintln!("Tests should be an s-expression that is a list of test cases.");
        eprintln!(
            "For available properties, check https://github.com/alpaylan/etna-cli/blob/main/docs/workloads/rbt.md"
        );
        return;
    }
    let tests = args[1].as_str();
    let property = args[2].as_str();

    let tests = if Path::new(tests).exists() {
        std::fs::read_to_string(tests).expect("Failed to read tests file")
    } else {
        tests.to_string()
    };

    match property {
        "InsertValid" => {
            let tests: Vec<(Tree, i32, i32)> = serde_lexpr::from_str(&tests).unwrap_or_else(|_| {
                eprintln!("Failed to parse tests: '{}'", tests);
                return vec![];
            });

            for (i, (t, k, v)) in tests.into_iter().enumerate() {
                if !spec::prop_insert_valid(t.clone(), k, v).unwrap_or(true) {
                    eprintln!(
                        "Test {} failed for InsertValid: ({}, {}, {})",
                        i,
                        serde_lexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k,
                        v
                    );
                }
            }
        }
        "DeleteValid" => {
            let tests: Vec<(Tree, i32)> = serde_lexpr::from_str(&tests).unwrap_or_else(|_| {
                eprintln!("Failed to parse tests: '{}'", tests);
                return vec![];
            });

            for (i, (t, k)) in tests.into_iter().enumerate() {
                if !spec::prop_delete_valid(t.clone(), k).unwrap_or(true) {
                    eprintln!(
                        "Test {} failed for DeleteValid: ({}, {})",
                        i,
                        serde_lexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k
                    );
                }
            }
        }
        "InsertPost" => {
            let tests: Vec<(Tree, i32, i32, i32)> =
                serde_lexpr::from_str(&tests).unwrap_or_else(|e| {
                    eprintln!("Failed to parse tests: '{}'", tests);
                    eprintln!("Error: {}", e);
                    return vec![];
                });

            for (i, (t, k, v, query_k)) in tests.into_iter().enumerate() {
                if !spec::prop_insert_post(t.clone(), k, v, query_k).unwrap_or(true) {
                    eprintln!(
                        "Test {} failed for InsertPost: ({}, {}, {}, {})",
                        i,
                        serde_lexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k,
                        v,
                        query_k
                    );
                }
            }
        }
        "DeletePost" => {
            let tests: Vec<(Tree, i32, i32)> = serde_lexpr::from_str(&tests).unwrap_or_else(|_| {
                eprintln!("Failed to parse tests: '{}'", tests);
                return vec![];
            });

            for (i, (t, k, query_k)) in tests.into_iter().enumerate() {
                if !spec::prop_delete_post(t.clone(), k, query_k).unwrap_or(true) {
                    eprintln!(
                        "Test {} failed for DeletePost: ({}, {}, {})",
                        i,
                        serde_lexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k,
                        query_k
                    );
                }
            }
        }
        "InsertModel" => {
            let tests: Vec<(Tree, i32, i32)> = serde_lexpr::from_str(&tests).unwrap_or_else(|_| {
                eprintln!("Failed to parse tests: '{}'", tests);
                return vec![];
            });

            for (i, (t, k, v)) in tests.into_iter().enumerate() {
                if !spec::prop_insert_model(t.clone(), k, v).unwrap_or(true) {
                    eprintln!(
                        "Test {} failed for InsertModel: ({}, {}, {})",
                        i,
                        serde_lexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k,
                        v
                    );
                }
            }
        }
        "DeleteModel" => {
            let tests: Vec<(Tree, i32)> = serde_lexpr::from_str(&tests).unwrap_or_else(|_| {
                eprintln!("Failed to parse tests: '{}'", tests);
                return vec![];
            });

            for (i, (t, k)) in tests.into_iter().enumerate() {
                if !spec::prop_delete_model(t.clone(), k).unwrap_or(true) {
                    eprintln!(
                        "Test {} failed for DeleteModel: ({}, {})",
                        i,
                        serde_lexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k
                    );
                }
            }
        }
        "InsertInsert" => {
            let tests: Vec<(Tree, i32, i32, i32, i32)> = serde_lexpr::from_str(&tests)
                .unwrap_or_else(|e| {
                    eprintln!("Failed to parse tests: '{}'", tests);
                    eprintln!("Error: {}", e);
                    return vec![];
                });

            for (i, (t, k, kp, v, vp)) in tests.into_iter().enumerate() {
                if !spec::prop_insert_insert(t.clone(), k, kp, v, vp).unwrap_or(true) {
                    eprintln!(
                        "Test {} failed for InsertInsert: ({}, {}, {}, {}, {})",
                        i,
                        serde_lexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k,
                        kp,
                        v,
                        vp
                    );
                }
            }
        }
        "InsertDelete" => {
            let tests: Vec<(Tree, i32, i32, i32)> =
                serde_lexpr::from_str(&tests).unwrap_or_else(|e| {
                    eprintln!("Failed to parse tests: '{}'", tests);
                    eprintln!("Error: {}", e);
                    return vec![];
                });

            for (i, (t, k, kp, v)) in tests.into_iter().enumerate() {
                if !spec::prop_insert_delete(t.clone(), k, kp, v).unwrap_or(true) {
                    eprintln!(
                        "Test {} failed for InsertDelete: ({}, {}, {}, {})",
                        i,
                        serde_lexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k,
                        kp,
                        v,
                    );
                }
            }
        }
        "DeleteInsert" => {
            let tests: Vec<(Tree, i32, i32, i32)> =
                serde_lexpr::from_str(&tests).unwrap_or_else(|e| {
                    eprintln!("Failed to parse tests: '{}'", tests);
                    eprintln!("Error: {}", e);
                    return vec![];
                });

            for (i, (t, k, kp, v)) in tests.into_iter().enumerate() {
                if !spec::prop_delete_insert(t.clone(), k, kp, v).unwrap_or(true) {
                    eprintln!(
                        "Test {} failed for DeleteInsert: ({}, {}, {}, {})",
                        i,
                        serde_lexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k,
                        kp,
                        v,
                    );
                }
            }
        }
        "DeleteDelete" => {
            let tests: Vec<(Tree, i32, i32)> = serde_lexpr::from_str(&tests).unwrap_or_else(|e| {
                eprintln!("Failed to parse tests: '{}'", tests);
                eprintln!("Error: {}", e);
                return vec![];
            });

            for (i, (t, k, kp)) in tests.into_iter().enumerate() {
                if !spec::prop_delete_delete(t.clone(), k, kp).unwrap_or(true) {
                    eprintln!(
                        "Test {} failed for DeleteDelete: ({}, {}, {})",
                        i,
                        serde_lexpr::to_string(&t)
                            .unwrap_or_else(|_| "failed to serialize tree".to_string()),
                        k,
                        kp,
                    );
                }
            }
        }

        _ => {
            eprintln!("Unknown property: {}", property);
        }
    }
}
