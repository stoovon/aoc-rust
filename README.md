# Advent of Code

Rust solutions grouped by year, with one module per puzzle in `YEAR/src/dDD/`.
Shared utilities live in `svutils`; the local-file command-line harness lives in
`runner`. Solver-specific input policies and mathematical explanations belong
beside their solutions in a local README when useful.

## Development

Run Cargo commands from the repository root. To use a container, open the
repository in VS Code and choose **Dev Containers: Reopen in Container**, or use
GitHub Codespaces. The configuration supplies stable Rust, rustfmt, Clippy,
rust-analyzer and LLDB as the non-root `vscode` user.

```sh
cargo test
```

Legacy file-based tests need locally supplied fixtures and expected answers.
Missing fixtures fail those tests; synthetic tests and runner tests need no
private files. To focus on one module and the runner:

```sh
cargo test -p aoc2023 d21::
cargo test -p aoc-runner
```

## Run a solution

Supply year, day, part, input path and optionally an expected-answer path:

```sh
cargo run -p aoc-runner -- 2023 21 1 /path/to/input.txt
cargo run -p aoc-runner -- 2023 21 2 /path/to/input.txt /path/to/solution_2.txt
```

The optional answer file is checked before the result is printed. File errors,
solver errors and answer mismatches exit unsuccessfully. Keep personal inputs,
answers and cache contents outside Git. The runner reads local files and does
not download inputs or submit answers.

For an existing cache, pass its files directly:

```sh
AOC_CACHE=/path/to/smelter-solves/.cache/2023/21
cargo run -p aoc-runner -- 2023 21 1 "$AOC_CACHE/input.txt" "$AOC_CACHE/solution_1.txt"
```

## Register a migrated solver

The explicit registry in [runner/src/main.rs](runner/src/main.rs) is the source
of truth for available selections. Unknown selections return an error.

Export the module from the year's `src/lib.rs` and add one `Day` entry to
`select_solver`. Solvers use `fn(&str) -> Result<String, svutils::SolveError>`;
set `part2` to `None` when unavailable. A new year also needs a path dependency
in `runner/Cargo.toml`.

Migrated solvers validate input and report arithmetic overflow. Parsing errors
can retain their underlying source and line or instruction-step context.
