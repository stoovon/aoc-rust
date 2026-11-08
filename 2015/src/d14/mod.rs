use std::{collections::HashSet, str::FromStr};
use svutils::SolveError;

const RACE_DURATION: u64 = 2503;
const MAX_REINDEER: usize = 1024;
const MAX_SCORING_UPDATES: u64 = 10_000_000;

struct Reindeer {
    name: String,
    speed: u64,
    fly_time: u64,
    rest_time: u64,
}

impl Reindeer {
    fn parse(line: &str) -> Result<Self, SolveError> {
        let tokens: Vec<_> = line.split_ascii_whitespace().collect();
        let [name, "can", "fly", speed, "km/s", "for", fly_time, "seconds,", "but", "then", "must", "rest", "for", rest_time, "seconds."] =
            tokens.as_slice()
        else {
            return Err(SolveError::InvalidInput(
                "expected a complete reindeer flight and rest statement".into(),
            ));
        };
        if !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(SolveError::InvalidInput(
                "expected an ASCII reindeer name".into(),
            ));
        }
        let number = |text: &str| -> Result<u64, SolveError> {
            if !text.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(SolveError::InvalidInput(
                    "expected nonnegative decimal race parameters".into(),
                ));
            }
            Ok(text.parse()?)
        };
        let reindeer = Self {
            name: (*name).to_owned(),
            speed: number(speed)?,
            fly_time: number(fly_time)?,
            rest_time: number(rest_time)?,
        };
        reindeer.cycle_time()?;
        Ok(reindeer)
    }

    fn cycle_time(&self) -> Result<u64, SolveError> {
        let cycle = self
            .fly_time
            .checked_add(self.rest_time)
            .ok_or(SolveError::ArithmeticOverflow)?;
        if cycle == 0 {
            return Err(SolveError::InvalidInput(
                "flying and resting times cannot both be zero".into(),
            ));
        }
        Ok(cycle)
    }

    fn distance_after(&self, time: u64) -> Result<u64, SolveError> {
        let cycle = self.cycle_time()?;
        let full_cycles = time / cycle;
        let partial_flight = (time % cycle).min(self.fly_time);
        let flying = full_cycles
            .checked_mul(self.fly_time)
            .and_then(|full| full.checked_add(partial_flight))
            .ok_or(SolveError::ArithmeticOverflow)?;
        flying
            .checked_mul(self.speed)
            .ok_or(SolveError::ArithmeticOverflow)
    }
}

struct Race {
    reindeer: Vec<Reindeer>,
}

struct Standing<'a> {
    reindeer: &'a Reindeer,
    distance: u64,
    points: u64,
}

impl FromStr for Race {
    type Err = SolveError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut reindeer = Vec::new();
        for (index, line) in input.lines().enumerate() {
            if reindeer.len() == MAX_REINDEER {
                return Err(SolveError::ResourceLimit("too many reindeer").at_line(index + 1));
            }
            reindeer.push(Reindeer::parse(line).map_err(|error| error.at_line(index + 1))?);
        }
        let mut names = HashSet::new();
        for (index, racer) in reindeer.iter().enumerate() {
            if !names.insert(racer.name.as_str()) {
                return Err(
                    SolveError::InvalidInput("duplicate reindeer name".into()).at_line(index + 1)
                );
            }
        }
        Ok(Self { reindeer })
    }
}

impl Race {
    fn winning_distance(&self, duration: u64) -> Result<u64, SolveError> {
        self.reindeer.iter().try_fold(0, |best, reindeer| {
            Ok(best.max(reindeer.distance_after(duration)?))
        })
    }

    fn standings_after(&self, duration: u64) -> Result<Vec<Standing<'_>>, SolveError> {
        let work =
            duration
                .checked_mul(self.reindeer.len() as u64)
                .ok_or(SolveError::ResourceLimit(
                    "race scoring work budget exceeded",
                ))?;
        if work > MAX_SCORING_UPDATES {
            return Err(SolveError::ResourceLimit(
                "race scoring work budget exceeded",
            ));
        }
        let mut standings: Vec<_> = self
            .reindeer
            .iter()
            .map(|reindeer| Standing {
                reindeer,
                distance: 0,
                points: 0,
            })
            .collect();
        for second in 1..=duration {
            let mut lead = 0;
            for standing in &mut standings {
                standing.distance = standing.reindeer.distance_after(second)?;
                lead = lead.max(standing.distance);
            }
            for standing in &mut standings {
                if standing.distance == lead {
                    standing.points += 1;
                }
            }
        }
        Ok(standings)
    }

    fn winning_points(&self, duration: u64) -> Result<u64, SolveError> {
        self.standings_after(duration)?
            .iter()
            .map(|standing| standing.points)
            .max()
            .ok_or(SolveError::EmptyInput)
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    input
        .parse::<Race>()?
        .winning_distance(RACE_DURATION)
        .map(|distance| distance.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    input
        .parse::<Race>()?
        .winning_points(RACE_DURATION)
        .map(|points| points.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn racer(name: &str, speed: u64, fly_time: u64, rest_time: u64) -> Reindeer {
        Reindeer {
            name: name.into(),
            speed,
            fly_time,
            rest_time,
        }
    }

    fn record(name: &str, speed: u64, fly_time: u64, rest_time: u64) -> String {
        format!("{name} can fly {speed} km/s for {fly_time} seconds, but then must rest for {rest_time} seconds.")
    }

    #[test]
    fn standings_keep_racers_and_award_all_ties() {
        let race = Race {
            reindeer: vec![racer("Copper", 4, 1, 1), racer("Silver", 2, 2, 1)],
        };
        let standings = race.standings_after(4).unwrap();
        assert_eq!(standings[0].reindeer.name, "Copper");
        assert_eq!((standings[0].distance, standings[0].points), (8, 4));
        assert_eq!(standings[1].reindeer.name, "Silver");
        assert_eq!((standings[1].distance, standings[1].points), (6, 1));
        let tied = Race {
            reindeer: vec![racer("Copper", 4, 1, 1), racer("Silver", 4, 1, 1)],
        };
        assert!(tied
            .standings_after(3)
            .unwrap()
            .iter()
            .all(|standing| standing.points == 3));
        assert!(tied
            .standings_after(0)
            .unwrap()
            .iter()
            .all(|standing| standing.points == 0));
        let stationary = Race {
            reindeer: vec![racer("Copper", 0, 1, 1), racer("Silver", 0, 2, 1)],
        };
        assert!(stationary
            .standings_after(4)
            .unwrap()
            .iter()
            .all(|standing| standing.points == 4));
        let reversed = Race {
            reindeer: race.reindeer.into_iter().rev().collect(),
        };
        let standings = reversed.standings_after(4).unwrap();
        assert_eq!(standings[0].reindeer.name, "Silver");
        assert_eq!(standings[0].points, 1);
        assert_eq!(standings[1].reindeer.name, "Copper");
        assert_eq!(standings[1].points, 4);
    }

    #[test]
    fn zero_phases_and_cycle_boundaries() {
        let running = racer("Copper", 7, 3, 2);
        for (time, distance) in [(0, 0), (1, 7), (3, 21), (4, 21), (5, 21), (6, 28), (10, 42)] {
            assert_eq!(running.distance_after(time).unwrap(), distance);
        }
        for time in 0..20 {
            assert_eq!(racer("Copper", 7, 0, 3).distance_after(time).unwrap(), 0);
            assert_eq!(
                racer("Copper", 7, 3, 0).distance_after(time).unwrap(),
                7 * time
            );
            assert_eq!(racer("Copper", 0, 3, 2).distance_after(time).unwrap(), 0);
        }
        assert!(record("Copper", 7, 0, 0).parse::<Race>().is_err());
    }

    #[test]
    fn distances_and_scores_match_phase_simulation() {
        let mut cases = 0;
        for speed in 0..=4 {
            for fly in 0..=4 {
                for rest in 0..=4 {
                    if fly + rest == 0 {
                        continue;
                    }
                    let race = Race {
                        reindeer: vec![racer("Copper", speed, fly, rest), racer("Silver", 3, 2, 3)],
                    };
                    let mut phase_left = [fly, 2];
                    let mut flying = [true, true];
                    let mut distances = [0u64; 2];
                    let mut points = [0u64; 2];
                    for time in 1..=30 {
                        for index in 0..2 {
                            let reindeer = &race.reindeer[index];
                            while phase_left[index] == 0 {
                                flying[index] = !flying[index];
                                phase_left[index] = if flying[index] {
                                    reindeer.fly_time
                                } else {
                                    reindeer.rest_time
                                };
                            }
                            if flying[index] {
                                distances[index] += reindeer.speed;
                            }
                            phase_left[index] -= 1;
                            assert_eq!(reindeer.distance_after(time).unwrap(), distances[index]);
                        }
                        let lead = *distances.iter().max().unwrap();
                        for index in 0..2 {
                            if distances[index] == lead {
                                points[index] += 1;
                            }
                        }
                        let standings = race.standings_after(time).unwrap();
                        for index in 0..2 {
                            assert_eq!(standings[index].distance, distances[index]);
                            assert_eq!(standings[index].points, points[index]);
                        }
                        assert_eq!(race.winning_distance(time).unwrap(), lead);
                        assert_eq!(
                            race.winning_points(time).unwrap(),
                            *points.iter().max().unwrap()
                        );
                        cases += 1;
                    }
                }
            }
        }
        assert_eq!(cases, 3600);
    }

    #[test]
    fn parser_rejects_invalid_and_duplicate_records() {
        for solver in [part1, part2] {
            for input in ["", " \r\n"] {
                assert!(matches!(solver(input), Err(SolveError::EmptyInput)));
            }
            for bad in ["", "broken", "Copper can swim 7 km/s for 3 seconds, but then must rest for 2 seconds.",
                "Copper can fly -7 km/s for 3 seconds, but then must rest for 2 seconds.",
                "Copper can fly +7 km/s for 3 seconds, but then must rest for 2 seconds.",
                "Copper can fly 7 km/s for 3 seconds, but then must rest for 2 seconds",
                "Copper can fly 7 km/s for 3 seconds, but then must rest for 2 seconds. extra",
                "Cøpper can fly 7 km/s for 3 seconds, but then must rest for 2 seconds.",
                "Copper can fly 18446744073709551616 km/s for 3 seconds, but then must rest for 2 seconds."] {
                let input=format!("{}\n{bad}\n",record("Copper",7,3,2));
                assert!(solver(&input).unwrap_err().to_string().starts_with("line 2:"));
            }
            let duplicate = format!(
                "{}\n{}",
                record("Copper", 7, 3, 2),
                record("Copper", 3, 2, 1)
            );
            assert!(solver(&duplicate)
                .unwrap_err()
                .to_string()
                .starts_with("line 2:"));
        }
        let input = format!(
            " {}\r\n\t{} \r\n",
            record("Copper", 7, 3, 2),
            record("Silver", 3, 2, 1)
        );
        assert!(part1(&input).is_ok());
        assert!(part2(&input).is_ok());
    }

    #[test]
    fn arithmetic_and_resource_budgets_return_errors() {
        assert!(matches!(
            Reindeer::parse(&record("Copper", 1, u64::MAX, 1)),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert!(matches!(
            racer("Copper", u64::MAX, 1, 0).distance_after(2),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert_eq!(
            racer("Copper", 1, u64::MAX, 0)
                .distance_after(u64::MAX)
                .unwrap(),
            u64::MAX
        );
        let race = Race {
            reindeer: vec![racer("Copper", 1, 1, 1)],
        };
        assert!(matches!(
            race.standings_after(MAX_SCORING_UPDATES + 1),
            Err(SolveError::ResourceLimit(_))
        ));
        assert!(matches!(
            race.standings_after(u64::MAX),
            Err(SolveError::ResourceLimit(_))
        ));
        let input = (0..=MAX_REINDEER)
            .map(|index| record(&format!("R{index}"), 1, 1, 1))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(part1(&input)
            .unwrap_err()
            .to_string()
            .contains("too many reindeer"));
        assert!(part2(&input)
            .unwrap_err()
            .to_string()
            .contains("too many reindeer"));
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 14, "input.txt", "input-spec.1.txt", part1);
    }
    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 14, "input.txt", "input-spec.2.txt", part2);
    }
}
