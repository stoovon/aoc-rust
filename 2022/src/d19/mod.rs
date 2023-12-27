extern crate core;

use regex::Regex;

#[derive(Clone, Copy, Debug)]
pub struct Blueprint {
    pub id: u8,
    pub ore_robot: u8,
    pub clay_robot: u8,
    pub obsidian_robot: (u8, u8),
    pub geode_robot: (u8, u8),
    pub max_ore_cost: u8,
}

impl Blueprint {
    #[inline]
    fn max_ore_cost(&self) -> u8 {
        self.max_ore_cost
    }

    #[inline]
    fn max_clay_cost(&self) -> u8 {
        self.obsidian_robot.1
    }

    #[inline]
    fn max_obsidian_cost(&self) -> u8 {
        self.geode_robot.1
    }
}

pub fn parse(input: &str) -> Vec<Blueprint> {
    let mut result = Vec::with_capacity(128);

    let re = Regex::new(r"\d+").unwrap();
    for line in input.lines() {
        let mut numbers = re.find_iter(line);
        let mut next = || -> u8 {
            let number = numbers.next().unwrap();
            number.as_str().parse().unwrap()
        };

        let id = next();
        let ore_robot = next();
        let clay_robot = next();
        let obsidian_robot = (next(), next());
        let geode_robot = (next(), next());
        result.push(Blueprint {
            id,
            ore_robot,
            clay_robot,
            obsidian_robot,
            geode_robot,
            max_ore_cost: ore_robot.max(clay_robot).max(obsidian_robot.0).max(geode_robot.0),
        });
        assert!(numbers.next().is_none());
    }

    result
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Pack {
    pub ore_robot:      u8,
    pub clay_robot:     u8,
    pub obsidian_robot: u8,
    pub geode_robot:    u8,
    pub ore:      u8,
    pub clay:     u8,
    pub obsidian: u8,
    pub geode:    u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct State {
    pub minute: u8,
    pub pack:   Pack,
}

impl State {
    pub fn new() -> Self {
        State {
            minute: 0,
            pack: Pack {
                ore_robot:      1,
                clay_robot:     0,
                obsidian_robot: 0,
                geode_robot:    0,
                ore:      0,
                clay:     0,
                obsidian: 0,
                geode:    0,
            },
        }
    }

    #[inline]
    pub fn can_build_ore_robot(&self, bp: &Blueprint) -> bool {
        self.pack.ore >= bp.ore_robot
    }

    #[inline]
    pub fn can_build_clay_robot(&self, bp: &Blueprint) -> bool {
        self.pack.ore >= bp.clay_robot
    }

    #[inline]
    pub fn can_build_obsidian_robot(&self, bp: &Blueprint) -> bool {
            self.pack.ore  >= bp.obsidian_robot.0
        && self.pack.clay >= bp.obsidian_robot.1
    }

    #[inline]
    pub fn can_build_geode_robot(&self, bp: &Blueprint) -> bool {
            self.pack.ore      >= bp.geode_robot.0
        && self.pack.obsidian >= bp.geode_robot.1
    }

    #[inline]
    pub fn build_ore_robot(self, bp: &Blueprint) -> Self {
        let mut result = self;
        result.pack.ore -= bp.ore_robot;
        result.pack.ore_robot += 1;
        return result;
    }

    #[inline]
    pub fn build_clay_robot(self, bp: &Blueprint) -> Self {
        let mut result = self;
        result.pack.ore -= bp.clay_robot;
        result.pack.clay_robot += 1;
        return result;
    }

    #[inline]
    pub fn build_obsidian_robot(self, bp: &Blueprint) -> Self {
        let mut result = self;
        result.pack.ore  -= bp.obsidian_robot.0;
        result.pack.clay -= bp.obsidian_robot.1;
        result.pack.obsidian_robot += 1;
        return result;
    }

    #[inline]
    pub fn build_geode_robot(self, bp: &Blueprint) -> Self {
        let mut result = self;
        result.pack.ore      -= bp.geode_robot.0;
        result.pack.obsidian -= bp.geode_robot.1;
        result.pack.geode_robot += 1;
        return result;
    }

    #[inline]
    pub fn step(self) -> Self {
        let mut this = self;
        this.minute += 1;
        this.pack.ore      += this.pack.ore_robot;
        this.pack.clay     += this.pack.clay_robot;
        this.pack.obsidian += this.pack.obsidian_robot;
        this.pack.geode    += this.pack.geode_robot;
        return this;
    }
}

fn solution(state: State, bp: &Blueprint, limit: u8, max_result: &mut u8,
    can_ore: bool, can_clay: bool, can_obsidian: bool
) {
    // OUTATIME
    if state.minute == limit {
        let result = state.pack.geode;
        *max_result = (*max_result).max(result);
        return;
    }

    // can we even beat max_result anymore?
    {
        // number of turns remaining.
        let remaining = (limit - state.minute) as u32;

        let max_yield =
                // future yield of current geode bots.
                remaining * state.pack.geode_robot as u32
                // max future yield, if we build one geode bot
                // on all future turns.
            + remaining*(remaining-1)/2;

        if state.pack.geode as u32 + max_yield <= *max_result as u32 {
            return;
        }
    }

    // always try to build a geode robot, it's the best option (especially once we've reached steady-state throughput of resources)
    if state.can_build_geode_robot(bp) {
        solution(state.step().build_geode_robot(bp), bp, limit, max_result, true, true, true);
    }
    else {
        let mut new_can_obsidian = true;
        if state.can_build_obsidian_robot(bp) {
            new_can_obsidian = false;

            // can only build one bot per turn.
            // so if our throughput of resources is sufficient to enable us to produce a steady stream of geode bots,
            // then anything else is waste
            if can_obsidian && state.pack.obsidian_robot < bp.max_obsidian_cost() {
                solution(state.step().build_obsidian_robot(bp), bp, limit, max_result, true, true, true);
            }
        }

        let mut new_can_clay = true;
        if state.can_build_clay_robot(bp) {
            new_can_clay = false;

            if can_clay && state.pack.clay_robot < bp.max_clay_cost() {
                solution(state.step().build_clay_robot(bp), bp, limit, max_result, true, true, true);
            }
        }

        let mut new_can_ore = true;
        if state.can_build_ore_robot(bp) {
            new_can_ore = false;

            if can_ore && state.pack.ore_robot < bp.max_ore_cost() {
                solution(state.step().build_ore_robot(bp), bp, limit, max_result, true, true, true);
            }
        }

        solution(state.step(), bp, limit, max_result, new_can_ore, new_can_clay, new_can_obsidian);
    }
}

pub fn fn1(input: &str) -> i64 {
    // Dynamic programming is recursion plus hashing.
    let bps = parse(input);
    let mut result = 0;
    for bp in &bps {
        let mut geodes = 0;
        solution(State::new(), bp, 24, &mut geodes, true, true, true);
        result += bp.id as u32 * geodes as u32;
    }
    result as i64
}

pub fn fn2(input: &str) -> i64 {
    let bps = parse(input);

    let mut geodes1 = 0;
    solution(State::new(), &bps[0], 32, &mut geodes1, true, true, true);
    let mut geodes2 = 0;
    solution(State::new(), &bps[1], 32, &mut geodes2, true, true, true);
    let mut geodes3 = 0;
    solution(State::new(), &bps[2], 32, &mut geodes3, true, true, true);

    geodes1 as i64 * geodes2 as i64 * geodes3 as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2022;
    const DAY: i16 = 19;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    // #[test]
    // fn test_fn2_example() {
    //     scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    // }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
