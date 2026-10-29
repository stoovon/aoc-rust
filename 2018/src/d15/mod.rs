use std::collections::{HashSet, VecDeque};
use svutils::SolveError;

const MAX_MAP_CELLS: usize = 1 << 20;
const MAX_QUIET_STATE_CELLS: usize = 1 << 20;
const MAX_BATTLE_WORK: usize = 100_000_000;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Pos {
    row: isize,
    col: isize,
}

impl Pos {
    fn new(row: isize, col: isize) -> Pos {
        Pos { row, col }
    }

    fn get_index(self, width: usize) -> usize {
        self.row as usize * width + self.col as usize
    }

    fn adjacency_iter(self, width: usize, height: usize) -> impl Iterator<Item = Self> {
        [
            Self::new(self.row - 1, self.col),
            Self::new(self.row, self.col - 1),
            Self::new(self.row, self.col + 1),
            Self::new(self.row + 1, self.col),
        ]
        .into_iter()
        .filter(move |pos| {
            pos.row >= 0 && pos.col >= 0 && pos.row < height as isize && pos.col < width as isize
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Team {
    Elf,
    Goblin,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct Unit {
    team: Team,
    health: u16,
    pos: Pos,
}

struct Pathfinder {
    distances: Vec<usize>,
    first_steps: Vec<Pos>,
    targets: Vec<bool>,
    queue: VecDeque<Pos>,
}

impl Pathfinder {
    fn new(area: usize) -> Self {
        Self {
            distances: vec![usize::MAX; area],
            first_steps: vec![Pos::new(0, 0); area],
            targets: vec![false; area],
            queue: VecDeque::new(),
        }
    }

    fn next_step(
        &mut self,
        unit: Unit,
        units: &[Unit],
        occupied: &[Option<usize>],
        arena: &Arena,
    ) -> Option<Pos> {
        if unit
            .pos
            .adjacency_iter(arena.width, arena.height)
            .filter_map(|pos| occupied[pos.get_index(arena.width)])
            .any(|id| units[id].team != unit.team)
        {
            return None;
        }
        self.targets.fill(false);
        let mut has_targets = false;
        for enemy in units
            .iter()
            .filter(|enemy| enemy.health > 0 && enemy.team != unit.team)
        {
            for pos in enemy.pos.adjacency_iter(arena.width, arena.height) {
                let index = pos.get_index(arena.width);
                if !arena.walls[index] && occupied[index].is_none() {
                    self.targets[index] = true;
                    has_targets = true;
                }
            }
        }
        if !has_targets {
            return None;
        }

        self.distances.fill(usize::MAX);
        self.queue.clear();
        self.distances[unit.pos.get_index(arena.width)] = 0;
        self.queue.push_back(unit.pos);
        let mut nearest_distance = usize::MAX;
        let mut destination: Option<usize> = None;

        while let Some(pos) = self.queue.pop_front() {
            let index = pos.get_index(arena.width);
            let distance = self.distances[index];
            if distance > nearest_distance {
                break;
            }
            if self.targets[index] {
                nearest_distance = distance;
                destination = Some(destination.map_or(index, |best| best.min(index)));
                continue;
            }
            if distance == nearest_distance {
                continue;
            }
            for next in pos.adjacency_iter(arena.width, arena.height) {
                let next_index = next.get_index(arena.width);
                if arena.walls[next_index]
                    || occupied[next_index].is_some()
                    || self.distances[next_index] != usize::MAX
                {
                    continue;
                }
                self.distances[next_index] = distance + 1;
                // FIFO layers and reading-ordered neighbours make first discovery
                // retain the earliest first step among shortest routes to this cell.
                self.first_steps[next_index] = if pos == unit.pos {
                    next
                } else {
                    self.first_steps[index]
                };
                self.queue.push_back(next);
            }
        }
        // Destination order takes priority over first-step order: finish the nearest layer.
        destination.map(|index| self.first_steps[index])
    }
}

struct Arena {
    width: usize,
    height: usize,
    walls: Vec<bool>,
    initial_units: Vec<Unit>,
}

impl Arena {
    fn parse(input: &str) -> Result<Self, SolveError> {
        let width = input.lines().next().ok_or(SolveError::EmptyInput)?.len();
        if width == 0 {
            return Err(SolveError::EmptyInput);
        }
        let height = input.lines().count();
        let area = width
            .checked_mul(height)
            .ok_or(SolveError::ArithmeticOverflow)?;
        if width > isize::MAX as usize || height > isize::MAX as usize {
            return Err(SolveError::ArithmeticOverflow);
        }
        if area > MAX_MAP_CELLS {
            return Err(SolveError::InvalidInput(
                "map exceeds the supported cell budget".into(),
            ));
        }
        let mut walls = Vec::with_capacity(area);
        let mut initial_units = Vec::new();
        let mut elves = 0;
        let mut goblins = 0;
        for (row, line) in input.lines().enumerate() {
            if line.len() != width {
                return Err(
                    SolveError::InvalidInput("map rows must have equal widths".into())
                        .at_line(row + 1),
                );
            }
            for (col, cell) in line.bytes().enumerate() {
                match cell {
                    b'.' => walls.push(false),
                    b'#' => walls.push(true),
                    b'E' | b'G' => {
                        walls.push(false);
                        let pos = Pos::new(row as isize, col as isize);
                        let unit = Unit {
                            team: if cell == b'E' {
                                Team::Elf
                            } else {
                                Team::Goblin
                            },
                            health: 200,
                            pos,
                        };
                        if unit.team == Team::Elf {
                            elves += 1;
                        } else {
                            goblins += 1;
                        }
                        initial_units.push(unit);
                    }
                    _ => {
                        return Err(SolveError::InvalidInput(
                            "map cells must be '.', '#', 'E', or 'G'".into(),
                        )
                        .at_line(row + 1))
                    }
                }
            }
        }
        if elves == 0 || goblins == 0 {
            return Err(SolveError::InvalidInput(
                "map must contain both elves and goblins".into(),
            ));
        }
        Ok(Self {
            width,
            height,
            walls,
            initial_units,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RoundStatus {
    Completed { moved: bool, attacked: bool },
    CombatEnded,
    ElfDied,
}

#[derive(Debug, PartialEq, Eq)]
enum BattleEnd {
    Finished,
    ElfDied,
    Stalemate,
}

struct Battle<'a> {
    arena: &'a Arena,
    units: Vec<Unit>,
    occupied: Vec<Option<usize>>,
    turn_order: Vec<usize>,
    pathfinder: Pathfinder,
    completed_rounds: i64,
    dead_elves: usize,
    elves: usize,
    goblins: usize,
    work_remaining: usize,
}

impl<'a> Battle<'a> {
    fn new(arena: &'a Arena) -> Self {
        let units = arena.initial_units.clone();
        let mut occupied = vec![None; arena.walls.len()];
        let mut elves = 0;
        for (id, unit) in units.iter().enumerate() {
            occupied[unit.pos.get_index(arena.width)] = Some(id);
            if unit.team == Team::Elf {
                elves += 1;
            }
        }
        let goblins = units.len() - elves;
        let turn_order = Vec::with_capacity(units.len());
        Self {
            arena,
            units,
            occupied,
            turn_order,
            pathfinder: Pathfinder::new(arena.walls.len()),
            completed_rounds: 0,
            dead_elves: 0,
            elves,
            goblins,
            work_remaining: MAX_BATTLE_WORK,
        }
    }

    fn adjacent_enemy(&self, id: usize) -> Option<usize> {
        let unit = self.units[id];
        unit.pos
            .adjacency_iter(self.arena.width, self.arena.height)
            .filter_map(|pos| self.occupied[pos.get_index(self.arena.width)])
            .filter(|&enemy| self.units[enemy].team != unit.team)
            .min_by_key(|&enemy| {
                let target = self.units[enemy];
                (target.health, target.pos.get_index(self.arena.width))
            })
    }

    fn attack(&mut self, attacker: usize, target: usize, elf_power: u16) {
        let damage = if self.units[attacker].team == Team::Elf {
            elf_power
        } else {
            3
        };
        let enemy = &mut self.units[target];
        enemy.health = enemy.health.saturating_sub(damage);
        if enemy.health == 0 {
            self.occupied[enemy.pos.get_index(self.arena.width)] = None;
            if enemy.team == Team::Elf {
                self.elves -= 1;
                self.dead_elves += 1;
            } else {
                self.goblins -= 1;
            }
        }
    }

    fn round(
        &mut self,
        elf_power: u16,
        abort_on_elf_death: bool,
    ) -> Result<RoundStatus, SolveError> {
        self.spend_work(self.units.len())?;
        let mut moved = false;
        let mut attacked = false;
        self.turn_order.clear();
        self.turn_order.extend(
            self.units
                .iter()
                .enumerate()
                .filter(|(_, unit)| unit.health > 0)
                .map(|(id, _)| id),
        );
        self.turn_order
            .sort_unstable_by_key(|&id| self.units[id].pos.get_index(self.arena.width));

        // IDs stay in round-start order even when units move or their squares are reused.
        for turn in 0..self.turn_order.len() {
            self.spend_work(1)?;
            let id = self.turn_order[turn];
            if self.units[id].health == 0 {
                continue;
            }
            if self.elves == 0 || self.goblins == 0 {
                return Ok(RoundStatus::CombatEnded);
            }
            if self.adjacent_enemy(id).is_none() {
                self.spend_work(self.arena.walls.len())?;
                let unit = self.units[id];
                if let Some(pos) =
                    self.pathfinder
                        .next_step(unit, &self.units, &self.occupied, self.arena)
                {
                    self.occupied[unit.pos.get_index(self.arena.width)] = None;
                    self.occupied[pos.get_index(self.arena.width)] = Some(id);
                    self.units[id].pos = pos;
                    moved = true;
                }
            }
            if let Some(target) = self.adjacent_enemy(id) {
                self.attack(id, target, elf_power);
                attacked = true;
                if abort_on_elf_death && self.dead_elves > 0 {
                    return Ok(RoundStatus::ElfDied);
                }
            }
        }
        self.completed_rounds = self
            .completed_rounds
            .checked_add(1)
            .ok_or(SolveError::ArithmeticOverflow)?;
        Ok(RoundStatus::Completed { moved, attacked })
    }

    fn spend_work(&mut self, amount: usize) -> Result<(), SolveError> {
        self.work_remaining = self.work_remaining.checked_sub(amount).ok_or_else(|| {
            SolveError::InvalidInput("combat exceeds the supported work budget".into())
        })?;
        Ok(())
    }

    fn fight(&mut self, elf_power: u16, abort_on_elf_death: bool) -> Result<BattleEnd, SolveError> {
        if elf_power == 0 {
            return Err(SolveError::InvalidInput(
                "attack power must be positive".into(),
            ));
        }
        let mut quiet_states = HashSet::new();
        let mut state_cells = 0;
        while self.elves > 0 && self.goblins > 0 {
            match self.round(elf_power, abort_on_elf_death)? {
                RoundStatus::CombatEnded => return Ok(BattleEnd::Finished),
                RoundStatus::ElfDied => return Ok(BattleEnd::ElfDied),
                RoundStatus::Completed { attacked: true, .. } => {
                    // Every attack reduces HP. Between attacks, HP and the roster
                    // are fixed, so positions by stable ID suffice for exact cycles.
                    quiet_states.clear();
                    state_cells = 0;
                }
                RoundStatus::Completed {
                    moved: false,
                    attacked: false,
                } => {
                    return Ok(BattleEnd::Stalemate);
                }
                RoundStatus::Completed {
                    moved: true,
                    attacked: false,
                } => {
                    if self.units.len() > MAX_QUIET_STATE_CELLS - state_cells {
                        return Err(SolveError::InvalidInput(
                            "combat exceeds the supported cycle-state budget".into(),
                        ));
                    }
                    let positions: Vec<usize> = self
                        .units
                        .iter()
                        .map(|unit| {
                            if unit.health == 0 {
                                usize::MAX
                            } else {
                                unit.pos.get_index(self.arena.width)
                            }
                        })
                        .collect();
                    if !quiet_states.insert(positions) {
                        return Ok(BattleEnd::Stalemate);
                    }
                    state_cells += self.units.len();
                }
            }
        }
        Ok(BattleEnd::Finished)
    }

    fn score(&self) -> Result<i64, SolveError> {
        let health = self.units.iter().try_fold(0i64, |sum, unit| {
            sum.checked_add(i64::from(unit.health))
                .ok_or(SolveError::ArithmeticOverflow)
        })?;
        health
            .checked_mul(self.completed_rounds)
            .ok_or(SolveError::ArithmeticOverflow)
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let arena = Arena::parse(input)?;
    let mut battle = Battle::new(&arena);
    if battle.fight(3, false)? == BattleEnd::Stalemate {
        return Err(SolveError::InvalidInput(
            "combat ends in a stalemate".into(),
        ));
    }
    Ok(battle.score()?.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let arena = Arena::parse(input)?;
    // At 200 damage every goblin dies in one hit; higher powers have identical behaviour.
    for elf_power in 3..=200 {
        let mut battle = Battle::new(&arena);
        if battle.fight(elf_power, true)? == BattleEnd::Finished
            && battle.goblins == 0
            && battle.dead_elves == 0
        {
            return Ok(battle.score()?.to_string());
        }
    }
    Err(SolveError::InvalidInput(
        "no elf victory without casualties is possible".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2018;
    const DAY: i16 = 15;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
    #[test]
    fn parser_requires_a_rectangular_map_with_both_factions() {
        for input in ["", "\n", "EG\n#", "E?G", "EéG", "E", "G", "..."] {
            assert!(Arena::parse(input).is_err(), "{input:?}");
        }
        assert!(Arena::parse("E.G\r\n###\r\n").is_ok());
        let error = Arena::parse("E.G\n#?#").err().unwrap();
        assert!(error.to_string().starts_with("line 2:"));
    }

    #[test]
    fn parser_preserves_large_coordinates_and_unit_ids() {
        let input = format!("{}G", "E".repeat(256));
        let arena = Arena::parse(&input).unwrap();
        assert_eq!(arena.width, 257);
        assert_eq!(arena.initial_units.len(), 257);
        assert_eq!(arena.initial_units[256].pos.get_index(arena.width), 256);
        assert_eq!(arena.initial_units[256].health, 200);
    }

    #[test]
    fn attempts_borrow_the_same_arena_but_own_independent_state() {
        let arena = Arena::parse("E.G").unwrap();
        let mut first = Battle::new(&arena);
        let second = Battle::new(&arena);
        assert!(std::ptr::eq(first.arena, second.arena));
        first.units[0].health = 0;
        first.occupied[0] = None;
        assert_eq!(second.units[0].health, 200);
        assert_eq!(second.occupied[0], Some(0));
        assert_eq!(arena.initial_units[0].health, 200);
    }
    #[test]
    fn final_round_counts_only_when_all_remaining_turns_are_dead() {
        let arena = Arena::parse("EG").unwrap();
        let mut battle = Battle::new(&arena);
        battle.units[1].health = 3;
        assert_eq!(
            battle.round(3, false).unwrap(),
            RoundStatus::Completed {
                moved: false,
                attacked: true
            }
        );
        assert_eq!(battle.completed_rounds, 1);
        assert_eq!(battle.score().unwrap(), 200);

        let arena = Arena::parse("EGE").unwrap();
        let mut battle = Battle::new(&arena);
        battle.units[1].health = 3;
        assert_eq!(battle.round(3, false).unwrap(), RoundStatus::CombatEnded);
        assert_eq!(battle.completed_rounds, 0);
    }

    #[test]
    fn a_reoccupied_square_does_not_give_a_unit_the_dead_units_turn() {
        let arena = Arena::parse("EGG").unwrap();
        let mut battle = Battle::new(&arena);
        assert_eq!(
            battle.round(200, false).unwrap(),
            RoundStatus::Completed {
                moved: true,
                attacked: true
            }
        );
        assert_eq!(battle.units[0].health, 197);
        assert_eq!(battle.units[1].health, 0);
        assert_eq!(battle.units[2].pos.get_index(arena.width), 1);
        assert_eq!(battle.occupied[1], Some(2));
    }

    #[test]
    fn movement_and_attacks_happen_once_per_round() {
        let arena = Arena::parse("G..E").unwrap();
        let mut battle = Battle::new(&arena);
        battle.round(3, false).unwrap();
        assert_eq!(battle.units[0].pos.get_index(arena.width), 1);
        assert_eq!(battle.units[1].pos.get_index(arena.width), 2);
        assert_eq!(battle.units[0].health, 197);
        assert_eq!(battle.units[1].health, 200);
        battle.round(3, false).unwrap();
        assert_eq!(battle.units[0].health, 194);
        assert_eq!(battle.units[1].health, 197);
    }

    #[test]
    fn elf_loss_aborts_before_later_turns() {
        let arena = Arena::parse("GEG").unwrap();
        let mut battle = Battle::new(&arena);
        battle.units[1].health = 3;
        assert_eq!(battle.round(3, true).unwrap(), RoundStatus::ElfDied);
        assert_eq!(battle.completed_rounds, 0);
        assert_eq!(battle.dead_elves, 1);
        assert_eq!(battle.occupied[1], None);
    }
    fn reference_step(
        arena: &Arena,
        units: &[Unit],
        occupied: &[Option<usize>],
        id: usize,
    ) -> Option<usize> {
        let unit = units[id];
        let origin = unit.pos.get_index(arena.width);
        let neighbours = |index: usize| {
            let row = (index / arena.width) as isize;
            let col = (index % arena.width) as isize;
            [(-1, 0), (0, -1), (0, 1), (1, 0)]
                .into_iter()
                .filter_map(move |(dr, dc)| {
                    let r = row + dr;
                    let c = col + dc;
                    (r >= 0 && c >= 0 && r < arena.height as isize && c < arena.width as isize)
                        .then(|| r as usize * arena.width + c as usize)
                })
        };
        let enemy_next_to = |index: usize| {
            neighbours(index)
                .filter_map(|next| occupied[next])
                .any(|enemy| units[enemy].team != unit.team)
        };
        if enemy_next_to(origin) {
            return None;
        }
        let distances_from = |start| {
            let mut distances = vec![usize::MAX; arena.walls.len()];
            let mut queue = VecDeque::from([start]);
            distances[start] = 0;
            while let Some(pos) = queue.pop_front() {
                for next in neighbours(pos) {
                    if arena.walls[next]
                        || (occupied[next].is_some() && next != origin)
                        || distances[next] != usize::MAX
                    {
                        continue;
                    }
                    distances[next] = distances[pos] + 1;
                    queue.push_back(next);
                }
            }
            distances
        };
        // Independently pick the destination, then search backwards to choose a first step.
        let outward = distances_from(origin);
        let destination = (0..arena.walls.len())
            .filter(|&index| {
                !arena.walls[index]
                    && occupied[index].is_none()
                    && outward[index] != usize::MAX
                    && enemy_next_to(index)
            })
            .min_by_key(|&index| (outward[index], index))?;
        let backward = distances_from(destination);
        neighbours(origin)
            .filter(|&index| {
                backward[index] != usize::MAX && backward[index] + 1 == outward[destination]
            })
            .min()
    }

    #[test]
    fn movement_matches_two_search_reference_on_every_small_map() {
        let mut pathfinder = Pathfinder::new(9);
        let mut comparisons = 0;
        for elf in 0..9 {
            for goblin in 0..9 {
                if elf == goblin {
                    continue;
                }
                let spare: Vec<_> = (0..9).filter(|&pos| pos != elf && pos != goblin).collect();
                for walls in 0usize..1 << spare.len() {
                    let mut cells = [b'.'; 9];
                    cells[elf] = b'E';
                    cells[goblin] = b'G';
                    for (bit, &pos) in spare.iter().enumerate() {
                        if walls & (1 << bit) != 0 {
                            cells[pos] = b'#';
                        }
                    }
                    let input = cells
                        .chunks(3)
                        .map(|row| std::str::from_utf8(row).unwrap())
                        .collect::<Vec<_>>()
                        .join("\n");
                    let arena = Arena::parse(&input).unwrap();
                    let battle = Battle::new(&arena);
                    // Both teams exercise the same search, with scratch reused across changing maps.
                    for id in 0..2 {
                        let expected = reference_step(&arena, &battle.units, &battle.occupied, id);
                        let actual = pathfinder
                            .next_step(battle.units[id], &battle.units, &battle.occupied, &arena)
                            .map(|pos| pos.get_index(arena.width));
                        assert_eq!(actual, expected, "map={input:?}, unit={id}");
                        comparisons += 1;
                    }
                }
            }
        }
        assert_eq!(comparisons, 18_432);
    }

    #[test]
    fn multiple_enemies_and_dead_units_match_reference() {
        let mut pathfinder = Pathfinder::new(9);
        for elf in 0..9 {
            for first in 0..9 {
                for second in first + 1..9 {
                    if elf == first || elf == second {
                        continue;
                    }
                    let spare: Vec<_> = (0..9)
                        .filter(|&pos| pos != elf && pos != first && pos != second)
                        .collect();
                    for walls in 0usize..1 << spare.len() {
                        let mut cells = [b'.'; 9];
                        cells[elf] = b'E';
                        cells[first] = b'G';
                        cells[second] = b'G';
                        for (bit, &pos) in spare.iter().enumerate() {
                            if walls & (1 << bit) != 0 {
                                cells[pos] = b'#';
                            }
                        }
                        let input = cells
                            .chunks(3)
                            .map(|row| std::str::from_utf8(row).unwrap())
                            .collect::<Vec<_>>()
                            .join("\n");
                        let arena = Arena::parse(&input).unwrap();
                        let mut battle = Battle::new(&arena);
                        let elf_id = battle.occupied[elf].unwrap();
                        for dead_goblin in [None, battle.occupied[first]] {
                            if let Some(id) = dead_goblin {
                                battle.units[id].health = 0;
                                battle.occupied[first] = None;
                            }
                            let expected =
                                reference_step(&arena, &battle.units, &battle.occupied, elf_id);
                            let actual = pathfinder
                                .next_step(
                                    battle.units[elf_id],
                                    &battle.units,
                                    &battle.occupied,
                                    &arena,
                                )
                                .map(|pos| pos.get_index(arena.width));
                            assert_eq!(actual, expected, "map={input:?}");
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn destination_order_precedes_first_step_order() {
        let arena = Arena::parse("..#..\n..E..\n.....\n....G\n.G.#.").unwrap();
        let mut battle = Battle::new(&arena);
        let elf = battle.occupied[7].unwrap();
        // Left discovers a distance-3 target first, but right leads to the earlier destination.
        let step = battle
            .pathfinder
            .next_step(battle.units[elf], &battle.units, &battle.occupied, &arena)
            .unwrap();
        assert_eq!(step.get_index(arena.width), 8);

        let arena = Arena::parse("....G\n.....\n..E..\n.....\n.....").unwrap();
        let mut battle = Battle::new(&arena);
        let elf = battle.occupied[12].unwrap();
        let step = battle
            .pathfinder
            .next_step(battle.units[elf], &battle.units, &battle.occupied, &arena)
            .unwrap();
        assert_eq!(step.get_index(arena.width), 7);
    }
    #[test]
    fn parts_apply_their_own_attack_power_and_casualty_rules() {
        assert_eq!(part1("GE").unwrap(), "134");
        assert_eq!(part2("GE").unwrap(), "2500");
        assert_eq!(part1("EG").unwrap(), "134");
        assert_eq!(part2("EG").unwrap(), "134");
    }
    #[test]
    fn stalemates_and_impossible_elf_victories_are_errors() {
        assert!(part1("E#G").unwrap_err().to_string().contains("stalemate"));
        assert!(part2("E#G")
            .unwrap_err()
            .to_string()
            .contains("no elf victory"));
        let army = "GGGGGGG\nGGGGGGG\nGGGGGGG\nGGGEGGG\nGGGGGGG\nGGGGGGG\nGGGGGGG";
        assert!(part2(army)
            .unwrap_err()
            .to_string()
            .contains("no elf victory"));
    }

    #[test]
    fn attack_targets_lowest_health_then_reading_order() {
        let arena = Arena::parse(".G.\nGEG\n.G.").unwrap();
        let mut battle = Battle::new(&arena);
        let elf = battle.occupied[4].unwrap();
        assert_eq!(battle.adjacent_enemy(elf), battle.occupied[1]);
        let left = battle.occupied[3].unwrap();
        let right = battle.occupied[5].unwrap();
        battle.units[left].health = 10;
        battle.units[right].health = 10;
        assert_eq!(battle.adjacent_enemy(elf), Some(left));
        battle.units[right].health = 9;
        assert_eq!(battle.adjacent_enemy(elf), Some(right));
    }

    #[test]
    fn unsupported_work_and_score_overflow_are_errors() {
        let arena = Arena::parse("EG").unwrap();
        let mut battle = Battle::new(&arena);
        battle.work_remaining = 0;
        assert!(battle
            .fight(3, false)
            .unwrap_err()
            .to_string()
            .contains("work budget"));
        battle.completed_rounds = i64::MAX;
        assert!(matches!(
            battle.score(),
            Err(SolveError::ArithmeticOverflow)
        ));
        let large_map = format!("EG{}", ".".repeat(MAX_MAP_CELLS));
        assert!(Arena::parse(&large_map)
            .err()
            .unwrap()
            .to_string()
            .contains("cell budget"));
    }
}
