extern crate core;

use fancy_regex::Regex;
use std::collections::HashMap;

struct Instruction {
    action: Action,
    start: (usize, usize),
    end: (usize, usize),
}

enum Action {
    TurnOn,
    TurnOff,
    Toggle,
}

struct BrightGrid {
    grid: HashMap<(usize, usize), bool>,
}

impl BrightGrid {
    fn new() -> Self {
        Self {
            grid: HashMap::new(),
        }
    }

    fn turn_on(&mut self, start: (usize, usize), end: (usize, usize)) {
        for x in start.0..=end.0 {
            for y in start.1..=end.1 {
                self.grid.insert((x, y), true);
            }
        }
    }

    fn turn_off(&mut self, start: (usize, usize), end: (usize, usize)) {
        for x in start.0..=end.0 {
            for y in start.1..=end.1 {
                self.grid.insert((x, y), false);
            }
        }
    }

    fn toggle(&mut self, start: (usize, usize), end: (usize, usize)) {
        for x in start.0..=end.0 {
            for y in start.1..=end.1 {
                let value = self.grid.get(&(x, y)).unwrap_or(&false);
                self.grid.insert((x, y), !value);
            }
        }
    }

    fn count(&self) -> usize {
        self.grid.values().filter(|&&v| v).count()
    }
}

struct FadingGrid {
    grid: HashMap<(usize, usize), i8>,
}

impl FadingGrid {
    fn new() -> Self {
        Self {
            grid: HashMap::new(),
        }
    }

    fn turn_on(&mut self, start: (usize, usize), end: (usize, usize)) {
        for x in start.0..=end.0 {
            for y in start.1..=end.1 {
                self.grid
                    .entry((x, y))
                    .and_modify(|brightness| *brightness += 1)
                    .or_insert(1);
            }
        }
    }

    fn turn_off(&mut self, start: (usize, usize), end: (usize, usize)) {
        for x in start.0..=end.0 {
            for y in start.1..=end.1 {
                self.grid
                    .entry((x, y))
                    .and_modify(|brightness| {
                        let new_brightness = *brightness - 1;
                        if new_brightness > 0 {
                            *brightness = new_brightness;
                        } else {
                            *brightness = 0;
                        }
                    })
                    .or_insert(0);
            }
        }
    }

    fn toggle(&mut self, start: (usize, usize), end: (usize, usize)) {
        for x in start.0..=end.0 {
            for y in start.1..=end.1 {
                self.grid
                    .entry((x, y))
                    .and_modify(|brightness| *brightness += 2)
                    .or_insert(2);
            }
        }
    }

    fn count(&self) -> usize {
        self.grid.values().map(|light| *light as usize).sum()
    }
}

fn parse(input: &str) -> Vec<Instruction> {
    let re = Regex::new(r"^(turn on|turn off|toggle) (\d+),(\d+) through (\d+),(\d+)$").unwrap();
    input
        .lines()
        .map(|line| {
            let caps = re.captures(line).unwrap().unwrap();
            let action = match &caps[1] {
                "turn on" => Action::TurnOn,
                "turn off" => Action::TurnOff,
                "toggle" => Action::Toggle,
                _ => panic!("Unknown action"),
            };
            let start = (
                caps[2].parse::<usize>().unwrap(),
                caps[3].parse::<usize>().unwrap(),
            );
            let end = (
                caps[4].parse::<usize>().unwrap(),
                caps[5].parse::<usize>().unwrap(),
            );
            Instruction { action, start, end }
        })
        .collect()
}

pub fn fn1(input: &str) -> i64 {
    let instructions = parse(input);

    let mut grid = BrightGrid::new();

    for instruction in instructions {
        match instruction.action {
            Action::TurnOn => grid.turn_on(instruction.start, instruction.end),
            Action::TurnOff => grid.turn_off(instruction.start, instruction.end),
            Action::Toggle => grid.toggle(instruction.start, instruction.end),
        }
    }

    grid.count() as i64
}

pub fn fn2(input: &str) -> i64 {
    let instructions = parse(input);

    let mut grid = FadingGrid::new();

    for instruction in instructions {
        match instruction.action {
            Action::TurnOn => grid.turn_on(instruction.start, instruction.end),
            Action::TurnOff => grid.turn_off(instruction.start, instruction.end),
            Action::Toggle => grid.toggle(instruction.start, instruction.end),
        }
    }

    grid.count() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 6;

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
