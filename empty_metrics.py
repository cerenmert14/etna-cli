
# read the store.json at the given path
import json
import os
import sys

def empty_metrics(store_path):
    """
    Empty the metrics in the store.json file at the given path.
    """
    store_file = os.path.join(store_path, 'store.json')
    
    if not os.path.exists(store_file):
        print(f"Store file does not exist at {store_file}")
        sys.exit(1)

    with open(store_file, 'r') as f:
        store = json.load(f)

    # Empty the metrics
    store['metrics'] = {}

    with open(store_file, 'w') as f:
        json.dump(store, f, indent=4)

    print("Metrics have been emptied successfully.")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python empty_metrics.py <store_path>")
        sys.exit(1)

    store_path = sys.argv[1]
    empty_metrics(store_path)
else:
    print("This script is intended to be run as a standalone program.")
    print("Please use the command line to execute it.")
    sys.exit(1)