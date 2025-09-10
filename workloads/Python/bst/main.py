import argparse
import sys
import time

from sampling import (
    sample_insert_valid,
    sample_delete_valid,
    sample_union_valid,
    sample_insert_post,
    sample_delete_post,
    sample_union_post,
    sample_insert_model,
    sample_delete_model,
    sample_union_model,
    sample_insert_insert,
    sample_insert_delete,
    sample_insert_union,
    sample_delete_insert,
    sample_delete_delete,
    sample_delete_union,
    sample_union_delete_insert,
    sample_union_union_idempotent,
    sample_union_union_assoc,
)

# Mapping from string names to Property(strategy, property) objects
PROPS = {
    "InsertValid": sample_insert_valid,
    "DeleteValid": sample_delete_valid,
    "UnionValid": sample_union_valid,
    "InsertPost": sample_insert_post,
    "DeletePost": sample_delete_post,
    "UnionPost": sample_union_post,
    "InsertModel": sample_insert_model,
    "DeleteModel": sample_delete_model,
    "UnionModel": sample_union_model,
    "InsertInsert": sample_insert_insert,
    "InsertDelete": sample_insert_delete,
    "InsertUnion": sample_insert_union,
    "DeleteInsert": sample_delete_insert,
    "DeleteDelete": sample_delete_delete,
    "DeleteUnion": sample_delete_union,
    "UnionDeleteInsert": sample_union_delete_insert,
    "UnionUnionIdempotent": sample_union_union_idempotent,
    "UnionUnionAssoc": sample_union_union_assoc,
}


def run_bespoke(property_name: str, max_tests: int, max_seconds: int) -> int:
    prop = PROPS[property_name]
    start = time.time()
    passed = 0
    # Evaluate property over generated examples, stop on first failure
    for i in range(1, max_tests + 1):
        if time.time() - start > max_seconds:
            break
        example = prop.strategy.example()
        try:
            ok = True
            if prop.property is not None:
                ok = bool(prop.property(example))
        except Exception as exc:  # Treat exceptions as failures
            print(f"FAILED after {i - 1} tests. Exception on example: {example}")
            print(f"Exception: {exc}")
            return 1
        if not ok:
            print(f"FAILED after {i - 1} tests. Counterexample: {example}")
            return 1
        passed += 1

    print(f"OK, passed {passed} tests")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="bst",
        description=(
            "ETNA BST workload for Python.\n"
            "Usage: bst <tool> <property>. Available tool: bespoke."
        ),
    )
    parser.add_argument("tool", type=str, choices=["bespoke"], help="Testing tool")
    parser.add_argument(
        "property",
        type=str,
        choices=list(PROPS.keys()),
        help="Name of the property to run",
    )
    parser.add_argument(
        "-n",
        "--tests",
        type=int,
        default=20000,
        help="Maximum number of tests to run (default: 20000)",
    )
    parser.add_argument(
        "--max-seconds",
        type=int,
        default=3600,
        help="Maximum wall-clock seconds to run (default: 3600)",
    )

    args = parser.parse_args()

    if args.tool != "bespoke":
        print(f"Unknown tool: {args.tool}")
        print(
            "For available properties, see https://github.com/alpaylan/etna-cli/blob/main/docs/workloads/bst.md"
        )
        return 2

    return run_bespoke(args.property, args.tests, args.max_seconds)


if __name__ == "__main__":
    sys.exit(main())
