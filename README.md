# aoc
Advent of Code solutions

- [2023](./input/2023/README.md)
- [2022](./input/2022/README.md)
- [2021](./input/2021/README.md)
- [2020](./input/2020/README.md)
- [2019](./input/2019/README.md)
- [2018](./input/2018/README.md)
- [2017](./input/2017/README.md)
- [2016](./input/2016/README.md)
- [2015](./input/2015/README.md)

`cargo test`

Run the local-file harness for 2023 Day 3 from the repository root:

```sh
cargo run -p aoc2023 --bin aoc -- 1 /path/to/input.txt
cargo run -p aoc2023 --bin aoc -- 2 /path/to/input.txt /path/to/solution_2.txt
```

The optional final argument checks the computed answer against an existing answer
file before printing it. File errors, solver errors, and answer mismatches exit
unsuccessfully. This runner does not download inputs or submit answers. Keep
personal input and answer files outside Git; no answers are embedded in the runner.

For an existing `smelter-solves` cache, pass its files directly (adjust the path):

```sh
AOC_CACHE=/path/to/smelter-solves/.cache/2023/03
cargo run -p aoc2023 --bin aoc -- 1 "$AOC_CACHE/input.txt" "$AOC_CACHE/solution_1.txt"
cargo run -p aoc2023 --bin aoc -- 2 "$AOC_CACHE/input.txt" "$AOC_CACHE/solution_2.txt"
```

Run the Day 3 and runner tests with:

```sh
cargo test -p aoc2023 d03::
cargo test -p aoc2023 --test local_runner
```

The existing file-based Day 3 tests read fixtures at runtime from
`input/2023/d03/`: `input.txt`, `example.txt`, and their corresponding
`input-spec.1.txt`, `input-spec.2.txt`, `example-spec.1.txt`, and
`example-spec.2.txt` answer files. These paths are ignored by Git. Missing fixtures
fail those tests; the synthetic regression tests and runner test need no private
fixtures. The runner itself accepts explicit paths and requires no fixture setup.
