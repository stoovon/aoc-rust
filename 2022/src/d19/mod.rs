use std::collections::HashSet;
use svutils::SolveError;

#[derive(Clone, Copy, Debug)]
struct Blueprint {
    id: i64,
    ore_robot: u16,
    clay_robot: u16,
    obsidian_robot: (u16, u16),
    geode_robot: (u16, u16),
    max_ore_cost: u16,
}

impl Blueprint {
    #[inline]
    fn max_ore_cost(&self) -> u16 {
        self.max_ore_cost
    }

    #[inline]
    fn max_clay_cost(&self) -> u16 {
        self.obsidian_robot.1
    }

    #[inline]
    fn max_obsidian_cost(&self) -> u16 {
        self.geode_robot.1
    }
}

fn take_decimal<'a>(rest: &mut &'a str, suffix: &str) -> Result<&'a str, SolveError> {
    let (number, following) = rest.split_once(suffix).ok_or_else(|| {
        SolveError::InvalidInput("expected a complete robot blueprint record".into())
    })?;
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SolveError::InvalidInput(
            "expected an unsigned decimal integer".into(),
        ));
    }
    *rest = following;
    Ok(number)
}

impl Blueprint {
    fn parse(line: &str) -> Result<Self, SolveError> {
        let mut rest = line
            .strip_prefix("Blueprint ")
            .ok_or_else(|| SolveError::InvalidInput("expected 'Blueprint '".into()))?;
        let id: i64 = take_decimal(&mut rest, ": Each ore robot costs ")?.parse()?;
        let ore_robot: u16 = take_decimal(&mut rest, " ore. Each clay robot costs ")?.parse()?;
        let clay_robot: u16 =
            take_decimal(&mut rest, " ore. Each obsidian robot costs ")?.parse()?;
        let obsidian_ore: u16 = take_decimal(&mut rest, " ore and ")?.parse()?;
        let obsidian_clay: u16 =
            take_decimal(&mut rest, " clay. Each geode robot costs ")?.parse()?;
        let geode_ore: u16 = take_decimal(&mut rest, " ore and ")?.parse()?;
        let geode_obsidian: u16 = take_decimal(&mut rest, " obsidian.")?.parse()?;
        if id == 0 || !rest.is_empty() {
            return Err(SolveError::InvalidInput(
                "expected a positive ID and no trailing data".into(),
            ));
        }
        Ok(Self {
            id,
            ore_robot,
            clay_robot,
            obsidian_robot: (obsidian_ore, obsidian_clay),
            geode_robot: (geode_ore, geode_obsidian),
            max_ore_cost: ore_robot.max(clay_robot).max(obsidian_ore).max(geode_ore),
        })
    }
}

fn parse(input: &str) -> Result<Vec<Blueprint>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut blueprints = Vec::new();
    let mut ids = HashSet::new();
    for (index, line) in input.lines().enumerate() {
        let blueprint = Blueprint::parse(line).map_err(|error| error.at_line(index + 1))?;
        if !ids.insert(blueprint.id) {
            return Err(
                SolveError::InvalidInput("duplicate blueprint ID".into()).at_line(index + 1)
            );
        }
        blueprints.push(blueprint);
    }
    Ok(blueprints)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Pack {
    ore_robot: u8,
    clay_robot: u8,
    obsidian_robot: u8,
    geode_robot: u8,
    ore: u16,
    clay: u16,
    obsidian: u16,
    geode: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct State {
    minute: u8,
    pack: Pack,
}

impl State {
    fn new() -> Self {
        State {
            minute: 0,
            pack: Pack {
                ore_robot: 1,
                clay_robot: 0,
                obsidian_robot: 0,
                geode_robot: 0,
                ore: 0,
                clay: 0,
                obsidian: 0,
                geode: 0,
            },
        }
    }

    #[inline]
    fn can_build_ore_robot(&self, bp: &Blueprint) -> bool {
        self.pack.ore >= bp.ore_robot
    }

    #[inline]
    fn can_build_clay_robot(&self, bp: &Blueprint) -> bool {
        self.pack.ore >= bp.clay_robot
    }

    #[inline]
    fn can_build_obsidian_robot(&self, bp: &Blueprint) -> bool {
        self.pack.ore >= bp.obsidian_robot.0 && self.pack.clay >= bp.obsidian_robot.1
    }

    #[inline]
    fn can_build_geode_robot(&self, bp: &Blueprint) -> bool {
        self.pack.ore >= bp.geode_robot.0 && self.pack.obsidian >= bp.geode_robot.1
    }

    #[inline]
    fn build_ore_robot(self, bp: &Blueprint) -> Self {
        let mut result = self;
        result.pack.ore -= bp.ore_robot;
        result.pack.ore_robot += 1;
        result
    }

    #[inline]
    fn build_clay_robot(self, bp: &Blueprint) -> Self {
        let mut result = self;
        result.pack.ore -= bp.clay_robot;
        result.pack.clay_robot += 1;
        result
    }

    #[inline]
    fn build_obsidian_robot(self, bp: &Blueprint) -> Self {
        let mut result = self;
        result.pack.ore -= bp.obsidian_robot.0;
        result.pack.clay -= bp.obsidian_robot.1;
        result.pack.obsidian_robot += 1;
        result
    }

    #[inline]
    fn build_geode_robot(self, bp: &Blueprint) -> Self {
        let mut result = self;
        result.pack.ore -= bp.geode_robot.0;
        result.pack.obsidian -= bp.geode_robot.1;
        result.pack.geode_robot += 1;
        result
    }

    #[inline]
    fn step(self) -> Self {
        let mut this = self;
        this.minute += 1;
        this.pack.ore += u16::from(this.pack.ore_robot);
        this.pack.clay += u16::from(this.pack.clay_robot);
        this.pack.obsidian += u16::from(this.pack.obsidian_robot);
        this.pack.geode += u16::from(this.pack.geode_robot);
        this
    }
}

const SEARCH_WORK_LIMIT: u32 = 10_000_000;

struct Search {
    best: u16,
    remaining_work: u32,
}

fn solution(
    state: State,
    bp: &Blueprint,
    limit: u8,
    search: &mut Search,
    can_ore: bool,
    can_clay: bool,
    can_obsidian: bool,
) -> Result<(), SolveError> {
    search.remaining_work =
        search
            .remaining_work
            .checked_sub(1)
            .ok_or(SolveError::ResourceLimit(
                "robot search exceeded its state-visit budget",
            ))?;
    if state.minute == limit {
        let result = state.pack.geode;
        search.best = search.best.max(result);
        return Ok(());
    }

    {
        let remaining = (limit - state.minute) as u32;

        // Relax resource costs and allow a new geode robot every remaining minute.
        let max_yield = remaining * state.pack.geode_robot as u32 + remaining * (remaining - 1) / 2;

        if state.pack.geode as u32 + max_yield <= u32::from(search.best) {
            return Ok(());
        }
    }

    // Explore geode production first to establish a strong result for the upper bound.
    if state.can_build_geode_robot(bp) {
        solution(
            state.step().build_geode_robot(bp),
            bp,
            limit,
            search,
            true,
            true,
            true,
        )?;
    }
    {
        let mut new_can_obsidian = true;
        if state.can_build_obsidian_robot(bp) {
            new_can_obsidian = false;

            // One build per minute limits the useful production rate for each resource.
            if can_obsidian && u16::from(state.pack.obsidian_robot) < bp.max_obsidian_cost() {
                solution(
                    state.step().build_obsidian_robot(bp),
                    bp,
                    limit,
                    search,
                    true,
                    true,
                    true,
                )?;
            }
        }

        let mut new_can_clay = true;
        if state.can_build_clay_robot(bp) {
            new_can_clay = false;

            if can_clay && u16::from(state.pack.clay_robot) < bp.max_clay_cost() {
                solution(
                    state.step().build_clay_robot(bp),
                    bp,
                    limit,
                    search,
                    true,
                    true,
                    true,
                )?;
            }
        }

        let mut new_can_ore = true;
        if state.can_build_ore_robot(bp) {
            new_can_ore = false;

            if can_ore && u16::from(state.pack.ore_robot) < bp.max_ore_cost() {
                solution(
                    state.step().build_ore_robot(bp),
                    bp,
                    limit,
                    search,
                    true,
                    true,
                    true,
                )?;
            }
        }

        solution(
            state.step(),
            bp,
            limit,
            search,
            new_can_ore,
            new_can_clay,
            new_can_obsidian,
        )?;
    }
    Ok(())
}

fn max_geodes_with_limit(blueprint: &Blueprint, minutes: u8, work: u32) -> Result<u16, SolveError> {
    if minutes > 32 {
        return Err(SolveError::InvalidInput(
            "robot search supports at most 32 minutes".into(),
        ));
    }
    // One build per minute bounds robots at 33 and balances at 528.
    let mut search = Search {
        best: 0,
        remaining_work: work,
    };
    solution(
        State::new(),
        blueprint,
        minutes,
        &mut search,
        true,
        true,
        true,
    )?;
    Ok(search.best)
}

fn max_geodes(blueprint: &Blueprint, minutes: u8) -> Result<u16, SolveError> {
    max_geodes_with_limit(blueprint, minutes, SEARCH_WORK_LIMIT)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let blueprints = parse(input)?;
    let mut quality = 0_i64;
    for blueprint in &blueprints {
        let score = blueprint
            .id
            .checked_mul(i64::from(max_geodes(blueprint, 24)?))
            .ok_or(SolveError::ArithmeticOverflow)?;
        quality = quality
            .checked_add(score)
            .ok_or(SolveError::ArithmeticOverflow)?;
    }
    Ok(quality.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let blueprints = parse(input)?;
    let mut product = 1_i64;
    for blueprint in blueprints.iter().take(3) {
        product = product
            .checked_mul(i64::from(max_geodes(blueprint, 32)?))
            .ok_or(SolveError::ArithmeticOverflow)?;
    }
    Ok(product.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2022;
    const DAY: i16 = 19;

    #[test]
    fn investing_in_ore_can_beat_an_affordable_geode_robot() {
        let blueprint = Blueprint {
            id: 71,
            ore_robot: 2,
            clay_robot: 1,
            obsidian_robot: (1, 1),
            geode_robot: (3, 1),
            max_ore_cost: 3,
        };
        // At minute 8, invest in ore despite having enough to build a geode robot.
        // The resulting three ore per minute sustain a new geode robot each turn.
        let mut state = State::new();
        for action in [
            None,
            None,
            Some(1),
            Some(0),
            Some(2),
            None,
            Some(3),
            Some(0),
            Some(3),
            Some(3),
            Some(3),
            Some(3),
            Some(3),
            None,
        ] {
            if let Some(robot) = action {
                let affordable = match robot {
                    0 => state.can_build_ore_robot(&blueprint),
                    1 => state.can_build_clay_robot(&blueprint),
                    2 => state.can_build_obsidian_robot(&blueprint),
                    3 => state.can_build_geode_robot(&blueprint),
                    _ => unreachable!(),
                };
                assert!(affordable);
            }
            if state.minute == 7 {
                assert!(state.can_build_geode_robot(&blueprint));
            }
            state = state.step();
            state = match action {
                Some(0) => state.build_ore_robot(&blueprint),
                Some(1) => state.build_clay_robot(&blueprint),
                Some(2) => state.build_obsidian_robot(&blueprint),
                Some(3) => state.build_geode_robot(&blueprint),
                None => state,
                _ => unreachable!(),
            };
        }
        assert_eq!(state.pack.geode, 22);
        assert_eq!(max_geodes(&blueprint, 14).unwrap(), 22);
    }

    #[test]
    fn resource_balances_can_exceed_one_byte() {
        let blueprint = Blueprint {
            id: 73,
            ore_robot: 0,
            clay_robot: 0,
            obsidian_robot: (0, 0),
            geode_robot: (0, 0),
            max_ore_cost: 0,
        };
        let mut state = State::new();
        for _ in 0..32 {
            state = state.step().build_ore_robot(&blueprint);
        }
        assert_eq!(state.minute, 32);
        assert_eq!(state.pack.ore_robot, 33);
        assert_eq!(u32::from(state.pack.ore), 32 * 33 / 2);

        assert_eq!(u32::from(max_geodes(&blueprint, 32).unwrap()), 32 * 31 / 2);
    }

    fn synthetic_record(id: i64, costs: [u16; 6]) -> String {
        format!(
            "Blueprint {id}: Each ore robot costs {} ore. Each clay robot costs {} ore. \
             Each obsidian robot costs {} ore and {} clay. Each geode robot costs {} ore and {} obsidian.",
            costs[0], costs[1], costs[2], costs[3], costs[4], costs[5],
        )
    }

    #[test]
    fn parser_validates_complete_records_and_preserves_numeric_ranges() {
        let record = synthetic_record(401, [2, 1, 1, 1, 3, 1]);
        let blueprints = parse(&format!("{record}\r\n")).unwrap();
        assert_eq!(blueprints[0].id, 401);
        assert_eq!(blueprints[0].obsidian_robot, (1, 1));
        assert_eq!(blueprints[0].max_ore_cost(), 3);
        assert_eq!(
            parse(&synthetic_record(71, [u16::MAX; 6])).unwrap()[0].ore_robot,
            u16::MAX
        );
        assert!(parse(&synthetic_record(73, [0; 6])).is_ok());
        for invalid in [
            String::new(),
            " \n\t".into(),
            synthetic_record(0, [1; 6]),
            synthetic_record(-1, [1; 6]),
            record.replacen("401", "9223372036854775808", 1),
            record.replacen("2 ore.", "65536 ore.", 1),
            record.replacen("2 ore.", "-2 ore.", 1),
            record.replacen("2 ore.", "+2 ore.", 1),
            record.replacen("2 ore.", " ore.", 1),
            record.replacen("Each ore", "Every ore", 1),
            record.replace(" obsidian.", " obsidian"),
            format!("{record} junk"),
            format!(" {record}"),
            format!("{record}\n\n"),
            format!("{record}\n{record}"),
            "1 2 3 4 5 6 7".into(),
        ] {
            assert!(
                parse(&invalid).is_err(),
                "accepted malformed blueprint: {invalid:?}"
            );
        }
        let invalid = format!("{record}\nbad blueprint");
        for part in [part1, part2] {
            assert!(part(&invalid)
                .unwrap_err()
                .to_string()
                .starts_with("line 2:"));
        }
    }

    #[test]
    fn parts_share_search_but_compose_different_blueprint_sets() {
        let first = synthetic_record(401, [0; 6]);
        let second = synthetic_record(902, [0; 6]);
        let geodes24 = 24_i64 * 23 / 2;
        let geodes32 = 32_i64 * 31 / 2;
        assert_eq!(part1(&first).unwrap(), (401 * geodes24).to_string());
        assert_eq!(
            part1(&format!("{first}\n{second}")).unwrap(),
            ((401 + 902) * geodes24).to_string()
        );
        assert_eq!(part2(&first).unwrap(), geodes32.to_string());
        assert_eq!(
            part2(&format!("{first}\n{second}")).unwrap(),
            geodes32.pow(2).to_string()
        );
        let third = synthetic_record(1203, [0; 6]);
        let fourth = synthetic_record(1604, [u16::MAX; 6]);
        assert_eq!(
            part2(&format!("{first}\n{second}\n{third}\n{fourth}")).unwrap(),
            geodes32.pow(3).to_string()
        );
        assert!(part2(&format!("{first}\n{second}\n{third}\nbad blueprint")).is_err());
    }

    #[test]
    fn quality_arithmetic_is_checked() {
        assert!(matches!(
            part1(&synthetic_record(i64::MAX, [0; 6])),
            Err(SolveError::ArithmeticOverflow)
        ));
        let id = i64::MAX / (24 * 23 / 2);
        let first = synthetic_record(id, [0; 6]);
        let second = synthetic_record(id - 1, [0; 6]);
        assert!(part1(&first).is_ok());
        assert!(part1(&second).is_ok());
        assert!(matches!(
            part1(&format!("{first}\n{second}")),
            Err(SolveError::ArithmeticOverflow)
        ));
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    struct ReferenceState {
        remaining: u8,
        resources: [u16; 3],
        robots: [u8; 3],
    }

    // Enumerate every affordable build and waiting each minute. No production
    // caps, delayed-build flags, optimistic bound, or geode preference are used.
    // A new geode robot contributes remaining-1 over the rest of the horizon.
    fn exhaustive_geodes(
        costs: &[[u16; 3]; 4],
        state: ReferenceState,
        memo: &mut std::collections::HashMap<ReferenceState, u16>,
    ) -> u16 {
        if state.remaining <= 1 {
            return 0;
        }
        if let Some(&result) = memo.get(&state) {
            return result;
        }
        let mut best = 0;
        for action in 0..=4 {
            if action < 4
                && (0..3).any(|resource| state.resources[resource] < costs[action][resource])
            {
                continue;
            }
            let mut next = state;
            next.remaining -= 1;
            for resource in 0..3 {
                next.resources[resource] += u16::from(state.robots[resource]);
                if action < 4 {
                    next.resources[resource] -= costs[action][resource];
                }
            }
            if action < 3 {
                next.robots[action] += 1;
            }
            let geodes = if action == 3 {
                u16::from(state.remaining - 1)
            } else {
                0
            };
            best = best.max(geodes + exhaustive_geodes(costs, next, memo));
        }
        memo.insert(state, best);
        best
    }

    #[test]
    fn all_pruning_matches_exhaustive_small_blueprints() {
        let mut cases = 0;
        for mut encoding in 0..3usize.pow(6) {
            let mut amounts = [0u16; 6];
            for amount in &mut amounts {
                *amount = (encoding % 3) as u16;
                encoding /= 3;
            }
            let blueprint = Blueprint {
                id: 79,
                ore_robot: amounts[0],
                clay_robot: amounts[1],
                obsidian_robot: (amounts[2], amounts[3]),
                geode_robot: (amounts[4], amounts[5]),
                max_ore_cost: amounts[0].max(amounts[1]).max(amounts[2]).max(amounts[4]),
            };
            let costs = [
                [amounts[0], 0, 0],
                [amounts[1], 0, 0],
                [amounts[2], amounts[3], 0],
                [amounts[4], 0, amounts[5]],
            ];
            let mut memo = std::collections::HashMap::new();
            for minutes in 0..=8 {
                let expected = exhaustive_geodes(
                    &costs,
                    ReferenceState {
                        remaining: minutes,
                        resources: [0; 3],
                        robots: [1, 0, 0],
                    },
                    &mut memo,
                );
                assert_eq!(
                    max_geodes(&blueprint, minutes).unwrap(),
                    expected,
                    "costs={amounts:?}, minutes={minutes}"
                );
                cases += 1;
            }
        }
        assert_eq!(cases, 6561);
    }

    #[test]
    fn search_limits_return_errors_instead_of_partial_best_results() {
        let blueprint = Blueprint::parse(&synthetic_record(83, [0; 6])).unwrap();
        assert!(matches!(
            max_geodes_with_limit(&blueprint, 32, 0),
            Err(SolveError::ResourceLimit(_))
        ));
        // A zero-cost branch discovers a positive result before visiting alternatives.
        // Exhaustion must still propagate rather than return that partial best.
        assert!(matches!(
            max_geodes_with_limit(&blueprint, 32, 33),
            Err(SolveError::ResourceLimit(_))
        ));
        assert_eq!(max_geodes(&blueprint, 32).unwrap(), 32 * 31 / 2);
        assert_eq!(max_geodes_with_limit(&blueprint, 0, 1).unwrap(), 0);
        assert!(max_geodes(&blueprint, 33).is_err());
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
