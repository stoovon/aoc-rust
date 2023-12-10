[What I learned solving in Rust](https://www.youtube.com/watch?v%253DudHjmno-tfA)

## Approach

1. Read and understand the puzzle
2. Parse the input
3. Write some code
4. Find solution
5. Submit solution
6. Repeat from point 1 for part 2

## Iterator combinators

- Sum sublists, take max.

```rust
fn combinators(input: &str) -> u64 {
    input
        .split("\n\n")
        .map(|batch| {
            batch
                .lines()
                .map(|line| line.parse::<u64>().unwrap())
                .sum::<u64>() // Consumes / pulsl from iterator
        })
        .max() // Consumes / pulls from iterator
        .unwrap()
}
```

- Don't do unwrap in production, this code is unsafe.
- Performant and flexible.

```rust
// cargo add itertools

use itertools::Itertools

fn combinators(input: &str) -> u64 {
    input
        .split("\n\n")
        .map(|batch| {
            batch
                .lines()
                .map(|line| line.parse::<u64>().unwrap())
                .sum::<u64>() // Consumes / pulls from iterator
        })
        .sorted()
        .rev()
        .take(3)
        .sum()
}
```

// More efficient

```rust
trait TopN<T> {
    fn top_n(self, n: usize) -> Vec<T>;
}

// Implement for all iterators (blanket implementation)

impl<T: PartialOrd, U: Iterator<Item=T>> TopN<T> for U {
    fn top_n(self, n: usize) -> Vec<T> {
        let mut top = Vec::with_capacity(n);
        for value in self {
            for i in 0..n {
                if let Some(top_value) = top.get(i) {
                    if value > *top_value {
                        top[i..].rotate_right(1);
                        top[i] = value;
                        break;
                    }

                } else {
                    top.push(value);
                    break;
                }
            }
        }

        top
    }
}
```

- Regexen

`cargo install regex`

`cargo add lazy_static`

```rust
#[macro_use]
extern crate lazy_static;
use regex::Regex;

lazy_static! {
    static ref LINE_REGEX: Regex = Regex::new(
        r"Sensor at x=(?P<x1>[-]?\d+), y=(?P<y1>[-]?\d+): closest beacon..."
    ).unwrap(); // Rust lets you use named capture groups. Nice.
}

fn parse_line_regex(line: &str) -> (Pos, Pos) {
    let captures = LINE_REGEX.captures(line).unwrap();
    let sensor = Pos {
        x: captures["x1"].parse().unwrap(),
        y: captures["y1"].parse().unwrap(),
    };
    let beacon = Pos {
        x: captures["x2"].parse().unwrap(),
        y; captures["y2"].parse().unwrap(),
    };
    (sensor, beacon)
}

pub fn parse_regex(input: &str) -> impl Iterator<Item = (Pos, Pos)> + '_ {
    input.lines().map(parse_line_regex)
}
```

NO REGEXEN

`cargo add nom`

```rust
fn parse_i64(input: &str) -> IResult<&str, i64> {
    let (input, sign) = opt(tag("-"))(input)?;
    let (input, value) = digit1(input)?;
    let mut value = value.parse::<i64>().unwrap();
    if sign.is_some() {
        value *= -1;
    }
    Ok((input, value))
}

// Learning curve but lovely code with error handling
// TURBO SPEED, 1700x times faster than regex.
fn parse_line(input: &str) -> IResult<&str, (Pos, Pos)> {
    let (input, (s_x, s_y, b_x, b_y)) = all_consuming(
        tuple((
            preceded(tag("Sensor at x="), parse_i64),
            preceded(tag(", y="), parse_i64),
            preceded(tag(": closest beacon is at x="), parse_i64),
            preceded(tag(", y="), parse_i64),
        ))
    )(input)?;
    let p1 = Pos { x: s_x, y: s_y };
    let p2 = Pos { x: b_x, y: b_y };
    Ok((input, (p1, p2)))
}
```

- Other great things:

  - The Iterator and FromIterator traits
  - Destructuring with `if let Some(x)` / `while let Some(x)`
  - Powerful `pattern-matching` syntax and the `matches!()` macro
  - `Newtype pattern` + `Deref` & `DerefMut` trait
  - The `Display` & `Debug` traits
  - `unreachable!()` & `todo!()` macros
  - `defaultdict` using `HashMap` and the `entry` API
  - Copy on Write (`CoW`)
  - Helper methods of the `Result` and `Option` types