use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use svutils::SolveError;

type Number = u64;

#[derive(Clone, Copy)]
enum Target {
    Bot(Number),
    Output(Number),
}

impl Target {
    fn parse(kind: &str, id: &str) -> Result<Self, SolveError> {
        match kind {
            "bot" => Ok(Self::Bot(id.parse()?)),
            "output" => Ok(Self::Output(id.parse()?)),
            _ => Err(SolveError::InvalidInput("expected bot or output".into())),
        }
    }
}

#[derive(Default)]
struct Factory {
    rules: HashMap<Number, (Target, Target)>,
    chips: BTreeMap<Number, Vec<Number>>,
    ready: VecDeque<Number>,
    outputs: [Option<Number>; 3],
}

impl Factory {
    fn parse(input: &str) -> Result<Self, SolveError> {
        if input.is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut factory = Self::default();
        for (index, line) in input.lines().enumerate() {
            factory
                .parse_line(line)
                .map_err(|error| error.at_line(index + 1))?;
        }
        Ok(factory)
    }

    fn parse_line(&mut self, line: &str) -> Result<(), SolveError> {
        let words: Vec<_> = line.split_ascii_whitespace().collect();
        match words.as_slice() {
            ["value", value, "goes", "to", "bot", bot] => {
                self.deliver(Target::Bot(bot.parse()?), value.parse()?)
            }
            ["bot", bot, "gives", "low", "to", low_kind, low_id, "and", "high", "to", high_kind, high_id] =>
            {
                let bot = bot.parse()?;
                let rule = (
                    Target::parse(low_kind, low_id)?,
                    Target::parse(high_kind, high_id)?,
                );
                if self.rules.insert(bot, rule).is_some() {
                    return Err(SolveError::InvalidInput(format!(
                        "duplicate rule for bot {bot}"
                    )));
                }
                Ok(())
            }
            _ => Err(SolveError::InvalidInput(
                "expected a chip assignment or bot rule".into(),
            )),
        }
    }

    fn deliver(&mut self, target: Target, value: Number) -> Result<(), SolveError> {
        match target {
            Target::Bot(bot) => {
                let chips = self.chips.entry(bot).or_default();
                if chips.len() == 2 {
                    return Err(SolveError::InvalidInput(format!(
                        "bot {bot} received a third chip"
                    )));
                }
                chips.push(value);
                if chips.len() == 2 {
                    self.ready.push_back(bot);
                }
            }
            Target::Output(bin @ 0..=2) => {
                self.outputs[bin as usize].get_or_insert(value);
            }
            Target::Output(_) => {}
        }
        Ok(())
    }

    fn run(
        mut self,
        mut compare: impl FnMut(Number, Number, Number) -> Option<Number>,
        mut output: impl FnMut([Option<Number>; 3]) -> Result<Option<Number>, SolveError>,
    ) -> Result<Number, SolveError> {
        let mut seen = HashSet::new();
        while !self.ready.is_empty() {
            if !seen.insert((self.chips.clone(), self.ready.clone(), self.outputs)) {
                return Err(SolveError::NoSolution(
                    "bot network repeats without reaching the goal",
                ));
            }
            let bot = self.ready.pop_front().unwrap();
            let chips = self.chips.remove(&bot).unwrap();
            let (low, high) = (chips[0].min(chips[1]), chips[0].max(chips[1]));
            if let Some(answer) = compare(bot, low, high) {
                return Ok(answer);
            }
            let &(low_target, high_target) = self.rules.get(&bot).ok_or_else(|| {
                SolveError::InvalidInput(format!("missing rule for ready bot {bot}"))
            })?;
            self.deliver(low_target, low)?;
            self.deliver(high_target, high)?;
            if let Some(answer) = output(self.outputs)? {
                return Ok(answer);
            }
        }
        Err(SolveError::NoSolution(
            "no ready bots and requested result was not found",
        ))
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let bot = Factory::parse(input)?.run(
        |bot, low, high| (low == 17 && high == 61).then_some(bot),
        |_| Ok(None),
    )?;
    Ok(bot.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let product = Factory::parse(input)?.run(
        |_, _, _| None,
        |outputs| match outputs {
            [Some(a), Some(b), Some(c)] => {
                let product = if a == 0 || b == 0 || c == 0 {
                    Some(0)
                } else {
                    a.checked_mul(b).and_then(|ab| ab.checked_mul(c))
                };
                product.map(Some).ok_or(SolveError::ArithmeticOverflow)
            }
            _ => Ok(None),
        },
    )?;
    Ok(product.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_chips_and_allows_a_bot_to_receive_its_own_chip() {
        let input = concat!(
            "value 17 goes to bot 4\nvalue 9 goes to bot 4\n",
            "value 61 goes to bot 5\nvalue 70 goes to bot 5\n",
            "bot 4 gives low to output 0 and high to bot 4\n",
            "bot 5 gives low to bot 4 and high to output 1\n",
        );
        assert_eq!(part1(input).unwrap(), "4");
        let routed = input.replace("high to bot 4", "high to output 2");
        assert_eq!(part2(&routed).unwrap(), "10710");
    }

    #[test]
    fn reports_stalled_and_cyclic_networks() {
        let stalled = "value 5 goes to bot 4";
        let cyclic = concat!(
            "value 5 goes to bot 4\nvalue 7 goes to bot 4\n",
            "bot 4 gives low to bot 4 and high to bot 4\n",
        );
        for solve in [part1, part2] {
            for input in [stalled, cyclic] {
                assert!(matches!(solve(input), Err(SolveError::NoSolution(_))));
            }
        }
    }
}
