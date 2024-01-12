extern crate core;

fn parse_line(line: &str) -> Vec<Instruction> {
    line
        .chars()
        .enumerate()
        .map(|s| {
            match s.1 {
                'U' => Instruction::Up,
                'R' => Instruction::Right,
                'D' => Instruction::Down,
                'L' => Instruction::Left,
                _ => panic!("Invalid direction: {}", s.1),
            }
        })
        .collect()
}

fn parse(input: &str) -> Vec<Vec<Instruction>> {
    input
        .lines()
        .map(|line| parse_line(line))
        .collect()
}

#[derive(Debug)]
enum Instruction {
    Up,
    Right,
    Down,
    Left,
}

impl Instruction {
    fn follow(&self, cur_pos: Pos) -> Pos {
        match self {
            Instruction::Up => cur_pos.go_up(),
            Instruction::Right => cur_pos.go_right(),
            Instruction::Down => cur_pos.go_down(),
            Instruction::Left => cur_pos.go_left(),
        }
    }

    fn follow_enhanced(&self, cur_pos: PosEnhanced) -> PosEnhanced {
        match self {
            Instruction::Up => cur_pos.go_up(),
            Instruction::Right => cur_pos.go_right(),
            Instruction::Down => cur_pos.go_down(),
            Instruction::Left => cur_pos.go_left(),
        }
    }
}

#[derive(Debug, Clone)]
struct Pos {
    x: i64,
    y: i64,
}

impl Pos {
    fn new(x: i64, y: i64) -> Pos {
        Pos { x, y }
    }

    fn number_representation(self) -> i64 {

        // 1 2 3
        // 4 5 6
        // 7 8 9

        (2 - self.y) * 3 + self.x + 1
    }

    fn go_up(self) -> Pos {
        if self.y + 1 > 2 {
            return self.clone();
        }

        Pos::new(self.x, self.y + 1)
    }

    fn go_right(self) -> Pos {
        if self.x + 1 > 2 {
            return self.clone();
        }

        Pos::new(self.x + 1, self.y)
    }

    fn go_down(self) -> Pos {
        if self.y - 1 < 0 {
            return self.clone();
        }

        Pos::new(self.x, self.y - 1)
    }

    fn go_left(self) -> Pos {
        if self.x - 1 < 0 {
            return self.clone();
        }

        Pos::new(self.x - 1, self.y)
    }
}

#[derive(Debug, Clone)]
struct PosEnhanced {
    x: i64,
    y: i64,
}

impl PosEnhanced {
    fn new(x: i64, y: i64) -> PosEnhanced {
        PosEnhanced { x, y }
    }

    fn representation(self) -> String {
        //       1
        //     2 3 4
        //   5 6 7 8 9
        //     A B C
        //       D

        match (self.x, self.y) {
            (2, 0) => "D".to_string(),
            (1, 1) => "A".to_string(),
            (2, 1) => "B".to_string(),
            (3, 1) => "C".to_string(),
            (0, 2) => "5".to_string(),
            (1, 2) => "6".to_string(),
            (2, 2) => "7".to_string(),
            (3, 2) => "8".to_string(),
            (4, 2) => "9".to_string(),
            (1, 3) => "2".to_string(),
            (2, 3) => "3".to_string(),
            (3, 3) => "4".to_string(),
            (2, 4) => "1".to_string(),
            _ => panic!("Invalid position: {:?}", self),
        }
    }

    fn go_up(self) -> PosEnhanced {
        let up = PosEnhanced::new(self.x, self.y + 1);

        match (self.x, self.y) {
            (2, 0) => up,               // D
            (1, 1) => up,               // A
            (2, 1) => up,               // B
            (3, 1) => up,               // C
            (1, 2) => up,               // 6
            (2, 2) => up,               // 7
            (3, 2) => up,               // 8
            (2, 3) => up,               // 3
            _ => self.clone(), // Not a legal move, so just return ourself
        }
    }

    fn go_right(self) -> PosEnhanced {
        let right = PosEnhanced::new(self.x + 1, self.y);

        match (self.x, self.y) {
            (1, 1) => right,            // A
            (2, 1) => right,            // B
            (0, 2) => right,            // 5
            (1, 2) => right,            // 6
            (2, 2) => right,            // 7
            (3, 2) => right,            // 8
            (1, 3) => right,            // 2
            (2, 3) => right,            // 3
            _ => self.clone(), // Not a legal move, so just return ourself
        }
    }

    fn go_down(self) -> PosEnhanced {
        let down = PosEnhanced::new(self.x, self.y - 1);

        match (self.x, self.y) {
            (2, 1) => down,             // B
            (1, 2) => down,             // 6
            (2, 2) => down,             // 7
            (3, 2) => down,             // 8
            (1, 3) => down,             // 2
            (2, 3) => down,             // 3
            (3, 3) => down,             // 4
            (2, 4) => down,             // 1
            _ => self.clone(), // Not a legal move, so just return ourself
        }
    }

    fn go_left(self) -> PosEnhanced {
        let left = PosEnhanced::new(self.x - 1, self.y);

        match (self.x, self.y) {
            (2, 1) => left,             // B
            (3, 1) => left,             // C
            (1, 2) => left,             // 6
            (2, 2) => left,             // 7
            (3, 2) => left,             // 8
            (4, 2) => left,             // 9
            (2, 3) => left,             // 3
            (3, 3) => left,             // 4
            _ => self.clone(), // Not a legal move, so just return ourself
        }
    }
}

pub fn fn1(input: &str) -> i64 {
    let instrs = parse(input);

    // Start on the 5
    let mut pos: Pos = Pos::new(1, 1);

    let mut code = Vec::new();

    instrs.into_iter().for_each(|line| {
        for instr in line {
            pos = instr.follow(pos.clone());
        }

        code.push(pos.clone().number_representation());
    });

    let mut door_code = 0i64;

    for n in &code {
        door_code = door_code * 10 + n;
    }

    door_code
}

pub fn fn2(input: &str) -> String {
    let instrs = parse(input);

    // Start on the 5
    let mut pos: PosEnhanced = PosEnhanced::new(0, 2);

    let mut code = Vec::new();

    instrs.into_iter().for_each(|line| {
        for instr in line {
            pos = instr.follow_enhanced(pos.clone());
        }

        code.push(pos.clone().representation());
    });

    let mut door_code = String::new();

    for n in &code {
        door_code.push_str(n);
    }

    door_code
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;
    use svutils::scaffold_test_string;

    const YEAR: i16 = 2016;
    const DAY: i16 = 2;

    #[test]
    fn test_number_reprs() {
        assert_eq!(Pos::new(0, 2).number_representation(), 1);
        assert_eq!(Pos::new(1, 2).number_representation(), 2);
        assert_eq!(Pos::new(2, 2).number_representation(), 3);
        assert_eq!(Pos::new(0, 1).number_representation(), 4);
        assert_eq!(Pos::new(1, 1).number_representation(), 5);
        assert_eq!(Pos::new(2, 1).number_representation(), 6);
        assert_eq!(Pos::new(0, 0).number_representation(), 7);
        assert_eq!(Pos::new(1, 0).number_representation(), 8);
        assert_eq!(Pos::new(2, 0).number_representation(), 9);
    }

    #[test]
    fn test_string_reprs() {
        assert_eq!(PosEnhanced::new(2, 0).representation(), "D");
        assert_eq!(PosEnhanced::new(1, 1).representation(), "A");
        assert_eq!(PosEnhanced::new(2, 1).representation(), "B");
        assert_eq!(PosEnhanced::new(3, 1).representation(), "C");
        assert_eq!(PosEnhanced::new(0, 2).representation(), "5");
        assert_eq!(PosEnhanced::new(1, 2).representation(), "6");
        assert_eq!(PosEnhanced::new(2, 2).representation(), "7");
        assert_eq!(PosEnhanced::new(3, 2).representation(), "8");
        assert_eq!(PosEnhanced::new(4, 2).representation(), "9");
        assert_eq!(PosEnhanced::new(1, 3).representation(), "2");
        assert_eq!(PosEnhanced::new(2, 3).representation(), "3");
        assert_eq!(PosEnhanced::new(3, 3).representation(), "4");
        assert_eq!(PosEnhanced::new(2, 4).representation(), "1");
    }

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test_string(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test_string(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
