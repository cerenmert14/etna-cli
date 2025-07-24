import sys
import argparse
import time
import json

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
    sample_union_union_idem,
    sample_union_union_assoc,
)

# Mapping from string names to property functions
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
    "UnionUnionIdem": sample_union_union_idem,
    "UnionUnionAssoc": sample_union_union_assoc,
}

class CustomEncoder(json.JSONEncoder):
    def default(self, o):
        if hasattr(o, "to_json"):
            return o.to_json()
        return super().default(o)

def main():
    parser = argparse.ArgumentParser(prog="rackcheck-bespoke")
    parser.add_argument("property", help="Name of the property to run")
    parser.add_argument("tests", type=int, help="Number of tests to run")
    args = parser.parse_args()

    if args.property not in PROPS:
        print(f"Unknown property: {args.property}")
        sys.exit(1)

    prop_fn = PROPS[args.property]

    print(f"Running {args.property} with {args.tests} tests...")

    # Run quick-sample using hypothesis.find()
    results = []
    for i in range(args.tests):
        t0 = time.time()
        e = prop_fn.strategy.example()
        t1 = time.time()
        duration = f"{t1 - t0:.6f}s"
        results.append({
            "time": duration,
            "value": repr(e),
        })
    print(json.dumps(results, cls=CustomEncoder))

if __name__ == "__main__":
    main()
