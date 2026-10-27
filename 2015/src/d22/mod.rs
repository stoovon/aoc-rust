use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use svutils::SolveError;

struct Boss {
    hp: i64,
    damage: i64,
}

impl Boss {
    fn parse(input: &str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let (mut hp, mut damage) = (None, None);
        for (index, line) in input.lines().enumerate() {
            let mut parse = || {
                let (name, value) = line
                    .split_once(": ")
                    .ok_or_else(|| SolveError::InvalidInput("expected boss stat: value".into()))?;
                let slot = match name {
                    "Hit Points" => &mut hp,
                    "Damage" => &mut damage,
                    _ => {
                        return Err(SolveError::InvalidInput(format!(
                            "unknown boss stat '{name}'"
                        )))
                    }
                };
                if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(SolveError::InvalidInput(
                        "boss stats must be positive decimal integers".into(),
                    ));
                }
                let value: i64 = value.parse()?;
                if value == 0 {
                    return Err(SolveError::InvalidInput(
                        "boss stats must be positive".into(),
                    ));
                }
                if slot.replace(value).is_some() {
                    return Err(SolveError::InvalidInput(format!(
                        "duplicate boss stat '{name}'"
                    )));
                }
                Ok(())
            };
            parse().map_err(|error: SolveError| error.at_line(index + 1))?;
        }
        Ok(Self {
            hp: hp.ok_or_else(|| SolveError::InvalidInput("missing Hit Points".into()))?,
            damage: damage.ok_or_else(|| SolveError::InvalidInput("missing Damage".into()))?,
        })
    }
}

#[derive(Clone, Copy)]
enum Difficulty {
    Normal,
    Hard,
}

#[derive(Clone, Copy, Debug)]
enum Spell {
    MagicMissile,
    Drain,
    Shield,
    Poison,
    Recharge,
}

impl Spell {
    const ALL: [Self; 5] = [
        Self::MagicMissile,
        Self::Drain,
        Self::Shield,
        Self::Poison,
        Self::Recharge,
    ];

    fn cost(self) -> i64 {
        match self {
            Self::MagicMissile => 53,
            Self::Drain => 73,
            Self::Shield => 113,
            Self::Poison => 173,
            Self::Recharge => 229,
        }
    }
}

// Every queued state is immediately before a player's turn, or an already won fight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct BattleState {
    hp: i64,
    mana: i64,
    boss_hp: i64,
    shield: u8,
    poison: u8,
    recharge: u8,
}

impl BattleState {
    fn new(hp: i64, mana: i64, boss_hp: i64) -> Self {
        Self {
            hp,
            mana,
            boss_hp,
            shield: 0,
            poison: 0,
            recharge: 0,
        }
    }

    fn apply_effects(&mut self) -> Result<(), SolveError> {
        if self.shield > 0 {
            self.shield -= 1;
        }
        if self.poison > 0 {
            self.boss_hp -= 3;
            self.poison -= 1;
        }
        if self.recharge > 0 {
            self.mana = self
                .mana
                .checked_add(101)
                .ok_or(SolveError::ArithmeticOverflow)?;
            self.recharge -= 1;
        }
        Ok(())
    }

    fn start_player_turn(mut self, difficulty: Difficulty) -> Result<Option<Self>, SolveError> {
        if matches!(difficulty, Difficulty::Hard) {
            self.hp -= 1;
        }
        if self.hp <= 0 {
            return Ok(None);
        }
        self.apply_effects()?;
        Ok(Some(self))
    }

    fn can_cast(self, spell: Spell) -> bool {
        self.mana >= spell.cost()
            && match spell {
                Spell::Shield => self.shield == 0,
                Spell::Poison => self.poison == 0,
                Spell::Recharge => self.recharge == 0,
                Spell::MagicMissile | Spell::Drain => true,
            }
    }

    // Called after player-turn effects, and only while the boss is alive.
    fn cast_and_boss_turn(
        mut self,
        spell: Spell,
        boss_damage: i64,
    ) -> Result<Option<Self>, SolveError> {
        if !self.can_cast(spell) {
            return Ok(None);
        }
        self.mana -= spell.cost();
        match spell {
            Spell::MagicMissile => self.boss_hp -= 4,
            Spell::Drain => {
                self.boss_hp -= 2;
                self.hp = self
                    .hp
                    .checked_add(2)
                    .ok_or(SolveError::ArithmeticOverflow)?;
            }
            Spell::Shield => self.shield = 6,
            Spell::Poison => self.poison = 6,
            Spell::Recharge => self.recharge = 5,
        }
        if self.boss_hp <= 0 {
            return Ok(Some(self));
        }
        self.apply_effects()?;
        if self.boss_hp <= 0 {
            return Ok(Some(self));
        }
        // Shield ending at the start of this turn no longer provides armour.
        let armour = if self.shield > 0 { 7 } else { 0 };
        self.hp -= (boss_damage - armour).max(1);
        Ok((self.hp > 0).then_some(self))
    }
}

fn minimum_mana(
    boss: &Boss,
    difficulty: Difficulty,
    hp: i64,
    mana: i64,
) -> Result<i64, SolveError> {
    let start = BattleState::new(hp, mana, boss.hp);
    let mut queue = BinaryHeap::from([Reverse((0_i64, start))]);
    let mut cheapest = HashMap::from([(start, 0_i64)]);
    while let Some(Reverse((spent, state))) = queue.pop() {
        if cheapest.get(&state) != Some(&spent) {
            continue;
        }
        if state.boss_hp <= 0 {
            return Ok(spent);
        }
        let Some(state) = state.start_player_turn(difficulty)? else {
            continue;
        };
        // This is already the cheapest pending path: poison needs no further spell.
        if state.boss_hp <= 0 {
            return Ok(spent);
        }
        for spell in Spell::ALL {
            let Some(next) = state.cast_and_boss_turn(spell, boss.damage)? else {
                continue;
            };
            let next_cost = spent
                .checked_add(spell.cost())
                .ok_or(SolveError::ArithmeticOverflow)?;
            if cheapest.get(&next).is_some_and(|&known| known <= next_cost) {
                continue;
            }
            cheapest.insert(next, next_cost);
            // A win discovered here must still compete with cheaper pending routes.
            queue.push(Reverse((next_cost, next)));
        }
    }
    Err(SolveError::InvalidInput("no winning spell sequence".into()))
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let boss = Boss::parse(input)?;
    Ok(minimum_mana(&boss, Difficulty::Normal, 50, 500)?.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let boss = Boss::parse(input)?;
    Ok(minimum_mana(&boss, Difficulty::Hard, 50, 500)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 22, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 22, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn poison_wins_before_casting_but_after_hard_mode_penalty() {
        let mut state = BattleState::new(1, 0, 3);
        state.poison = 1;
        let won = state
            .start_player_turn(Difficulty::Normal)
            .unwrap()
            .unwrap();
        assert!(won.boss_hp <= 0);
        assert_eq!(won.mana, 0);
        assert!(state.start_player_turn(Difficulty::Hard).unwrap().is_none());
    }

    #[test]
    fn effects_expire_and_can_be_recast_on_that_turn() {
        let mut state = BattleState::new(20, 500, 50);
        state.shield = 1;
        state.poison = 1;
        state.recharge = 1;
        assert!(!state.can_cast(Spell::Shield));
        let state = state
            .start_player_turn(Difficulty::Normal)
            .unwrap()
            .unwrap();
        assert_eq!((state.shield, state.poison, state.recharge), (0, 0, 0));
        assert_eq!((state.boss_hp, state.mana), (47, 601));
        assert!(state.can_cast(Spell::Shield));
        assert!(state.can_cast(Spell::Poison));
        assert!(state.can_cast(Spell::Recharge));
    }

    #[test]
    fn shield_expiry_and_minimum_boss_damage() {
        let mut state = BattleState::new(20, 500, 50);
        state.shield = 1;
        let next = state
            .cast_and_boss_turn(Spell::MagicMissile, 8)
            .unwrap()
            .unwrap();
        assert_eq!(next.hp, 12);
        state.shield = 2;
        let next = state
            .cast_and_boss_turn(Spell::MagicMissile, 3)
            .unwrap()
            .unwrap();
        assert_eq!(next.hp, 19);
    }

    #[test]
    fn immediate_and_boss_turn_poison_wins_prevent_attacks() {
        let state = BattleState::new(1, 53, 4);
        let next = state
            .cast_and_boss_turn(Spell::MagicMissile, 100)
            .unwrap()
            .unwrap();
        assert!(next.boss_hp <= 0);
        assert_eq!(next.hp, 1);
        let state = BattleState::new(1, 173, 3);
        let next = state
            .cast_and_boss_turn(Spell::Poison, 100)
            .unwrap()
            .unwrap();
        assert!(next.boss_hp <= 0);
        assert_eq!(next.hp, 1);
    }

    #[test]
    fn tiny_searches_and_unwinnable_fights() {
        let boss = Boss { hp: 5, damage: 1 };
        assert_eq!(
            minimum_mana(&boss, Difficulty::Normal, 10, 106).unwrap(),
            106
        );
        assert!(minimum_mana(&boss, Difficulty::Normal, 1, 0).is_err());
        assert!(minimum_mana(&boss, Difficulty::Hard, 1, 500).is_err());
    }

    #[test]
    fn invalid_boss_stats_are_rejected() {
        for input in [
            "",
            "Hit Points: 5",
            "Damage: 8",
            "Hit Points: 0\nDamage: 8",
            "Hit Points: -1\nDamage: 8",
            "Hit Points: 5\nDamage: 8\nDamage: 9",
            "Hit Points: 5\nArmour: 8",
            "Hit Points: nope\nDamage: 8",
            "Hit Points: 5\n\nDamage: 8",
            "Hit Points: 5\nDamage: +8",
        ] {
            assert!(Boss::parse(input).is_err(), "{input:?}");
        }
        assert_eq!(part1("Damage: 8\r\nHit Points: 4\r\n").unwrap(), "53");
    }
    #[test]
    fn recharge_overflow_is_an_error() {
        let mut state = BattleState::new(10, i64::MAX, 10);
        state.recharge = 1;
        assert!(matches!(
            state.apply_effects(),
            Err(SolveError::ArithmeticOverflow)
        ));
    }
}
