# ETNA Proto

ETNA assumes 3 protocols over JSON for communication between ETNA and the framework, one for describing the tests to run,
one for invoking the framework, and one for reading the results from the frameworks.

## Test Description Protocol

The input parameters are passed in a JSON `test` object, examples of which can be found in `templates/tests/`.

The `test` object follows the following structure, `schemas/test.json` contains the schema for validation:

```json
[
        {
        "language": "Haskell",
        "workload": "BST",
        "mutations": [
            "insert_1"
        ],
        "trials": 10,
        "timeout": 60,
        "cross": false,
        "params": [],
        "tasks": [
            {
                "strategy": "Correct",
                "property": "InsertInsert"
            },
            {
                "strategy": "Correct",
                "property": "InsertPost"
            },
        ]
    },
]
```

All top level fields are used by ETNA for configuring the the experiment, the users can inject additional parameters
in the `params` field, which can be passed to the framework using the framework adapter.

## Test Invocation Protocol

The input parameters are passed in a JSON `invoke` object, examples of which can be found in `templates/invokes/`.

The `invoke` object follows the following structure, `schemas/invoke.json` contains the schema for validation:

```json
{
    "clean_steps": [
        {
            "Command": { "command": "make", "args": [ "clean" ] }
        }
    ],
    "build_steps": [
        {
            "Command": { "command": "$workload_path/build" }
        },
        {
            "Command": { "command": "$workload_path/build_generator", "args": [ "!generator" ] }
        },
        {
            "Command": { "command": "$workload_path/build_fuzzer", "args": [ "!fuzzer" ] }
        },
        {
            "Command": { "command": "$workload_path/build_sampler", "args": [ "!sampler" ] }
        }
    ],
    "run_step": {
        "Match": {
            "value": "strategy",
            "options": {
                "fuzzer": {
                    "Command": { "command": "$workload_path/main_exec", "args": [ "\"$strategy_exec $property\""] }
                },
                "generator": {
                    "Match": {
                        "value": "cross",
                        "options": {
                            "false": {
                                "Command": { "command": "$workload_path/$strategy_test_runner.native", "args": [ "$property" ] }
                            },
                            "true": {
                                "Command": { "command": "$workload_path/$strategy_sampler.native", "args": [ "$property", "$tests" ] }
                            }
                        }
                    }
                }
            }
        }
    },
    "tags": {
        "generator": [
            "BespokeGenerator",
            "SpecificationBasedGenerator",
            "TypeBasedGenerator"
        ],
        "fuzzer": [
            "TypeBasedFuzzer"
        ],
        "sampler": [
            "BespokeGenerator"
        ]
    }
}
```

todo: Explain the semantics of each field.

## Result Reading Protocol

The output of the framework is read using a JSON `result` object, examples of which can be found in `templates/results/`.

The `result` object follows the following structure, `schemas/result.json` contains the schema for validation:

```json
{
    "status": "FoundBug",
    "tests": 100,
    "passed": 95,
    "discarded": 4,
    "time": "0.42s",
    "counterexample": "(42 43)",
}
```

```json
{
    "status": "TimedOut",
    "tests": 100,
    "passed": 95,
    "discarded": 5,
    "timeout": 30,
}
```

```json
{
    "status": "Finished",
    "tests": 105,
    "passed": 100,
    "discarded": 5,
}
```

```json
{
    "status": "GaveUp",
    "tests": 105,
    "passed": 5,
    "discarded": 100,
}
```

```json
{
    "status": "Aborted",
    "error": "Failed to run the command",
    "tests": 0,
    "passed": 0,
    "discarded": 0,
}
```
