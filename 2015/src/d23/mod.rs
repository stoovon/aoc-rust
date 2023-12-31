extern crate core;

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
    a: i32,
    b: i32,
    pc: usize,
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

    fn reg(s: &str) -> Register {
        match s {
            "a" => Register::A,
            "b" => Register::B,
            _ => panic!("input file corrupted"),
        }
    }

    fn parse(&mut self, input: &str) {
        for line in input.lines() {
            let (mnemonic, rest) = line.split_once(' ').unwrap();
            let (reg, offset) = rest.split_once(", ").unwrap_or(("", ""));
            let inst = match mnemonic {
                "hlf" => Instruction::Hlf(Self::reg(rest)),
                "tpl" => Instruction::Tpl(Self::reg(rest)),
                "inc" => Instruction::Inc(Self::reg(rest)),
                "jmp" => Instruction::Jmp(rest.parse().unwrap()),
                "jie" => Instruction::Jie(Self::reg(reg), offset.parse().unwrap()),
                "jio" => Instruction::Jio(Self::reg(reg), offset.parse().unwrap()),
                _ => {
                    panic!("input file corrupted");
                }
            };

            self.program.push(inst)
        }
    }

    fn run(&mut self) {
        while self.cpu.pc < self.program.len() {
            match &self.program[self.cpu.pc] {
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
                        self.cpu.a *= 3;
                    } else {
                        self.cpu.b *= 3;
                    }
                    self.cpu.pc += 1;
                }
                Instruction::Inc(r) => {
                    if *r == Register::A {
                        self.cpu.a += 1;
                    } else {
                        self.cpu.b += 1;
                    }
                    self.cpu.pc += 1;
                }
                Instruction::Jmp(offset) => {
                    self.cpu.jump(*offset);
                }
                Instruction::Jie(r, offset) => {
                    let value = if *r == Register::A {
                        self.cpu.a
                    } else {
                        self.cpu.b
                    };
                    if value % 2 == 0 {
                        self.cpu.jump(*offset);
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
                        self.cpu.jump(*offset);
                    } else {
                        self.cpu.pc += 1;
                    }
                }
            }
        }
    }
}

impl CPU {
    fn jump(&mut self, offset: i32) {
        if offset < 0 {
            self.pc -= (-offset) as usize;
        } else {
            self.pc += offset as usize;
        }
    }
}

pub fn fn1(input: &str) -> i64 {
    let mut machine = VM::new();
    machine.parse(input);

    machine.run();

    machine.cpu.b as i64
}

pub fn fn2(input: &str) -> i64 {
    let mut machine = VM::new();
    machine.parse(input);
    machine.cpu.a = 1;
    machine.run();

    machine.cpu.b as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 23;

    // #[test]
    // fn test_fn1_example() {
    //     scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    // }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    // #[test]
    // #[ignore]
    // fn test_fn2_example() {
    //     scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    // }

    #[test]
    #[ignore]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
