extern crate core;

use std::{collections::HashMap, str::FromStr};

#[derive(Clone)]
struct Reindeer {
    name: String,
    speed: i64,
    fly_time: i64,
    rest_time: i64,
}

impl Reindeer {
    fn new(name: &str, speed: i64, fly_time: i64, rest_time: i64) -> Self {
        Self {
            name: name.to_string(),
            speed,
            fly_time,
            rest_time,
        }
    }

    fn distance_after(&self, time: i64) -> i64 {
        let cycle_time = self.fly_time + self.rest_time;
        let cycles = time / cycle_time;
        let remainder = time % cycle_time;
        let fly_time = if remainder > self.fly_time {
            self.fly_time
        } else {
            remainder
        };
        cycles * self.speed * self.fly_time + self.speed * fly_time
    }
}

struct Race {
    reindeer: Vec<Reindeer>,
}

impl FromStr for Race {
    type Err = std::num::ParseIntError; // Define the associated type Err with a concrete type.

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reindeer = s
            .lines()
            .map(|line| {
                let mut words = line.split_whitespace();
                let name = words.next().unwrap();
                let speed = words.nth(2).unwrap().parse().unwrap();
                let fly_time = words.nth(2).unwrap().parse().unwrap();
                let rest_time = words.nth(6).unwrap().parse().unwrap();
                Reindeer::new(name, speed, fly_time, rest_time)
            })
            .collect();
        Ok(Self { reindeer })
    }
}

pub fn fn1(input: &str, race_duration: i64) -> i64 {
    let race: Race = input.parse().unwrap();

    let mut max_distance = 0;

    race.reindeer.iter().for_each(|reindeer| {
        let distance = reindeer.distance_after(race_duration);
        if distance > max_distance {
            max_distance = distance;
        }
    });

    max_distance
}

pub fn fn2(input: &str, race_duration: i64) -> i64 {
    let mut race: Race = input.parse().unwrap();

    let mut score = vec![0; input.len()];
    let mut distances: HashMap<String, i64> = HashMap::new();

    for minute in 1..race_duration + 1 {
        let mut lead = 0;

        for (_, reindeer) in race.reindeer.iter_mut().enumerate() {
            let next = reindeer.clone().distance_after(minute);
            distances.insert(reindeer.name.clone(), next);
            lead = lead.max(next);
        }

        for (index, distance) in distances.iter().enumerate() {
            if *distance.1 == lead {
                score[index] += 1;
            }
        }
    }

    *score.iter().max().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 14;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", |input| {
            fn1(input, 1000)
        });
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", |input| {
            fn1(input, 2503)
        });
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", |input| {
            fn2(input, 1000)
        });
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", |input| {
            fn2(input, 2503)
        });
    }
}
