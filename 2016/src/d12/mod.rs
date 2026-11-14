use svutils::SolveError;

const MAX_STEPS: usize = 100_000_000;

#[derive(Clone, Copy)]
enum Register {
    A,
    B,
    C,
    D,
}

impl Register {
    fn read(text: &str) -> Option<Self> {
        match text {
            "a" => Some(Self::A),
            "b" => Some(Self::B),
            "c" => Some(Self::C),
            "d" => Some(Self::D),
            _ => None,
        }
    }

    fn parse(text: &str) -> Result<Self, SolveError> {
        Self::read(text)
            .ok_or_else(|| SolveError::InvalidInput("expected register a, b, c or d".into()))
    }
}

#[derive(Default)]
struct Registers([i64; 4]);

#[derive(Clone, Copy)]
enum Operand {
    Immediate(i64),
    Register(Register),
}

impl Operand {
    fn parse(text: &str) -> Result<Self, SolveError> {
        match Register::read(text) {
            Some(register) => Ok(Self::Register(register)),
            None => Ok(Self::Immediate(text.parse()?)),
        }
    }

    fn value(self, registers: &Registers) -> i64 {
        match self {
            Self::Immediate(value) => value,
            Self::Register(register) => registers.0[register as usize],
        }
    }
}

#[derive(Clone, Copy)]
enum Instruction {
    Copy {
        source: Operand,
        destination: Register,
    },
    Increment(Register),
    Decrement(Register),
    JumpIfNonzero {
        condition: Operand,
        offset: Operand,
    },
}

fn parse_instruction(line: &str) -> Result<Instruction, SolveError> {
    let mut words = line.split_ascii_whitespace();
    match (words.next(), words.next(), words.next(), words.next()) {
        (Some("cpy"), Some(source), Some(destination), None) => Ok(Instruction::Copy {
            source: Operand::parse(source)?,
            destination: Register::parse(destination)?,
        }),
        (Some("inc"), Some(register), None, None) => {
            Ok(Instruction::Increment(Register::parse(register)?))
        }
        (Some("dec"), Some(register), None, None) => {
            Ok(Instruction::Decrement(Register::parse(register)?))
        }
        (Some("jnz"), Some(condition), Some(offset), None) => Ok(Instruction::JumpIfNonzero {
            condition: Operand::parse(condition)?,
            offset: Operand::parse(offset)?,
        }),
        _ => Err(SolveError::InvalidInput(
            "expected cpy, inc, dec or jnz with operands".into(),
        )),
    }
}

fn execute(
    program: &[Instruction],
    mut registers: Registers,
    limit: usize,
) -> Result<i64, SolveError> {
    let mut pc = 0i64;
    let mut steps = 0;
    while let Some(instruction) = usize::try_from(pc)
        .ok()
        .and_then(|index| program.get(index))
    {
        if steps == limit {
            return Err(SolveError::ResourceLimit("instruction budget exhausted"));
        }
        steps += 1;
        let mut advance = 1;
        match *instruction {
            Instruction::Copy {
                source,
                destination,
            } => {
                registers.0[destination as usize] = source.value(&registers);
            }
            Instruction::Increment(register) => {
                let value = &mut registers.0[register as usize];
                *value = value.checked_add(1).ok_or(SolveError::ArithmeticOverflow)?;
            }
            Instruction::Decrement(register) => {
                let value = &mut registers.0[register as usize];
                *value = value.checked_sub(1).ok_or(SolveError::ArithmeticOverflow)?;
            }
            Instruction::JumpIfNonzero { condition, offset } => {
                if condition.value(&registers) != 0 {
                    advance = offset.value(&registers);
                }
            }
        }
        pc = pc
            .checked_add(advance)
            .ok_or(SolveError::ArithmeticOverflow)?;
    }
    Ok(registers.0[Register::A as usize])
}

fn solve(input: &str, registers: Registers) -> Result<String, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let program = input
        .lines()
        .enumerate()
        .map(|(index, line)| parse_instruction(line).map_err(|error| error.at_line(index + 1)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(execute(&program, registers, MAX_STEPS)?.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, Registers::default())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut registers = Registers::default();
    registers.0[Register::C as usize] = 1;
    solve(input, registers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_share_execution_with_different_initial_registers() {
        let input = "cpy c a\r\ncpy 3 b\r\ninc a\r\ndec b\r\njnz b -2";
        assert_eq!(part1(input).unwrap(), "3");
        assert_eq!(part2(input).unwrap(), "4");
    }

    #[test]
    fn validates_destinations_and_instruction_arity() {
        for line in ["cpy b 7", "inc e", "dec", "jnz 1 2 extra"] {
            let input = format!("inc a\n{line}");
            assert!(matches!(
                part1(&input),
                Err(SolveError::AtLine { line: 2, .. })
            ));
        }
    }

    #[test]
    fn handles_signed_jumps_and_execution_errors() {
        assert_eq!(part1("cpy -2 d\njnz 1 d\ninc a").unwrap(), "0");
        assert!(matches!(
            part1("cpy 9223372036854775807 a\ninc a"),
            Err(SolveError::ArithmeticOverflow)
        ));
        let loop_instruction = parse_instruction("jnz 1 0").unwrap();
        assert!(matches!(
            execute(&[loop_instruction], Registers::default(), 3),
            Err(SolveError::ResourceLimit(_))
        ));
    }
}
