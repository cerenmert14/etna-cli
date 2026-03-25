# Demystifying Property-Based Testing through Evaluation & Visualization
The data presented in the paper for Tyche and ETNA experiments is located in the `thesis-experiments/thesis-data` directory for both frameworks.

## Reproducing the experiments
The experiments produced here used OCaml version 5.2.0, and Racket version 8.18. Follow the steps below to first install the necessary packages / languages

1. Ensure you have [OCaml](https://ocaml.org/install#linux_mac_bsd) and [`opam`](https://opam.ocaml.org/), along with [Racket](https://download.racket-lang.org/releases/8.18/).

2. Once those are installed, to reproduce Tyche experiments, pin the repositories that has Tyche support by running the commands below:
    * QCheck: `opam pin add qcheck https://github.com/cerenmert14/qcheck`
        * Confirm the module is pinned by running `opam list`
    * Rackcheck: `raco pkg install --pin https://github.com/cerenmert14/rackcheck/tree/master/rackcheck-lib`
        * Confirm the module is pinned by running `raco pkg show`
3. If you are using VS or VS Code, install the Tyche Extension [here](https://marketplace.visualstudio.com/items?itemName=HarrisonGoldstein.tyche)

Once you have the necessary packages / languages, first run `thesis-experiments/setup-etna.sh` to setup ETNA. Then, there are four seperate scripts to run experiments for ETNA task-bucket charts, and Tyche log files to plug them into Tyche:
* Run ETNA task-bucket charts via `thesis-experiments/etna-stlc-<qcheck || rackcheck>.sh`
    * This will populate results under `thesis-experiments/<qcheck || rackcheck>/etna-experiments`
* Run `thesis-experiments/tyche-stlc-<qcheck || rackcheck>.sh` to get the Tyche log files which will be located under `thesis-experiments/<qcheck || rackcheck>/tyche-experiments`. Once you have them, you can upload them to the Tyche either through the VS Code extension or on its [website](https://tyche-pbt.github.io/tyche-extension/)

Feel free to contact cmert@umd.edu with questions!