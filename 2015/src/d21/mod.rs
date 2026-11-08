use std::{iter, str::FromStr};
use svutils::SolveError;

#[derive(Debug)]
struct Player {
    hp: u64,
    dmg: u64,
    armour: u64,
    cost: u64,
}

impl Player {
    fn new(
        hp: u64,
        weapon: &Item,
        armour: Option<&Item>,
        ring1: Option<&Item>,
        ring2: Option<&Item>,
    ) -> Self {
        let mut player = Self {
            hp,
            dmg: 0,
            armour: 0,
            cost: 0,
        };
        for item in [Some(weapon), armour, ring1, ring2].into_iter().flatten() {
            player.dmg += item.dmg;
            player.armour += item.armor;
            player.cost += item.cost;
        }
        player
    }

    fn wins(&self, boss: &Player) -> bool {
        let my_turns_to_win = divide_round_up(boss.hp, calculate_damage(self, boss));
        let boss_turns_to_win = divide_round_up(self.hp, calculate_damage(boss, self));

        my_turns_to_win <= boss_turns_to_win
    }
}

fn calculate_damage(attacker: &Player, defender: &Player) -> u64 {
    attacker.dmg.saturating_sub(defender.armour).max(1)
}

fn divide_round_up(a: u64, b: u64) -> u64 {
    (a / b) + if a % b == 0 { 0 } else { 1 }
}

impl FromStr for Player {
    type Err = SolveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut hp = None;
        let mut dmg = None;
        let mut armour = None;
        for (index, line) in s.lines().enumerate() {
            let invalid =
                |message: &str| SolveError::InvalidInput(format!("line {}: {message}", index + 1));
            let (name, value) = line
                .split_once(':')
                .ok_or_else(|| invalid("expected a boss field and value"))?;
            let field = match name.trim() {
                "Hit Points" => &mut hp,
                "Damage" => &mut dmg,
                "Armor" => &mut armour,
                _ => return Err(invalid("unknown boss field")),
            };
            if field.is_some() {
                return Err(invalid("duplicate boss field"));
            }
            let value = value.trim();
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(invalid("expected a nonnegative decimal integer"));
            }
            *field = Some(
                value
                    .parse::<u64>()
                    .map_err(|_| invalid("boss value exceeds u64"))?,
            );
        }
        let missing = || SolveError::InvalidInput("expected Hit Points, Damage and Armor".into());
        let hp = hp.ok_or_else(missing)?;
        let dmg = dmg.ok_or_else(missing)?;
        let armour = armour.ok_or_else(missing)?;
        if hp == 0 {
            return Err(SolveError::InvalidInput(
                "boss hit points must be positive".into(),
            ));
        }
        Ok(Self {
            hp,
            dmg,
            armour,
            cost: 0,
        })
    }
}

struct Item {
    cost: u64,
    dmg: u64,
    armor: u64,
}

struct ItemShoppe {
    weapons: Vec<Item>,
    armor: Vec<Item>,
    rings: Vec<Item>,
}

impl ItemShoppe {
    fn ring_choices(&self) -> impl Iterator<Item = (Option<&Item>, Option<&Item>)> {
        iter::once((None, None))
            .chain(self.rings.iter().map(|ring| (Some(ring), None)))
            .chain(self.rings.iter().enumerate().flat_map(|(index, first)| {
                self.rings[index + 1..]
                    .iter()
                    .map(move |second| (Some(first), Some(second)))
            }))
    }

    fn loadouts(&self) -> impl Iterator<Item = Player> + '_ {
        self.weapons.iter().flat_map(move |weapon| {
            iter::once(None)
                .chain(self.armor.iter().map(Some))
                .flat_map(move |armour| {
                    self.ring_choices()
                        .map(move |(ring1, ring2)| Player::new(100, weapon, armour, ring1, ring2))
                })
        })
    }

    fn new() -> Self {
        Self {
            weapons: vec![
                Item {
                    cost: 8,
                    dmg: 4,
                    armor: 0,
                },
                Item {
                    cost: 10,
                    dmg: 5,
                    armor: 0,
                },
                Item {
                    cost: 25,
                    dmg: 6,
                    armor: 0,
                },
                Item {
                    cost: 40,
                    dmg: 7,
                    armor: 0,
                },
                Item {
                    cost: 74,
                    dmg: 8,
                    armor: 0,
                },
            ],
            armor: vec![
                Item {
                    cost: 13,
                    dmg: 0,
                    armor: 1,
                },
                Item {
                    cost: 31,
                    dmg: 0,
                    armor: 2,
                },
                Item {
                    cost: 53,
                    dmg: 0,
                    armor: 3,
                },
                Item {
                    cost: 75,
                    dmg: 0,
                    armor: 4,
                },
                Item {
                    cost: 102,
                    dmg: 0,
                    armor: 5,
                },
            ],
            rings: vec![
                Item {
                    cost: 25,
                    dmg: 1,
                    armor: 0,
                },
                Item {
                    cost: 50,
                    dmg: 2,
                    armor: 0,
                },
                Item {
                    cost: 100,
                    dmg: 3,
                    armor: 0,
                },
                Item {
                    cost: 20,
                    dmg: 0,
                    armor: 1,
                },
                Item {
                    cost: 40,
                    dmg: 0,
                    armor: 2,
                },
                Item {
                    cost: 80,
                    dmg: 0,
                    armor: 3,
                },
            ],
        }
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let boss: Player = input.parse()?;
    ItemShoppe::new()
        .loadouts()
        .filter(|player| player.wins(&boss))
        .map(|player| player.cost)
        .min()
        .map(|cost| cost.to_string())
        .ok_or(SolveError::NoSolution("no winning loadout"))
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let boss: Player = input.parse()?;
    ItemShoppe::new()
        .loadouts()
        .filter(|player| !player.wins(&boss))
        .map(|player| player.cost)
        .max()
        .map(|cost| cost.to_string())
        .ok_or(SolveError::NoSolution("no losing loadout"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2015;
    const DAY: i16 = 21;

    #[test]
    fn ring_choices_match_all_subsets_of_at_most_two_items() {
        use std::collections::HashSet;
        let shoppe = ItemShoppe::new();
        let masks: Vec<_> = shoppe
            .ring_choices()
            .map(|(first, second)| {
                let mut mask = 0usize;
                for ring in [first, second].into_iter().flatten() {
                    let index = shoppe
                        .rings
                        .iter()
                        .position(|item| std::ptr::eq(item, ring))
                        .unwrap();
                    assert_eq!(mask & (1 << index), 0);
                    mask |= 1 << index;
                }
                mask
            })
            .collect();
        let expected: HashSet<_> = (0usize..1 << shoppe.rings.len())
            .filter(|mask| mask.count_ones() <= 2)
            .collect();
        assert_eq!(masks.len(), 22);
        assert_eq!(masks.iter().copied().collect::<HashSet<_>>(), expected);
        assert_eq!(shoppe.loadouts().count(), 660);
    }

    fn wins_by_attacking(player: &Player, boss: &Player) -> bool {
        let mut player_hp = player.hp;
        let mut boss_hp = boss.hp;
        loop {
            boss_hp = boss_hp.saturating_sub(player.dmg.saturating_sub(boss.armour).max(1));
            if boss_hp == 0 {
                return true;
            }
            player_hp = player_hp.saturating_sub(boss.dmg.saturating_sub(player.armour).max(1));
            if player_hp == 0 {
                return false;
            }
        }
    }

    #[test]
    fn equal_hit_counts_favour_the_player() {
        let player = Player {
            hp: 16,
            dmg: 4,
            armour: 0,
            cost: 0,
        };
        let boss = Player {
            hp: 16,
            dmg: 4,
            armour: 0,
            cost: 0,
        };
        assert!(wins_by_attacking(&player, &boss));
        assert!(player.wins(&boss));
    }

    #[test]
    fn hit_count_formula_matches_turn_by_turn_combat() {
        for hp in 1..=8 {
            for boss_hp in 1..=8 {
                for dmg in 0..=5 {
                    for boss_dmg in 0..=5 {
                        for armour in 0..=3 {
                            for boss_armour in 0..=3 {
                                let player = Player {
                                    hp,
                                    dmg,
                                    armour,
                                    cost: 0,
                                };
                                let boss = Player {
                                    hp: boss_hp,
                                    dmg: boss_dmg,
                                    armour: boss_armour,
                                    cost: 0,
                                };
                                assert_eq!(
                                    player.wins(&boss),
                                    wins_by_attacking(&player, &boss),
                                    "player={player:?}, boss={boss:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn boss_parser_requires_three_unique_valid_fields() {
        let boss: Player = "Armor: 0\nHit Points: 19\nDamage: 0\n".parse().unwrap();
        assert_eq!((boss.hp, boss.dmg, boss.armour), (19, 0, 0));
        for input in [
            "",
            "Hit Points: 19\nDamage: 2",
            "Hit Points: 0\nDamage: 2\nArmor: 1",
            "Hit Points: 19\nDamage: 2\nArmor: 1\nArmor: 1",
            "Hit Points: 19\nDamage: 2\nArmor: 1\nSpeed: 3",
            "Hit Points 19\nDamage: 2\nArmor: 1",
        ] {
            assert!(part1(input).is_err(), "{input:?}");
            assert!(part2(input).is_err(), "{input:?}");
        }
        for name in ["Hit Points", "Damage", "Armor"] {
            for value in ["", "-1", "+1", "1.5", "1:2", "18446744073709551616"] {
                let input = ["Hit Points", "Damage", "Armor"]
                    .map(|field| format!("{field}: {}", if field == name { value } else { "2" }))
                    .join("\n");
                assert!(input.parse::<Player>().is_err(), "{input:?}");
            }
        }
    }

    #[test]
    fn combat_handles_maximum_unsigned_stats() {
        let fighter: Player = format!(
            "Hit Points: {}\nDamage: {}\nArmor: {}",
            u64::MAX,
            u64::MAX,
            u64::MAX
        )
        .parse()
        .unwrap();
        assert_eq!(calculate_damage(&fighter, &fighter), 1);
        assert!(fighter.wins(&fighter));
        assert_eq!(divide_round_up(u64::MAX, 2), u64::MAX / 2 + 1);
    }

    #[test]
    fn valid_bosses_without_qualifying_loadouts_have_semantic_errors() {
        let easy = "Hit Points: 1\nDamage: 0\nArmor: 0";
        assert!(easy.parse::<Player>().is_ok());
        assert!(part1(easy).is_ok());
        assert!(matches!(
            part2(easy),
            Err(SolveError::NoSolution("no losing loadout"))
        ));
        let hard = format!(
            "Hit Points: {}\nDamage: {}\nArmor: {}",
            u64::MAX,
            u64::MAX,
            u64::MAX
        );
        assert!(hard.parse::<Player>().is_ok());
        assert!(matches!(
            part1(&hard),
            Err(SolveError::NoSolution("no winning loadout"))
        ));
        assert!(part2(&hard).is_ok());
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
