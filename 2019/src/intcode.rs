use std::collections::HashMap;
use svutils::SolveError;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Event {
    NeedInput,
    Output(i64),
    Halted,
}

pub(crate) struct Machine {
    memory: HashMap<i64, i64>,
    ip: i64,
    base: i64,
}

impl Machine {
    pub(crate) fn parse(input: &str) -> Result<Self, SolveError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let memory = input
            .split(',')
            .enumerate()
            .map(|(index, value)| {
                let value = value
                    .trim()
                    .parse::<i64>()
                    .map_err(|error| SolveError::from(error).at_step(index + 1))?;
                Ok((index as i64, value))
            })
            .collect::<Result<_, SolveError>>()?;
        Ok(Self {
            memory,
            ip: 0,
            base: 0,
        })
    }

    fn read(&self, address: i64) -> Result<i64, SolveError> {
        if address < 0 {
            return Err(SolveError::InvalidInput(format!(
                "negative Intcode address {address}"
            )));
        }
        Ok(self.memory.get(&address).copied().unwrap_or(0))
    }

    fn raw(&self, offset: i64) -> Result<i64, SolveError> {
        self.read(
            self.ip
                .checked_add(offset)
                .ok_or(SolveError::ArithmeticOverflow)?,
        )
    }

    fn relative(&self, value: i64) -> Result<i64, SolveError> {
        self.base
            .checked_add(value)
            .ok_or(SolveError::ArithmeticOverflow)
    }

    fn argument(&self, offset: i64, mode: i64) -> Result<i64, SolveError> {
        let value = self.raw(offset)?;
        match mode {
            0 => self.read(value),
            1 => Ok(value),
            2 => self.read(self.relative(value)?),
            _ => Err(SolveError::InvalidInput(format!(
                "invalid Intcode read mode {mode}"
            ))),
        }
    }

    fn destination(&self, offset: i64, mode: i64) -> Result<i64, SolveError> {
        let value = self.raw(offset)?;
        let address = match mode {
            0 => value,
            2 => self.relative(value)?,
            _ => {
                return Err(SolveError::InvalidInput(format!(
                    "invalid Intcode write mode {mode}"
                )))
            }
        };
        if address < 0 {
            return Err(SolveError::InvalidInput(format!(
                "negative Intcode address {address}"
            )));
        }
        Ok(address)
    }

    fn advance(&mut self, words: i64) -> Result<(), SolveError> {
        self.ip = self
            .ip
            .checked_add(words)
            .ok_or(SolveError::ArithmeticOverflow)?;
        Ok(())
    }

    pub(crate) fn run(&mut self, mut input: Option<i64>) -> Result<Event, SolveError> {
        loop {
            let instruction = self.read(self.ip)?;
            let opcode = instruction % 100;
            let first = instruction / 100 % 10;
            let second = instruction / 1000 % 10;
            let third = instruction / 10000 % 10;
            match opcode {
                1 | 2 | 7 | 8 => {
                    let a = self.argument(1, first)?;
                    let b = self.argument(2, second)?;
                    let address = self.destination(3, third)?;
                    let result = match opcode {
                        1 => a.checked_add(b).ok_or(SolveError::ArithmeticOverflow)?,
                        2 => a.checked_mul(b).ok_or(SolveError::ArithmeticOverflow)?,
                        7 => i64::from(a < b),
                        _ => i64::from(a == b),
                    };
                    self.advance(4)?;
                    self.memory.insert(address, result);
                }
                3 => {
                    let address = self.destination(1, first)?;
                    let Some(value) = input.take() else {
                        return Ok(Event::NeedInput);
                    };
                    self.advance(2)?;
                    self.memory.insert(address, value);
                }
                4 => {
                    let output = self.argument(1, first)?;
                    self.advance(2)?;
                    return Ok(Event::Output(output));
                }
                5 | 6 => {
                    let condition = self.argument(1, first)?;
                    let target = self.argument(2, second)?;
                    if (condition != 0) == (opcode == 5) {
                        if target < 0 {
                            return Err(SolveError::InvalidInput(format!(
                                "negative Intcode jump target {target}"
                            )));
                        }
                        self.ip = target;
                    } else {
                        self.advance(3)?;
                    }
                }
                9 => {
                    let base = self.relative(self.argument(1, first)?)?;
                    self.advance(2)?;
                    self.base = base;
                }
                99 => return Ok(Event::Halted),
                _ => {
                    return Err(SolveError::InvalidInput(format!(
                        "invalid Intcode opcode {opcode} at {}",
                        self.ip
                    )))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pauses_resumes_and_uses_sparse_relative_memory() {
        let mut vm = Machine::parse("109,500000000,203,4,204,4,204,5,99").unwrap();
        assert_eq!(vm.run(None).unwrap(), Event::NeedInput);
        assert_eq!(vm.run(None).unwrap(), Event::NeedInput);
        assert_eq!(vm.run(Some(-27)).unwrap(), Event::Output(-27));
        assert_eq!(vm.run(None).unwrap(), Event::Output(0));
        assert_eq!(vm.run(None).unwrap(), Event::Halted);
        assert_eq!(vm.run(None).unwrap(), Event::Halted);
    }

    #[test]
    fn computes_branches_and_comparisons() {
        let mut vm = Machine::parse("1101,6,7,50,1002,50,2,50,1007,50,27,51,1008,50,26,52,1006,51,24,1005,52,25,104,-1,99,4,50,99").unwrap();
        assert_eq!(vm.run(None).unwrap(), Event::Output(26));
        assert_eq!(vm.run(None).unwrap(), Event::Halted);
    }

    #[test]
    fn rejects_bad_addresses_modes_and_overflow() {
        for program in ["4,-1,99", "103,0,99", "1105,1,-1", "42"] {
            assert!(Machine::parse(program).unwrap().run(None).is_err());
        }
        assert!(matches!(
            Machine::parse("1101,9223372036854775807,1,0")
                .unwrap()
                .run(None),
            Err(SolveError::ArithmeticOverflow)
        ));
    }
}
