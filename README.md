# ETNA

ETNA is an Analysis and Evaluation Platform for benchmarking and analyzing the performance of Property-Based Testing (PBT) tools. It hosts a collection of testing workloads implemented in different languages, allowing users to plug-in their own PBT tools and libraries to compare their performance against others.

ETNA was originally written as a Python library that provided a set of APIs for accessing the workloads and running experiments, the library implementation can be found in the [jwshii/etna](https://github.com/jwshii/etna) repository.

This repository hosts a command line interface (CLI) for ETNA, which allows users to interact with the ETNA platform from the command line. The CLI provides commands to manage experiments, workloads, and results, making it easier to run and analyze benchmarks, detailed information regarding the installation and usage of the CLI can be found in the [CLI.md](CLI.md) file.

You can easly install the ETNA CLI, you can use the following CURL command:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/alpaylan/etna-cli/releases/download/v0.1.0/etna-installer.sh | sh
```

## Coverage

We are currently working on expanding the coverage of ETNA with more workloads and testing tools. Below is a list of the currently supported workloads and tools:

| Language       | Testing Tools                             | Workloads                              |
|:---------------|:------------------------------------------|:---------------------------------------|
| Haskell        | QuickCheck, LeanCheck, SmallCheck         | BST, RBT, STLC, System F<:, LuParser   |
| Rocq           | QuickChick                                | BST, RBT, STLC, IFC                    |
| Racket         | RackCheck                                 | BST, RBT, STLC, System F               |
| Rust           | QuickCheck(WIP)                           | BST, RBT, STLC                         |
