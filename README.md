# aoc
Advent of Code solutions

`cargo test`

For a container environment, open the repository in VS Code and choose
**Dev Containers: Reopen in Container**, or open it in GitHub Codespaces.
The configuration uses the maintained Microsoft Rust devcontainer image with
stable Rust, rustfmt, Clippy, rust-analyzer, and LLDB, running as the non-root
`vscode` user. Local Cargo commands below work inside the container too; supply
input and answer files separately.

Run the local-file harness (year, day, part, input path, optional answer path) from the repository root:

```sh
cargo run -p aoc-runner -- 2023 3 1 /path/to/input.txt
cargo run -p aoc-runner -- 2023 3 2 /path/to/input.txt /path/to/solution_2.txt
```

The optional final argument checks the computed answer against an existing answer
file before printing it. File errors, solver errors, and answer mismatches exit
unsuccessfully. This runner does not download inputs or submit answers. Keep
personal input and answer files outside Git; no answers are embedded in the runner.

For an existing `smelter-solves` cache, pass its files directly (adjust the path):

```sh
AOC_CACHE=/path/to/smelter-solves/.cache/2023/03
cargo run -p aoc-runner -- 2023 3 1 "$AOC_CACHE/input.txt" "$AOC_CACHE/solution_1.txt"
cargo run -p aoc-runner -- 2023 3 2 "$AOC_CACHE/input.txt" "$AOC_CACHE/solution_2.txt"
```

Currently registered: 2023 Days 1–6, both parts. Other selections return an error.

To register another migrated day, export its module from the year's `src/lib.rs`
and add one `Day` entry to `select_solver` in `runner/src/main.rs`. Each solver
has the signature `fn(&str) -> Result<String, svutils::SolveError>`. Set `part2`
to `None` when that day has no second solver. A new year also needs a path
dependency in `runner/Cargo.toml`.

Run the migrated-day and runner tests with:

```sh
cargo test -p aoc2023 d01::
cargo test -p aoc2023 d02::
cargo test -p aoc2023 d03::
cargo test -p aoc2023 d04::
cargo test -p aoc2023 d05::
cargo test -p aoc2023 d06::
cargo test -p aoc-runner
```

Legacy file-based tests require locally supplied private fixtures and expected
answers; missing fixtures fail those tests. Synthetic regression tests and runner
tests need no private fixtures. Use the runner's explicit file paths to check
solutions against an existing external cache.
