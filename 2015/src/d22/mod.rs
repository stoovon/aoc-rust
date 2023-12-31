extern crate core;

use std::fmt;

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Boss {
    hp: isize,
    dmg: isize,
    effects: Vec<Effect>,
}

impl FromStr for Boss {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut hp = 0 as isize;
        let mut dmg = 0 as isize;

        for line in s.lines() {
            let mut parts = line.split(": ");
            let name = parts.next().unwrap();
            let value = parts.next().unwrap().parse::<isize>().unwrap();

            match name {
                "Hit Points" => hp = value,
                "Damage" => dmg = value,
                _ => panic!("Unknown boss stat: {}", name),
            }
        }

        Ok(Self {
            hp,
            dmg,
            effects: vec![],
        })
    }
}

impl Fighter for Boss {
    fn damage(&self) -> isize {
        self.dmg
    }
    fn health(&self) -> isize {
        self.hp
    }
    fn health_mut(&mut self) -> &mut isize {
        &mut self.hp
    }
    fn mana(&self) -> isize {
        0
    }
    fn mana_mut(&mut self) -> &mut isize {
        unimplemented!()
    }
    fn effects(&self) -> &[Effect] {
        &self.effects
    }
    fn effects_mut(&mut self) -> &mut Vec<Effect> {
        &mut self.effects
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    id: usize,
    duration: usize,
    armor_boost_while_active: isize, // while active
    health_per_turn: isize,          // each tick
    mana_per_turn: isize,            // each tick
}

#[derive(Debug, PartialEq, Eq)]
pub struct Spell {
    name: String,
    hp_self: isize,
    hp_target: isize,
    mp_self: isize,
    mp_target: isize,
    effect_self: Option<Effect>,
    effect_target: Option<Effect>,
}

impl Spell {
    fn mana_usage(&self) -> isize {
        if self.mp_self < 0 {
            -self.mp_self
        } else {
            0
        }
    }
}

fn spell_book() -> Vec<Spell> {
    vec![
        Spell {
            name: "magic missile".to_string(),
            hp_self: 0,
            hp_target: -4,
            mp_self: -53,
            mp_target: 0,
            effect_self: None,
            effect_target: None,
        },
        Spell {
            name: "drain".to_string(),
            hp_self: 2,
            hp_target: -2,
            mp_self: -73,
            mp_target: 0,
            effect_self: None,
            effect_target: None,
        },
        Spell {
            name: "shield".to_string(),
            hp_self: 0,
            hp_target: 0,
            mp_self: -113,
            mp_target: 0,
            effect_self: Some(Effect {
                id: 1,
                duration: 6,
                armor_boost_while_active: 7,
                health_per_turn: 0,
                mana_per_turn: 0,
            }),
            effect_target: None,
        },
        Spell {
            name: "poison".to_string(),
            hp_self: 0,
            hp_target: 0,
            mp_self: -173,
            mp_target: 0,
            effect_target: Some(Effect {
                id: 2,
                duration: 6,
                armor_boost_while_active: 0,
                health_per_turn: -3,
                mana_per_turn: 0,
            }),
            effect_self: None,
        },
        Spell {
            name: "recharge".to_string(),
            hp_self: 0,
            hp_target: 0,
            mp_self: -229,
            mp_target: 0,
            effect_self: Some(Effect {
                id: 3,
                duration: 5,
                armor_boost_while_active: 0,
                health_per_turn: 0,
                mana_per_turn: 101,
            }),
            effect_target: None,
        },
    ]
}

pub trait Fighter: Sized {
    fn damage(&self) -> isize;
    fn health(&self) -> isize;
    fn health_mut(&mut self) -> &mut isize;
    fn mana(&self) -> isize;
    fn mana_mut(&mut self) -> &mut isize;
    fn effects(&self) -> &[Effect];
    fn effects_mut(&mut self) -> &mut Vec<Effect>;

    fn alive(&self) -> bool {
        self.health() <= 0
    }

    fn take_damage(&mut self, dmg: isize) {
        if dmg != 0 {
            *self.health_mut() -= dmg;
        }
    }

    fn drain(&mut self, mn: isize) {
        if mn != 0 {
            *self.mana_mut() -= mn;
        }
    }

    fn has_effect(&self, effect: &Effect) -> bool {
        self.effects().iter().any(|e| e.id == effect.id)
    }

    fn add_effect(&mut self, effect: Effect) {
        self.effects_mut().push(effect);
    }

    fn apply_effects(&mut self) {
        let mut hpt = 0;
        let mut mpt = 0;
        for effect in self.effects_mut() {
            hpt += effect.health_per_turn;
            mpt += effect.mana_per_turn;
            effect.duration -= 1;
        }
        self.take_damage(-hpt);
        self.drain(-mpt);
        self.effects_mut().retain(|e| e.duration > 0);
    }

    fn armor(&self) -> isize {
        self.effects()
            .iter()
            .map(|e| e.armor_boost_while_active)
            .sum()
    }

    fn attack<F: Fighter>(&mut self, target: &mut F) {
        let damage = self.damage() - target.armor();
        target.take_damage(if damage > 0 { damage } else { 1 });
    }

    fn can_cast<F: Fighter>(&self, target: &F, spell: &Spell) -> bool {
        spell.mana_usage() <= self.mana()
            && (spell.effect_self.is_none()
                || !self.has_effect(spell.effect_self.as_ref().unwrap()))
            && (spell.effect_target.is_none()
                || !target.has_effect(spell.effect_target.as_ref().unwrap()))
    }

    fn cast<F: Fighter>(&mut self, target: &mut F, spell: &Spell) -> isize {
        if !self.can_cast(target, spell) {
            return 0;
        }
        self.take_damage(-spell.hp_self);
        target.take_damage(-spell.hp_target);
        self.drain(-spell.mp_self);
        target.drain(-spell.mp_target);
        if let Some(ref effect) = spell.effect_self {
            self.add_effect(effect.clone());
        }
        if let Some(ref effect) = spell.effect_target {
            target.add_effect(effect.clone());
        }
        spell.mana_usage()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    hp: isize,
    mp: isize,
    effects: Vec<Effect>,
}

impl Player {
    fn new(hp: isize, mp: isize) -> Player {
        Player {
            hp: hp,
            mp: mp,
            effects: vec![],
        }
    }
}

impl Fighter for Player {
    fn damage(&self) -> isize {
        0
    }
    fn health(&self) -> isize {
        self.hp
    }
    fn health_mut(&mut self) -> &mut isize {
        &mut self.hp
    }
    fn mana(&self) -> isize {
        self.mp
    }
    fn mana_mut(&mut self) -> &mut isize {
        &mut self.mp
    }
    fn effects(&self) -> &[Effect] {
        &self.effects
    }
    fn effects_mut(&mut self) -> &mut Vec<Effect> {
        &mut self.effects
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("Player({}|{})", self.hp, self.mp))
    }
}

fn bfs(player: &Player, boss: &Boss, min_mp: &mut isize, mp: isize, hard: bool) {
    for spell in spell_book().iter() {
        let mut player = player.clone();
        let mut boss = boss.clone();

        if hard {
            player.take_damage(1);
            if player.alive() {
                continue;
            }
        }

        player.apply_effects();
        boss.apply_effects();
        if player.alive() {
            continue;
        }
        if boss.alive() {
            if mp < *min_mp {
                *min_mp = mp
            };
            continue;
        }

        if !player.can_cast(&boss, &spell) {
            continue;
        }
        if mp + spell.mana_usage() >= *min_mp {
            continue;
        }

        let mp_used = player.cast(&mut boss, spell);
        if player.alive() {
            continue;
        }
        if boss.alive() {
            if mp + mp_used < *min_mp {
                *min_mp = mp + mp_used
            };
            continue;
        }

        player.apply_effects();
        boss.apply_effects();
        if player.alive() {
            continue;
        }
        if boss.alive() {
            if mp + mp_used < *min_mp {
                *min_mp = mp + mp_used
            };
            continue;
        }

        boss.attack(&mut player);
        if player.alive() {
            continue;
        }
        if boss.alive() {
            if mp + mp_used < *min_mp {
                *min_mp = mp + mp_used
            };
            continue;
        }

        bfs(&player, &boss, min_mp, mp + mp_used, hard);
    }
}

fn find_min_mana(boss: &Boss, hard: bool) -> isize {
    let player = Player::new(50, 500);
    let mut min_mp = isize::max_value();
    bfs(&player, boss, &mut min_mp, 0, hard);
    min_mp
}

pub fn fn1(input: &str) -> i64 {
    let boss: Boss = input.parse().unwrap();
    find_min_mana(&boss, false) as i64
}

pub fn fn2(input: &str) -> i64 {
    let boss: Boss = input.parse().unwrap();
    find_min_mana(&boss, true) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 22;

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
