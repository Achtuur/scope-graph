# Memoising Scope Graph Query Resolution

This is the repository containing the implementations of both `A_mem` and `A_cur`, alongside the tools to recreate all graphs in the thesis.

## Organisation

* `scope-graph`: contains Scope graph and query resolution algorithm definitions
* `data-parse`: parses raw data obtained from [artifact of Specializing Scope Graph Resolution Queries](https://zenodo.org/records/7189413).
* `pattern-recog`: crude pattern recognition for data obtained from `data-parse`.
* `graph_bench`: graph and table generator for benchmark results.
* `graphing`: utilities to create MMD and PlantUML diagrams, for debugging.


## Requirements

It is recommended to use [`uv`](https://docs.astral.sh/uv/) for the python scripts. To install, run:

```sh
curl -LsSf https://astral.sh/uv/install.sh | sh
```

If this doesn't work, consult the official documentation.

## How to run

Steps to recreate the benchmarks:

1. Run the benchmarks

```sh
cargo bench -p scope-graph --bench sg-patterns
```

This can take between 5-10 minutes, the results are in `scope-graph/output/benches/results.json`

2. Create graphs

With `uv`:
```sh
cd graph_bench
uv run plot_bench.py
```

The figures and tables are saved in `graph_bench/plots/`


## `data-parse`

`data-parse` parses the data obtained from the artifact and provides a `cosmo.csv` to view the resulting scope graph in [`cosmograph`](https://cosmograph.app/). Run it with:

```sh
cargo run -p data-parse --release
```

It is recommended to run in a release build, as deserialisation can take a while.