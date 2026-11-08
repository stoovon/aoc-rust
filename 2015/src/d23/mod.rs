use svutils::SolveError;

const MAX_STEPS: usize = 1_000_000;

#[derive(Debug, PartialEq)]
enum Register {
    A,
    B,
}

#[derive(Debug)]
enum Instruction {
    Hlf(Register),
    Tpl(Register),
    Inc(Register),
    Jmp(i32),
    Jie(Register, i32),
    Jio(Register, i32),
}

struct CPU {
    a: u64,
    b: u64,
    pc: i64,
}

pub struct VM {
    program: Vec<Instruction>,
    cpu: CPU,
}

impl VM {
    pub fn new() -> Self {
        Self {
            program: Vec::new(),
            cpu: CPU { a: 0, b: 0, pc: 0 },
        }
    }

    fn parse(&mut self, input: &str) -> Result<(), SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let program = input
            .lines()
            .enumerate()
            .map(|(index, line)| parse_instruction(line).map_err(|error| error.at_line(index + 1)))
            .collect::<Result<Vec<_>, _>>()?;
        self.program = program;
        Ok(())
    }

    fn run(&mut self) -> Result<(), SolveError> {
        self.run_with_limit(MAX_STEPS).map(|_| ())
    }

    fn run_with_limit(&mut self, limit: usize) -> Result<usize, SolveError> {
        let mut steps = 0;
        while let Ok(pc) = usize::try_from(self.cpu.pc) {
            let Some(instruction) = self.program.get(pc) else {
                break;
            };
            if steps == limit {
                return Err(SolveError::ResourceLimit("instruction budget exhausted"));
            }
            steps += 1;
            match instruction {
                Instruction::Hlf(r) => {
                    if *r == Register::A {
                        self.cpu.a /= 2;
                    } else {
                        self.cpu.b /= 2;
                    }
                    self.cpu.pc += 1;
                }

                Instruction::Tpl(r) => {
                    if *r == Register::A {
                        self.cpu.a = self
                            .cpu
                            .a
                            .checked_mul(3)
                            .ok_or(SolveError::ArithmeticOverflow)?;
                    } else {
                        self.cpu.b = self
                            .cpu
                            .b
                            .checked_mul(3)
                            .ok_or(SolveError::ArithmeticOverflow)?;
                    }
                    self.cpu.pc += 1;
                }
                Instruction::Inc(r) => {
                    if *r == Register::A {
                        self.cpu.a = self
                            .cpu
                            .a
                            .checked_add(1)
                            .ok_or(SolveError::ArithmeticOverflow)?;
                    } else {
                        self.cpu.b = self
                            .cpu
                            .b
                            .checked_add(1)
                            .ok_or(SolveError::ArithmeticOverflow)?;
                    }
                    self.cpu.pc += 1;
                }
                Instruction::Jmp(offset) => {
                    self.cpu.jump(*offset)?;
                }
                Instruction::Jie(r, offset) => {
                    let value = if *r == Register::A {
                        self.cpu.a
                    } else {
                        self.cpu.b
                    };
                    if value % 2 == 0 {
                        self.cpu.jump(*offset)?;
                    } else {
                        self.cpu.pc += 1;
                    }
                }
                Instruction::Jio(r, offset) => {
                    let value = if *r == Register::A {
                        self.cpu.a
                    } else {
                        self.cpu.b
                    };
                    if value == 1 {
                        self.cpu.jump(*offset)?;
                    } else {
                        self.cpu.pc += 1;
                    }
                }
            }
        }
        Ok(steps)
    }
}

fn parse_register(text: &str) -> Result<Register, SolveError> {
    match text {
        "a" => Ok(Register::A),
        "b" => Ok(Register::B),
        _ => Err(SolveError::InvalidInput("expected register a or b".into())),
    }
}

fn parse_offset(text: &str) -> Result<i32, SolveError> {
    let bytes = text.as_bytes();
    if !matches!(bytes.first(), Some(b'+' | b'-'))
        || bytes.len() < 2
        || !bytes[1..].iter().all(u8::is_ascii_digit)
    {
        return Err(SolveError::InvalidInput(
            "expected a signed decimal jump offset".into(),
        ));
    }
    Ok(text.parse()?)
}

fn parse_instruction(line: &str) -> Result<Instruction, SolveError> {
    let mut words = line.split_ascii_whitespace();
    let (Some(opcode), Some(first), second, None) =
        (words.next(), words.next(), words.next(), words.next())
    else {
        return Err(SolveError::InvalidInput(
            "expected an instruction and its operands".into(),
        ));
    };
    match (opcode, second) {
        ("hlf", None) => Ok(Instruction::Hlf(parse_register(first)?)),
        ("tpl", None) => Ok(Instruction::Tpl(parse_register(first)?)),
        ("inc", None) => Ok(Instruction::Inc(parse_register(first)?)),
        ("jmp", None) => Ok(Instruction::Jmp(parse_offset(first)?)),
        ("jie" | "jio", Some(offset)) => {
            let register = first.strip_suffix(',').ok_or_else(|| {
                SolveError::InvalidInput("expected a comma after the register".into())
            })?;
            let register = parse_register(register)?;
            let offset = parse_offset(offset)?;
            Ok(if opcode == "jie" {
                Instruction::Jie(register, offset)
            } else {
                Instruction::Jio(register, offset)
            })
        }
        _ => Err(SolveError::InvalidInput(
            "unknown instruction or incorrect operands".into(),
        )),
    }
}

impl CPU {
    fn jump(&mut self, offset: i32) -> Result<(), SolveError> {
        if offset == 0 {
            return Err(SolveError::NoSolution("execution is stuck on a self-jump"));
        }
        self.pc = self
            .pc
            .checked_add(i64::from(offset))
            .ok_or(SolveError::ArithmeticOverflow)?;
        Ok(())
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let mut machine = VM::new();
    machine.parse(input)?;

    machine.run()?;

    Ok(machine.cpu.b.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut machine = VM::new();
    machine.parse(input)?;
    machine.cpu.a = 1;
    machine.run()?;

    Ok(machine.cpu.b.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2015;
    const DAY: i16 = 23;

    #[test]
    fn jumping_before_the_first_instruction_halts() {
        let mut machine = VM::new();
        machine.parse("inc b\njmp -3\ninc b").unwrap();
        machine.run().unwrap();
        assert_eq!(machine.cpu.pc, -2);
        assert_eq!(machine.cpu.b, 1);
    }

    #[test]
    fn jumping_past_the_last_instruction_halts() {
        let mut machine = VM::new();
        machine.parse("jmp +3\ninc b").unwrap();
        machine.run().unwrap();
        assert_eq!(machine.cpu.pc, 3);
        assert_eq!(machine.cpu.b, 0);
    }

    #[test]
    fn minimum_jump_offset_does_not_overflow() {
        let mut machine = VM::new();
        machine.parse("jmp -2147483648").unwrap();
        machine.run().unwrap();
        assert_eq!(machine.cpu.pc, i64::from(i32::MIN));
    }

    #[test]
    fn backward_jump_can_land_on_the_first_instruction() {
        let mut machine = VM::new();
        machine.parse("inc b\nhlf a\njie a, +2\njmp -3").unwrap();
        machine.cpu.a = 3;
        machine.run().unwrap();
        assert_eq!(machine.cpu.pc, 4);
        assert_eq!(machine.cpu.b, 2);
    }

    #[test]
    fn register_overflow_returns_an_error_without_wrapping() {
        for register in ["a", "b"] {
            for (operation, value) in [("inc", u64::MAX), ("tpl", u64::MAX / 3 + 1)] {
                let mut machine = VM::new();
                machine.parse(&format!("{operation} {register}")).unwrap();
                machine.cpu.a = value;
                machine.cpu.b = value;
                assert!(matches!(machine.run(), Err(SolveError::ArithmeticOverflow)));
                assert_eq!((machine.cpu.a, machine.cpu.b), (value, value));
                assert_eq!(machine.cpu.pc, 0);
            }
        }
    }

    #[test]
    fn register_operations_preserve_values_at_the_unsigned_boundary() {
        for register in ["a", "b"] {
            for (operation, before, after) in [
                ("inc", u64::MAX - 1, u64::MAX),
                ("tpl", u64::MAX / 3, u64::MAX),
                ("hlf", u64::MAX, u64::MAX / 2),
            ] {
                let mut machine = VM::new();
                machine.parse(&format!("{operation} {register}")).unwrap();
                machine.cpu.a = before;
                machine.cpu.b = before;
                machine.run().unwrap();
                let actual = if register == "a" {
                    machine.cpu.a
                } else {
                    machine.cpu.b
                };
                assert_eq!(actual, after);
            }
        }
    }

    #[test]
    fn malformed_instructions_return_their_line_number() {
        for invalid in [
            "",
            "wat a",
            "inc c",
            "hlf",
            "tpl a b",
            "inc a trailing",
            "jmp 2",
            "jmp +",
            "jmp --1",
            "jmp +1.5",
            "jmp +2147483648",
            "jmp -2147483649",
            "jmp +1 trailing",
            "jie a +2",
            "jio a,, -1",
            "jie c, +1",
            "jio a, 1",
            "jie a, +1 trailing",
            "inc a b c d",
        ] {
            let mut machine = VM::new();
            let input = format!("inc b\n{invalid}\ninc a");
            assert!(
                matches!(
                    machine.parse(&input),
                    Err(SolveError::AtLine { line: 2, .. })
                ),
                "{invalid:?}"
            );
            assert!(machine.program.is_empty());
            assert!(part1(&input).is_err());
            assert!(part2(&input).is_err());
        }
        assert!(matches!(part1(" \n"), Err(SolveError::EmptyInput)));
    }

    #[test]
    fn parser_accepts_all_opcodes_and_replaces_the_program() {
        let mut machine = VM::new();
        machine
            .parse("hlf a\ntpl b\ninc a\njmp +0\njie b, -2\njio a, +3")
            .unwrap();
        assert_eq!(machine.program.len(), 6);
        assert!(matches!(
            machine.program[4],
            Instruction::Jie(Register::B, -2)
        ));
        assert!(matches!(
            machine.program[5],
            Instruction::Jio(Register::A, 3)
        ));
        machine.parse("  inc\tb  ").unwrap();
        assert_eq!(machine.program.len(), 1);
        machine.run().unwrap();
        assert_eq!(machine.cpu.b, 1);
    }

    #[test]
    fn executed_self_jumps_are_semantic_errors() {
        for (program, a) in [("jmp +0", 0), ("jie a, +0", 2), ("jio a, -0", 1)] {
            let mut machine = VM::new();
            machine.parse(program).unwrap();
            machine.cpu.a = a;
            assert!(matches!(machine.run(), Err(SolveError::NoSolution(_))));
            assert_eq!(machine.cpu.pc, 0);
        }
    }

    #[test]
    fn unreachable_and_untaken_self_jumps_are_valid() {
        for (program, a) in [("jmp +2\njmp +0", 0), ("jie a, +0", 1), ("jio a, +0", 0)] {
            let mut machine = VM::new();
            machine.parse(program).unwrap();
            machine.cpu.a = a;
            machine.run().unwrap();
        }
    }

    #[test]
    fn instruction_budget_bounds_longer_cycles() {
        let mut machine = VM::new();
        machine.parse("inc b\njmp -1").unwrap();
        assert!(matches!(
            machine.run_with_limit(6),
            Err(SolveError::ResourceLimit(_))
        ));
        assert_eq!(machine.cpu.b, 3);
        assert_eq!(machine.cpu.pc, 0);
    }

    #[test]
    fn normal_halt_at_the_exact_budget_is_allowed() {
        let mut machine = VM::new();
        machine.parse("inc b\ninc b").unwrap();
        assert_eq!(machine.run_with_limit(2).unwrap(), 2);
        assert_eq!(machine.cpu.b, 2);
        let mut machine = VM::new();
        machine.parse("inc b").unwrap();
        assert!(matches!(
            machine.run_with_limit(0),
            Err(SolveError::ResourceLimit(_))
        ));
        assert_eq!(machine.cpu.b, 0);
    }

    #[test]
    fn conditional_jumps_test_evenness_and_exactly_one() {
        for register in ["a", "b"] {
            for value in 0..=5 {
                for (opcode, taken) in [("jie", value % 2 == 0), ("jio", value == 1)] {
                    let mut machine = VM::new();
                    machine
                        .parse(&format!("{opcode} {register}, +2\ninc {register}"))
                        .unwrap();
                    machine.cpu.a = value;
                    machine.cpu.b = value;
                    machine.run().unwrap();
                    let actual = if register == "a" {
                        machine.cpu.a
                    } else {
                        machine.cpu.b
                    };
                    assert_eq!(actual, value + u64::from(!taken));
                }
            }
        }
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
