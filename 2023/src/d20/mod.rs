use std::collections::{HashMap, HashSet, VecDeque};
use svutils::SolveError;

enum Node<'a> {
    FlipFlop(bool),
    Conjunction(HashMap<&'a str, bool>),
    Broadcaster,
}

fn parse_record(line: &str) -> Result<(&str, Node<'_>, Vec<&str>), SolveError> {
    let invalid = || SolveError::InvalidInput("expected a module and destination list".into());
    let (src, rest) = line.split_once(" -> ").ok_or_else(invalid)?;
    let (name, node) = if let Some(name) = src.strip_prefix('%') {
        (name, Node::FlipFlop(false))
    } else if let Some(name) = src.strip_prefix('&') {
        (name, Node::Conjunction(HashMap::new()))
    } else if src == "broadcaster" {
        (src, Node::Broadcaster)
    } else {
        return Err(invalid());
    };
    let valid_name =
        |name: &str| !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_lowercase());
    let connections: Vec<_> = rest.split(", ").collect();
    if !valid_name(name)
        || (name == "broadcaster" && src != "broadcaster")
        || connections.iter().any(|name| !valid_name(name))
    {
        return Err(invalid());
    }
    Ok((name, node, connections))
}

struct Network<'a> {
    outputs: HashMap<&'a str, Vec<&'a str>>,
    nodes: HashMap<&'a str, Node<'a>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pulse<'a> {
    source: &'a str,
    destination: &'a str,
    high: bool,
}

impl<'a> Network<'a> {
    fn parse(input: &'a str) -> Result<Self, SolveError> {
        let mut outputs = HashMap::new();
        let mut nodes = HashMap::new();
        if input.is_empty() {
            return Err(SolveError::EmptyInput);
        }
        for (index, line) in input.lines().enumerate() {
            let (name, node, connections) =
                parse_record(line).map_err(|error| error.at_line(index + 1))?;
            if outputs.insert(name, connections).is_some() {
                return Err(
                    SolveError::InvalidInput(format!("duplicate module {name}")).at_line(index + 1)
                );
            }
            nodes.insert(name, node);
        }
        if !nodes.contains_key("broadcaster") {
            return Err(SolveError::InvalidInput("missing broadcaster".into()));
        }
        for (&source, destinations) in &outputs {
            for &destination in destinations {
                if let Some(Node::Conjunction(memory)) = nodes.get_mut(destination) {
                    memory.insert(source, false);
                }
            }
        }
        Ok(Self { outputs, nodes })
    }

    fn ancestors(&self, target: &'a str) -> HashSet<&'a str> {
        let mut incoming: HashMap<&str, Vec<&str>> = HashMap::new();
        for (&source, destinations) in &self.outputs {
            for &destination in destinations {
                incoming.entry(destination).or_default().push(source);
            }
        }
        let mut ancestors = HashSet::new();
        let mut pending = vec![target];
        while let Some(name) = pending.pop() {
            if ancestors.insert(name) {
                if let Some(sources) = incoming.get(name) {
                    pending.extend(sources.iter().copied());
                }
            }
        }
        ancestors
    }

    fn snapshot(&self) -> Vec<bool> {
        self.snapshot_where(|_| true)
    }

    fn snapshot_where(&self, include: impl Fn(&str) -> bool) -> Vec<bool> {
        // HashMap iteration order is unspecified: give every state bit a stable slot.
        let mut nodes: Vec<_> = self
            .nodes
            .iter()
            .filter(|&(name, _)| include(name))
            .collect();
        nodes.sort_unstable_by_key(|&(name, _)| *name);
        let mut bits = Vec::new();
        for (_, node) in nodes {
            match node {
                Node::FlipFlop(on) => bits.push(*on),
                Node::Conjunction(memory) => {
                    let mut inputs: Vec<_> = memory.iter().collect();
                    inputs.sort_unstable_by_key(|&(name, _)| *name);
                    bits.extend(inputs.into_iter().map(|(_, high)| *high));
                }
                Node::Broadcaster => {}
            }
        }
        bits
    }

    #[cfg(test)]
    fn branch_cycle(&mut self, source: &'a str, parent: &str) -> Result<BranchCycle, SolveError> {
        use std::collections::hash_map::Entry;

        let names = self.ancestors(source);
        if names.contains(parent)
            || self
                .outputs
                .values()
                .any(|outputs| outputs.contains(&"broadcaster"))
        {
            return Err(SolveError::InvalidInput(
                "branch has external feedback".into(),
            ));
        }
        let Some(Node::Conjunction(memory)) = self.nodes.get(parent) else {
            return Err(SolveError::InvalidInput(
                "expected a conjunction parent".into(),
            ));
        };
        let Some(&remembered) = memory.get(source) else {
            return Err(SolveError::InvalidInput(
                "source does not feed the parent".into(),
            ));
        };
        let mut remembered = remembered;
        let mut seen = HashMap::new();
        let mut pulses = Vec::new();
        let mut queue = VecDeque::new();
        loop {
            let mut snapshot = self.snapshot_where(|name| names.contains(name));
            // The parent's memory of this branch can persist between presses.
            snapshot.push(remembered);
            match seen.entry(snapshot) {
                Entry::Vacant(entry) => {
                    entry.insert(pulses.len());
                }
                Entry::Occupied(entry) => {
                    let start = *entry.get();
                    return Ok(BranchCycle::from_recording(start, pulses, seen));
                }
            }
            let mut delivered = Vec::new();
            self.press(&mut queue, |pulse| {
                if pulse.source == source && pulse.destination == parent {
                    delivered.push(pulse.high);
                    remembered = pulse.high;
                }
                Ok(())
            })?;
            pulses.push(delivered);
        }
    }

    #[cfg(test)]
    fn branch_cycles(&mut self, parent: &str) -> Result<HashMap<&'a str, BranchCycle>, SolveError> {
        Ok(self.branch_collection(parent)?.cycles)
    }

    #[cfg(test)]
    fn branch_collection(&mut self, parent: &str) -> Result<BranchCollection<'a>, SolveError> {
        match self.collect_branches(parent, None)? {
            BranchAnalysis::Cycles(collection) => Ok(collection),
            BranchAnalysis::Answer(_) => unreachable!("no success destination is monitored"),
        }
    }

    fn collect_branches(
        &mut self,
        parent: &str,
        stop_at: Option<&str>,
    ) -> Result<BranchAnalysis<'a>, SolveError> {
        let Some(Node::Conjunction(memory)) = self.nodes.get(parent) else {
            return Err(SolveError::InvalidInput(
                "expected a conjunction parent".into(),
            ));
        };
        let mut active: Vec<_> = memory
            .iter()
            .map(|(&source, &remembered)| BranchRecorder {
                source,
                names: self.ancestors(source),
                remembered,
                seen: HashMap::new(),
                pulses: Vec::new(),
            })
            .collect();
        if active.iter().any(|branch| branch.names.contains(parent))
            || self
                .outputs
                .values()
                .any(|outputs| outputs.contains(&"broadcaster"))
        {
            return Err(SolveError::InvalidInput(
                "branch has external feedback".into(),
            ));
        }
        let mut completed = HashMap::new();
        let mut queue = VecDeque::new();
        let mut presses = 0usize;
        loop {
            active.retain_mut(|branch| {
                if let Some(cycle) = branch.check_state(self) {
                    completed.insert(branch.source, cycle);
                    false
                } else {
                    true
                }
            });
            if active.is_empty() {
                return Ok(BranchAnalysis::Cycles(BranchCollection {
                    cycles: completed,
                    presses,
                }));
            }
            for branch in &mut active {
                branch.pulses.push(Vec::new());
            }
            presses = presses
                .checked_add(1)
                .ok_or(SolveError::ArithmeticOverflow)?;
            let mut reached_target = false;
            self.press(&mut queue, |pulse| {
                reached_target |= stop_at == Some(pulse.destination) && !pulse.high;
                if pulse.destination == parent {
                    if let Some(branch) = active
                        .iter_mut()
                        .find(|branch| branch.source == pulse.source)
                    {
                        branch.pulses.last_mut().unwrap().push(pulse.high);
                        branch.remembered = pulse.high;
                    }
                }
                Ok(())
            })?;
            if reached_target {
                let answer = i64::try_from(presses).map_err(|_| SolveError::ArithmeticOverflow)?;
                return Ok(BranchAnalysis::Answer(answer));
            }
        }
    }

    fn restore_branch(
        &mut self,
        source: &'a str,
        parent: &str,
        snapshot: &[bool],
    ) -> Result<(), SolveError> {
        let names = self.ancestors(source);
        let invalid = || SolveError::InvalidInput("invalid branch snapshot".into());
        let mut bits = snapshot.iter().copied();
        let mut nodes: Vec<_> = self
            .nodes
            .iter_mut()
            .filter(|&(name, _)| names.contains(name))
            .collect();
        nodes.sort_unstable_by_key(|&(name, _)| *name);
        for (_, node) in nodes {
            match node {
                Node::FlipFlop(on) => *on = bits.next().ok_or_else(invalid)?,
                Node::Conjunction(memory) => {
                    let mut inputs: Vec<_> = memory.iter_mut().collect();
                    inputs.sort_unstable_by_key(|&(name, _)| *name);
                    for (_, high) in inputs {
                        *high = bits.next().ok_or_else(invalid)?;
                    }
                }
                Node::Broadcaster => {}
            }
        }
        let Some(Node::Conjunction(memory)) = self.nodes.get_mut(parent) else {
            return Err(invalid());
        };
        *memory.get_mut(source).ok_or_else(invalid)? = bits.next().ok_or_else(invalid)?;
        if bits.next().is_some() {
            return Err(invalid());
        }
        Ok(())
    }

    fn combine_branches(
        &mut self,
        parent: &str,
        collection: BranchCollection<'a>,
    ) -> Result<i64, SolveError> {
        let mut congruences = vec![Congruence {
            residue: 0,
            period: 1,
        }];
        // All inputs must be high at some point in a successful press. This is
        // necessary, not sufficient: replay below checks actual FIFO ordering.
        let mut branches: Vec<_> = collection.cycles.iter().collect();
        branches.sort_unstable_by_key(|&(source, _)| *source);
        for (_, branch) in &branches {
            let mut next = Vec::new();
            for &existing in &congruences {
                for phase in branch.possible_high_phases() {
                    if let Some(combined) = existing.combine(Congruence {
                        residue: phase as i128,
                        period: branch.period() as i128,
                    })? {
                        next.push(combined);
                    }
                }
            }
            next.sort_unstable();
            next.dedup();
            congruences = next;
        }
        let mut candidates = congruences
            .into_iter()
            .map(|phase| phase.first_at_or_after(collection.presses as i128))
            .collect::<Result<Vec<_>, _>>()?;
        candidates.sort_unstable();
        candidates.dedup();
        let mut queue = VecDeque::new();
        for completed_presses in candidates {
            for &(&source, branch) in &branches {
                self.restore_branch(
                    source,
                    parent,
                    &branch.states[branch.index_at(completed_presses)],
                )?;
            }
            let mut reached_rx = false;
            self.press(&mut queue, |pulse| {
                reached_rx |= pulse.destination == "rx" && !pulse.high;
                Ok(())
            })?;
            if reached_rx {
                let answer = completed_presses
                    .checked_add(1)
                    .ok_or(SolveError::ArithmeticOverflow)?;
                return i64::try_from(answer).map_err(|_| SolveError::ArithmeticOverflow);
            }
            // Every later press in this congruence class has the same relevant
            // state, so replaying the earliest member is sufficient.
        }
        Err(SolveError::InvalidInput(
            "rx never receives a low pulse".into(),
        ))
    }

    fn first_low_to_rx(&mut self) -> Result<i64, SolveError> {
        let mut seen = HashSet::new();
        let mut queue = VecDeque::new();
        let mut presses = 0i64;
        // Snapshots are taken only between presses, when the pulse queue is empty.
        while seen.insert(self.snapshot()) {
            presses = presses
                .checked_add(1)
                .ok_or(SolveError::ArithmeticOverflow)?;
            let mut reached_rx = false;
            self.press(&mut queue, |pulse| {
                reached_rx |= pulse.destination == "rx" && !pulse.high;
                Ok(())
            })?;
            if reached_rx {
                return Ok(presses);
            }
        }
        Err(SolveError::InvalidInput(
            "rx never receives a low pulse".into(),
        ))
    }

    fn press(
        &mut self,
        queue: &mut VecDeque<Pulse<'a>>,
        mut observe: impl FnMut(Pulse<'a>) -> Result<(), SolveError>,
    ) -> Result<(), SolveError> {
        queue.clear();
        queue.push_back(Pulse {
            source: "button",
            destination: "broadcaster",
            high: false,
        });
        while let Some(pulse) = queue.pop_front() {
            observe(pulse)?;
            let high = match self.nodes.get_mut(pulse.destination) {
                Some(Node::FlipFlop(_)) if pulse.high => continue,
                Some(Node::FlipFlop(on)) => {
                    *on = !*on;
                    *on
                }
                Some(Node::Conjunction(memory)) => {
                    memory.insert(pulse.source, pulse.high);
                    memory.values().any(|&high| !high)
                }
                Some(Node::Broadcaster) => pulse.high,
                None => continue,
            };
            queue.extend(
                self.outputs[pulse.destination]
                    .iter()
                    .map(|&destination| Pulse {
                        source: pulse.destination,
                        destination,
                        high,
                    }),
            );
        }
        Ok(())
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let mut network = Network::parse(input)?;
    let mut queue = VecDeque::new();
    let mut counts = [0i64; 2];
    for _ in 0..1000 {
        network.press(&mut queue, |pulse| {
            let count = &mut counts[usize::from(pulse.high)];
            *count = count.checked_add(1).ok_or(SolveError::ArithmeticOverflow)?;
            Ok(())
        })?;
    }
    counts[0]
        .checked_mul(counts[1])
        .map(|answer| answer.to_string())
        .ok_or(SolveError::ArithmeticOverflow)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut network = Network::parse(input)?;
    let parents: Vec<_> = network
        .outputs
        .iter()
        .filter(|(_, destinations)| destinations.contains(&"rx"))
        .map(|(&name, _)| name)
        .collect();
    let [parent] = parents.as_slice() else {
        return network.first_low_to_rx().map(|answer| answer.to_string());
    };
    let Some(Node::Conjunction(memory)) = network.nodes.get(parent) else {
        return network.first_low_to_rx().map(|answer| answer.to_string());
    };
    if memory.is_empty() {
        return network.first_low_to_rx().map(|answer| answer.to_string());
    }
    let branches: Vec<_> = memory.keys().map(|&name| network.ancestors(name)).collect();
    let has_feedback = branches.iter().any(|branch| branch.contains(parent));
    let shares_state = branches.iter().enumerate().any(|(index, branch)| {
        branches[..index].iter().any(|other| {
            branch
                .intersection(other)
                .any(|&name| name != "broadcaster")
        })
    });
    let broadcaster_feedback = network
        .outputs
        .values()
        .any(|outputs| outputs.contains(&"broadcaster"));
    if has_feedback || shares_state || broadcaster_feedback || network.nodes.contains_key("rx") {
        return network.first_low_to_rx().map(|answer| answer.to_string());
    }
    let answer = match network.collect_branches(parent, Some("rx"))? {
        BranchAnalysis::Answer(answer) => answer,
        BranchAnalysis::Cycles(collection) => network.combine_branches(parent, collection)?,
    };
    Ok(answer.to_string())
}

enum BranchAnalysis<'a> {
    Answer(i64),
    Cycles(BranchCollection<'a>),
}

struct BranchCollection<'a> {
    cycles: HashMap<&'a str, BranchCycle>,
    presses: usize,
}

struct BranchRecorder<'a> {
    source: &'a str,
    names: HashSet<&'a str>,
    remembered: bool,
    seen: HashMap<Vec<bool>, usize>,
    pulses: Vec<Vec<bool>>,
}

impl BranchRecorder<'_> {
    fn check_state(&mut self, network: &Network<'_>) -> Option<BranchCycle> {
        use std::collections::hash_map::Entry;

        let mut snapshot = network.snapshot_where(|name| self.names.contains(name));
        snapshot.push(self.remembered);
        match self.seen.entry(snapshot) {
            Entry::Vacant(entry) => {
                entry.insert(self.pulses.len());
                None
            }
            Entry::Occupied(entry) => {
                let start = *entry.get();
                Some(BranchCycle::from_recording(
                    start,
                    std::mem::take(&mut self.pulses),
                    std::mem::take(&mut self.seen),
                ))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BranchCycle {
    // The repeating suffix begins after `start` completed presses.
    start: usize,
    // pulses[i] contains all deliveries during press i + 1, in delivery order.
    pulses: Vec<Vec<bool>>,
    // State just before each corresponding press; final bit is parent memory.
    states: Vec<Vec<bool>>,
}

impl BranchCycle {
    fn from_recording(
        start: usize,
        pulses: Vec<Vec<bool>>,
        seen: HashMap<Vec<bool>, usize>,
    ) -> Self {
        let mut states = vec![Vec::new(); pulses.len()];
        for (snapshot, index) in seen {
            states[index] = snapshot;
        }
        Self {
            start,
            pulses,
            states,
        }
    }

    fn index_at(&self, completed_presses: i128) -> usize {
        if completed_presses < self.states.len() as i128 {
            completed_presses as usize
        } else {
            // The remainder is bounded by the period, which is a Vec length.
            self.start + ((completed_presses - self.start as i128) % self.period() as i128) as usize
        }
    }

    fn possible_high_phases(&self) -> impl Iterator<Item = usize> + '_ {
        (self.start..self.states.len())
            .filter(|&index| {
                self.states[index].last() == Some(&true) || self.pulses[index].contains(&true)
            })
            .map(|index| index % self.period())
    }

    fn period(&self) -> usize {
        self.pulses.len() - self.start
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// All solutions are residue + k * period; 0 <= residue < period.
struct Congruence {
    residue: i128,
    period: i128,
}

// Returns gcd(a, b) and the coefficient x in a*x + b*y = gcd(a, b).
fn extended_gcd(mut a: i128, mut b: i128) -> Result<(i128, i128), SolveError> {
    let (mut x, mut next_x) = (1_i128, 0_i128);
    while b != 0 {
        let quotient = a / b;
        (a, b) = (b, a % b);
        let updated = quotient
            .checked_mul(next_x)
            .and_then(|product| x.checked_sub(product))
            .ok_or(SolveError::ArithmeticOverflow)?;
        (x, next_x) = (next_x, updated);
    }
    Ok((a, x))
}

impl Congruence {
    fn combine(self, other: Self) -> Result<Option<Self>, SolveError> {
        let (gcd, inverse) = extended_gcd(self.period, other.period)?;
        let difference = other.residue - self.residue;
        if difference % gcd != 0 {
            return Ok(None);
        }
        // Divide out shared factors, then solve for the number of our periods to add.
        let reduced = other.period / gcd;
        let multiplier = (difference / gcd)
            .rem_euclid(reduced)
            .checked_mul(inverse.rem_euclid(reduced))
            .ok_or(SolveError::ArithmeticOverflow)?
            % reduced;
        let period = self
            .period
            .checked_mul(reduced)
            .ok_or(SolveError::ArithmeticOverflow)?;
        let residue = self
            .period
            .checked_mul(multiplier)
            .and_then(|offset| self.residue.checked_add(offset))
            .ok_or(SolveError::ArithmeticOverflow)?
            % period;
        Ok(Some(Self { residue, period }))
    }

    fn first_at_or_after(self, minimum: i128) -> Result<i128, SolveError> {
        if self.residue >= minimum {
            return Ok(self.residue);
        }
        let difference = minimum - self.residue;
        let periods = difference / self.period + i128::from(difference % self.period != 0);
        self.period
            .checked_mul(periods)
            .and_then(|offset| self.residue.checked_add(offset))
            .ok_or(SolveError::ArithmeticOverflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 20;

    #[test]
    fn successful_collection_stops_after_the_first_winning_press() {
        let input = "broadcaster -> a, b, extra\n%a -> con\n%b -> c\n%c -> con\n&con -> rx\n%extra -> more\n%more -> sink";
        let mut network = Network::parse(input).unwrap();
        let result = network.collect_branches("con", Some("rx")).unwrap();
        assert!(matches!(result, BranchAnalysis::Answer(3)));
        let mut reference = Network::parse(input).unwrap();
        for _ in 0..3 {
            reference.press(&mut VecDeque::new(), |_| Ok(())).unwrap();
        }
        // The winning press was fully drained; no later press was started.
        assert_eq!(network.snapshot(), reference.snapshot());
        let completed = Network::parse(input)
            .unwrap()
            .branch_collection("con")
            .unwrap();
        assert!(completed.presses > 3);
        assert_eq!(part2(input).unwrap(), "3");
    }

    #[test]
    fn candidate_replay_rejects_high_pulses_that_never_overlap() {
        let input = "broadcaster -> a, a, b, b\n%a -> con\n%b -> con\n&con -> rx";
        assert!(part2(input)
            .unwrap_err()
            .to_string()
            .contains("never receives a low"));
        assert_eq!(
            part2("broadcaster -> a, b\n%a -> con\n%b -> inv\n&inv -> con\n&con -> rx").unwrap(),
            "3"
        );
    }

    #[test]
    fn branch_composition_matches_direct_search_on_small_networks() {
        fn branch(prefix: char, length: usize, ring: bool, invert: bool) -> Vec<String> {
            let name = |index| format!("{prefix}{}", char::from(b'a' + index as u8));
            let mut lines = Vec::new();
            for index in 0..length {
                let mut destinations = if index + 1 < length {
                    vec![name(index + 1)]
                } else {
                    vec![if invert {
                        format!("{prefix}z")
                    } else {
                        "con".into()
                    }]
                };
                if ring && index + 1 == length {
                    destinations.push(name(0));
                }
                lines.push(format!("%{} -> {}", name(index), destinations.join(", ")));
            }
            if invert {
                lines.push(format!("&{prefix}z -> con"));
            }
            lines
        }
        let shapes: Vec<_> = (1..=3)
            .flat_map(|length| {
                [false, true]
                    .into_iter()
                    .filter(move |&ring| !ring || length > 1)
                    .flat_map(move |ring| [false, true].map(|invert| (length, ring, invert)))
            })
            .collect();
        for &(left_len, left_ring, left_invert) in &shapes {
            for &(right_len, right_ring, right_invert) in &shapes {
                let mut lines = vec!["broadcaster -> aa, ba, extra".into()];
                lines.extend(branch('a', left_len, left_ring, left_invert));
                lines.extend(branch('b', right_len, right_ring, right_invert));
                lines.extend(["&con -> rx".into(), "%extra -> sink".into()]);
                let input = lines.join("\n");
                let direct = Network::parse(&input).unwrap().first_low_to_rx().ok();
                assert_eq!(
                    part2(&input).ok(),
                    direct.map(|value| value.to_string()),
                    "{input}"
                );

                // Exercise periodic composition even when the first solution was
                // already seen while collecting cycles. Compare the next solution.
                let mut network = Network::parse(&input).unwrap();
                let collection = network.branch_collection("con").unwrap();
                let mut reference = Network::parse(&input).unwrap();
                for _ in 0..collection.presses {
                    reference.press(&mut VecDeque::new(), |_| Ok(())).unwrap();
                }
                let expected = reference
                    .first_low_to_rx()
                    .ok()
                    .map(|relative| collection.presses as i64 + relative);
                assert_eq!(
                    network.combine_branches("con", collection).ok(),
                    expected,
                    "{input}"
                );
            }
        }
    }

    #[test]
    fn congruences_match_small_integer_enumeration() {
        for a in 1..=8 {
            for b in 1..=8 {
                for left in 0..a {
                    for right in 0..b {
                        let combined = Congruence {
                            residue: left,
                            period: a,
                        }
                        .combine(Congruence {
                            residue: right,
                            period: b,
                        })
                        .unwrap();
                        let expected =
                            (0..a * b).find(|value| value % a == left && value % b == right);
                        assert_eq!(combined.map(|phase| phase.residue), expected);
                        if let Some(phase) = combined {
                            for minimum in 0..=a * b {
                                let expected = (minimum..=minimum + a * b)
                                    .find(|value| value % a == left && value % b == right)
                                    .unwrap();
                                assert_eq!(phase.first_at_or_after(minimum).unwrap(), expected);
                            }
                        }
                    }
                }
            }
        }
        assert!(matches!(
            Congruence {
                residue: 0,
                period: i128::MAX
            }
            .combine(Congruence {
                residue: 0,
                period: 2
            }),
            Err(SolveError::ArithmeticOverflow)
        ));
    }

    #[test]
    fn recorded_states_replay_complete_future_presses() {
        for input in [
            "broadcaster -> a, b\n%a -> con\n%b -> c\n%c -> con\n&con -> rx",
            "broadcaster -> a, a, b\n%a -> inv\n&inv -> con\n%b -> con\n&con -> rx",
            "broadcaster -> a, b\n%a -> con\n%b -> inv\n&inv -> con\n&con -> rx",
        ] {
            let cycles = Network::parse(input).unwrap().branch_cycles("con").unwrap();
            let mut sequential = Network::parse(input).unwrap();
            for completed in 0..40 {
                let mut restored = Network::parse(input).unwrap();
                for (&source, cycle) in &cycles {
                    restored
                        .restore_branch(source, "con", &cycle.states[cycle.index_at(completed)])
                        .unwrap();
                }
                let mut expected = Vec::new();
                sequential
                    .press(&mut VecDeque::new(), |pulse| {
                        expected.push(pulse);
                        Ok(())
                    })
                    .unwrap();
                let mut actual = Vec::new();
                restored
                    .press(&mut VecDeque::new(), |pulse| {
                        actual.push(pulse);
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(actual, expected, "after {completed} completed presses");
                assert_eq!(restored.snapshot(), sequential.snapshot());
            }
        }
    }

    #[test]
    fn simultaneous_recording_matches_separate_runs() {
        for input in [
            "broadcaster -> a, b\n%a -> con\n%b -> c\n%c -> con\n&con -> rx",
            "broadcaster -> a, a, b\n%a -> inv\n&inv -> con\n%b -> con\n&con -> rx",
        ] {
            let together = Network::parse(input).unwrap().branch_cycles("con").unwrap();
            assert_eq!(together.len(), 2);
            for (source, cycle) in together {
                let separate = Network::parse(input)
                    .unwrap()
                    .branch_cycle(source, "con")
                    .unwrap();
                assert_eq!(cycle, separate);
            }
        }
        for input in [
            "broadcaster -> a\n%a -> con\n&con -> a, rx",
            "broadcaster -> a\n%a -> inv, broadcaster\n&inv -> con\n&con -> rx",
        ] {
            assert!(Network::parse(input).unwrap().branch_cycles("con").is_err());
        }
    }

    #[test]
    fn branch_cycles_record_periods_offsets_and_all_deliveries() {
        let mut network = Network::parse("broadcaster -> a\n%a -> con\n&con -> rx").unwrap();
        let cycle = network.branch_cycle("a", "con").unwrap();
        assert_eq!(cycle.start, 0);
        assert_eq!(cycle.period(), 2);
        assert_eq!(cycle.pulses, [vec![true], vec![false]]);

        let mut network =
            Network::parse("broadcaster -> a\n%a -> b\n%b -> con\n&con -> rx").unwrap();
        let cycle = network.branch_cycle("b", "con").unwrap();
        assert_eq!(cycle.start, 0);
        assert_eq!(cycle.period(), 4);
        assert_eq!(cycle.pulses, [vec![], vec![true], vec![], vec![false]]);

        // A one-input conjunction ends each press high: the initial low memory
        // is a transient. Both deliveries in a press must be retained.
        let mut network =
            Network::parse("broadcaster -> a, a\n%a -> inv\n&inv -> con\n&con -> rx").unwrap();
        let cycle = network.branch_cycle("inv", "con").unwrap();
        assert_eq!(cycle.start, 1);
        assert_eq!(cycle.period(), 1);
        assert_eq!(cycle.pulses, [vec![false, true], vec![false, true]]);
    }

    #[test]
    fn branch_cycles_reject_feedback() {
        for (input, source) in [
            ("broadcaster -> a\n%a -> con\n&con -> a, rx", "a"),
            (
                "broadcaster -> a\n%a -> inv, broadcaster\n&inv -> con\n&con -> rx",
                "inv",
            ),
        ] {
            let mut network = Network::parse(input).unwrap();
            assert!(network
                .branch_cycle(source, "con")
                .unwrap_err()
                .to_string()
                .contains("feedback"));
        }
    }

    #[test]
    fn ancestor_names_survive_network_mutation() {
        let input = "broadcaster -> a, b\n%a -> con\n%b -> con\n&con -> rx";
        let mut network = Network::parse(input).unwrap();
        let left = network.ancestors("a");
        let right = network.ancestors("b");
        network.press(&mut VecDeque::new(), |_| Ok(())).unwrap();
        assert_eq!(left, HashSet::from(["a", "broadcaster"]));
        assert_eq!(right, HashSet::from(["b", "broadcaster"]));
        assert_eq!(
            left.intersection(&right).copied().collect::<HashSet<_>>(),
            HashSet::from(["broadcaster"])
        );
        assert_eq!(
            network.ancestors("rx"),
            HashSet::from(["rx", "con", "a", "b", "broadcaster"])
        );
    }

    #[test]
    fn ancestors_handle_shared_state_and_feedback() {
        let shared =
            Network::parse("broadcaster -> a\n%a -> b, c\n%b -> con\n%c -> con\n&con -> rx")
                .unwrap();
        assert!(shared
            .ancestors("b")
            .intersection(&shared.ancestors("c"))
            .any(|&name| name == "a"));
        let feedback = Network::parse("broadcaster -> a\n%a -> con\n&con -> a, rx").unwrap();
        assert_eq!(
            feedback.ancestors("a"),
            HashSet::from(["a", "con", "broadcaster"])
        );
        assert_eq!(
            part2("broadcaster -> a\n%a -> b, c\n%b -> con\n%c -> con\n&con -> rx").unwrap(),
            "2"
        );
        assert_eq!(
            part2("broadcaster -> a\n%a -> con\n&con -> a, rx").unwrap(),
            "1"
        );
    }

    #[test]
    fn exact_search_observes_rx_and_detects_no_solution() {
        for (input, expected) in [
            ("broadcaster -> rx", 1),
            ("broadcaster -> a\n%a -> rx", 2),
            ("broadcaster -> a\n%a -> b\n%b -> rx", 4),
            // On press 3, a's high arrives before inv clears its remembered high.
            (
                "broadcaster -> a, b\n%a -> con\n%b -> inv\n&inv -> con\n&con -> rx",
                3,
            ),
        ] {
            let mut network = Network::parse(input).unwrap();
            assert_eq!(network.first_low_to_rx().unwrap(), expected);
        }
        for input in ["broadcaster -> output", "broadcaster -> con\n&con -> rx"] {
            let error = Network::parse(input)
                .unwrap()
                .first_low_to_rx()
                .unwrap_err();
            assert!(error.to_string().contains("never receives a low"));
        }
        assert_eq!(part2("broadcaster -> a\n%a -> rx").unwrap(), "2");
    }

    #[test]
    fn snapshots_are_owned_and_include_conjunction_memory() {
        let input = "broadcaster -> a, con\n%a -> con\n&con -> output";
        let mut first = Network::parse(input).unwrap();
        let second = Network::parse("&con -> output\n%a -> con\nbroadcaster -> a, con").unwrap();
        let before = first.snapshot();
        assert_eq!(before, second.snapshot());
        if let Some(Node::Conjunction(memory)) = first.nodes.get_mut("con") {
            *memory.get_mut("a").unwrap() = true;
        }
        assert_ne!(before, first.snapshot());
        assert_eq!(before, second.snapshot());
    }

    #[test]
    fn part1_finishes_all_presses_without_rx_assumptions() {
        assert_eq!(
            part1("broadcaster -> a\n%a -> output\n").unwrap(),
            "1250000"
        );
        assert_eq!(
            part1("broadcaster -> a\n%a -> con\n&con -> rx\n").unwrap(),
            "3000000"
        );
        assert_eq!(part1("broadcaster -> output").unwrap(), "0");
        assert!(part1("%a -> output")
            .unwrap_err()
            .to_string()
            .contains("missing broadcaster"));
    }

    #[test]
    fn press_observes_fifo_deliveries_and_keeps_state() {
        let mut network =
            Network::parse("broadcaster -> a, b\n%a -> con\n%b -> con\n&con -> output").unwrap();
        let mut queue = VecDeque::new();
        let mut delivered = Vec::new();
        network
            .press(&mut queue, |pulse| {
                delivered.push((pulse.source, pulse.destination, pulse.high));
                Ok(())
            })
            .unwrap();
        assert_eq!(
            delivered,
            [
                ("button", "broadcaster", false),
                ("broadcaster", "a", false),
                ("broadcaster", "b", false),
                ("a", "con", true),
                ("b", "con", true),
                ("con", "output", true),
                ("con", "output", false),
            ]
        );
        assert!(queue.is_empty());
        delivered.clear();
        network
            .press(&mut queue, |pulse| {
                delivered.push((pulse.source, pulse.destination, pulse.high));
                Ok(())
            })
            .unwrap();
        assert_eq!(
            &delivered[3..],
            [
                ("a", "con", false),
                ("b", "con", false),
                ("con", "output", true),
                ("con", "output", true),
            ]
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn broadcaster_forwards_received_high_pulses() {
        let mut network = Network::parse("broadcaster -> a\n%a -> broadcaster, output").unwrap();
        let mut delivered = Vec::new();
        network
            .press(&mut VecDeque::new(), |pulse| {
                delivered.push((pulse.source, pulse.destination, pulse.high));
                Ok(())
            })
            .unwrap();
        assert_eq!(delivered.last(), Some(&("broadcaster", "a", true)));
    }

    #[test]
    fn malformed_records_report_the_line() {
        for record in [
            "",
            "bad",
            " -> a",
            "% -> a",
            "%a -> ",
            "%a -> b,",
            "banana -> a",
            "%broadcaster -> a",
            "&broadcaster -> a",
        ] {
            let input = format!("broadcaster -> a\n{record}\n%a -> rx");
            let error = part1(&input).unwrap_err();
            assert!(matches!(error, SolveError::AtLine { line: 2, .. }));
        }
        assert!(matches!(part1(""), Err(SolveError::EmptyInput)));
        assert!(part1("broadcaster -> a\n%a -> rx\n%a -> rx")
            .unwrap_err()
            .to_string()
            .starts_with("line 3:"));
    }

    #[test]
    fn records_accept_crlf_and_borrow_names() {
        let input = "broadcaster -> a\r\n%a -> output\r\n";
        let records = input
            .lines()
            .map(parse_record)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[1].0, "a");
        assert_eq!(records[1].2, ["output"]);
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
