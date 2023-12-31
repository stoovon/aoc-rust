extern crate core;

use itertools::iproduct;
use std::str::FromStr;

#[derive(Clone, Debug)]
struct Player {
    hp: i64,
    dmg: i64,
    armour: i64,
    cost: i64,
}

impl Player {
    fn new(hp: i64, wep: Item, armour: Item, ring1: Item, ring2: Item) -> Self {
        Self {
            hp,
            dmg: wep.dmg + ring1.dmg + ring2.dmg,
            armour: armour.armor + ring1.armor + ring2.armor,
            cost: wep.cost + armour.cost + ring1.cost + ring2.cost,
        }
    }

    fn wins(&self, boss: &mut Player) -> bool {
        let my_turns_to_win = divide_round_up(boss.hp, calculate_damage(self, boss));
        let boss_turns_to_win = divide_round_up(self.hp, calculate_damage(boss, self));

        my_turns_to_win <= boss_turns_to_win
    }
}

fn calculate_damage(attacker: &Player, defender: &Player) -> i64 {
    let damage = attacker.dmg - defender.armour;
    if damage < 1 {
        // Turns out minimum damage is always 1, so looks like Player is a Bard and can use a weak form of Vicious Mockery.
        1
    } else {
        damage
    }
}

fn divide_round_up(a: i64, b: i64) -> i64 {
    (a / b) + if a % b == 0 { 0 } else { 1 }
}

impl FromStr for Player {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut hp = 0;
        let mut dmg = 0;
        let mut armor = 0;
        let cost = 0;

        for line in s.lines() {
            let mut parts = line.split(": ");
            let name = parts.next().unwrap();
            let value = parts.next().unwrap().parse::<i64>().unwrap();

            match name {
                "Hit Points" => hp = value,
                "Damage" => dmg = value,
                "Armor" => armor = value,
                _ => panic!("Unknown name: {}", name),
            }
        }

        Ok(Player {
            hp,
            dmg,
            armour: armor,
            cost,
        })
    }
}

#[derive(Clone)]
struct Item {
    name: String,
    cost: i64,
    dmg: i64,
    armor: i64,
}

struct ItemShoppe {
    weapons: Vec<Item>,
    armor: Vec<Item>,
    rings: Vec<Item>,
}

impl ItemShoppe {
    fn new() -> Self {
        Self {
            weapons: vec![
                Item {
                    name: "Harsh language".to_string(),
                    cost: 0,
                    dmg: 0,
                    armor: 0,
                },
                Item {
                    name: "Dagger".to_string(),
                    cost: 8,
                    dmg: 4,
                    armor: 0,
                },
                Item {
                    name: "Shortsword".to_string(),
                    cost: 10,
                    dmg: 5,
                    armor: 0,
                },
                Item {
                    name: "Warhammer".to_string(),
                    cost: 25,
                    dmg: 6,
                    armor: 0,
                },
                Item {
                    name: "Longsword".to_string(),
                    cost: 40,
                    dmg: 7,
                    armor: 0,
                },
                Item {
                    name: "Greataxe".to_string(),
                    cost: 74,
                    dmg: 8,
                    armor: 0,
                },
            ],
            armor: vec![
                Item {
                    name: "+5 Armour of Invisible".to_string(),
                    cost: 0,
                    dmg: 0,
                    armor: 0,
                },
                Item {
                    name: "Leather".to_string(),
                    cost: 13,
                    dmg: 0,
                    armor: 1,
                },
                Item {
                    name: "Chainmail".to_string(),
                    cost: 31,
                    dmg: 0,
                    armor: 2,
                },
                Item {
                    name: "Splintmail".to_string(),
                    cost: 53,
                    dmg: 0,
                    armor: 3,
                },
                Item {
                    name: "Bandedmail".to_string(),
                    cost: 75,
                    dmg: 0,
                    armor: 4,
                },
                Item {
                    name: "Platemail".to_string(),
                    cost: 102,
                    dmg: 0,
                    armor: 5,
                },
            ],
            rings: vec![
                Item {
                    name: "Ring 'o Nothing".to_string(),
                    cost: 0,
                    dmg: 0,
                    armor: 0,
                },
                Item {
                    name: "Damage +1".to_string(),
                    cost: 25,
                    dmg: 1,
                    armor: 0,
                },
                Item {
                    name: "Damage +2".to_string(),
                    cost: 50,
                    dmg: 2,
                    armor: 0,
                },
                Item {
                    name: "Damage +3".to_string(),
                    cost: 100,
                    dmg: 3,
                    armor: 0,
                },
                Item {
                    name: "Defense +1".to_string(),
                    cost: 20,
                    dmg: 0,
                    armor: 1,
                },
                Item {
                    name: "Defense +2".to_string(),
                    cost: 40,
                    dmg: 0,
                    armor: 2,
                },
                Item {
                    name: "Defense +3".to_string(),
                    cost: 80,
                    dmg: 0,
                    armor: 3,
                },
            ],
        }
    }
}

pub fn fn1(input: &str) -> i64 {
    let boss: Player = input.parse().unwrap();

    let shoppe = ItemShoppe::new();

    let mut min_cost = i64::MAX;

    for (wep, arm, ring1, ring2) in iproduct!(
        shoppe.weapons,
        shoppe.armor,
        shoppe.rings.clone(),
        shoppe.rings
    ) {
        if ring1.name == ring2.name && ring1.name != "Ring 'o Nothing" {
            continue;
        }

        let player = Player::new(100, wep, arm, ring1, ring2);

        if player.wins(&mut boss.clone()) && player.cost < min_cost {
            min_cost = player.cost;
        }
    }

    min_cost
}

pub fn fn2(input: &str) -> i64 {
    let boss: Player = input.parse().unwrap();

    let shoppe = ItemShoppe::new();

    let mut max_cost = i64::MIN;

    for (wep, arm, ring1, ring2) in iproduct!(
        shoppe.weapons,
        shoppe.armor,
        shoppe.rings.clone(),
        shoppe.rings
    ) {
        if ring1.name == ring2.name && ring1.name != "Ring 'o Nothing" {
            continue;
        }

        let player = Player::new(100, wep, arm, ring1, ring2);

        if !player.wins(&mut boss.clone()) && player.cost > max_cost {
            // Print the player and cost so we can see the build
            println!("{:?} {}", player, player.cost);
            max_cost = player.cost;
        }
    }

    max_cost
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 21;

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
