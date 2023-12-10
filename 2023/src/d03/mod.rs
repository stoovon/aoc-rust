extern crate core;

use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
struct Coords {
    x: i64,
    y: i64,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
struct ComponentCoordinates {
    c: Coords,
    symbol: String,
}

#[derive(Debug, Clone)]
struct Component {
    coordinates: ComponentCoordinates,
    nearby_nums: Vec<i64>,
}

#[derive(Debug, Clone)]
struct UnlinkedComponent {
    coordinates: ComponentCoordinates,
}

#[derive(Debug, Clone)]
struct UnlinkedNumber {
    places_to_scan: Vec<Coords>,
    value: i64,
}

#[derive(Debug)]
struct ParseResult {
    components: Vec<UnlinkedComponent>,
    numbers: Vec<UnlinkedNumber>
}

fn chebyshev(uln: UnlinkedNumber, coords: HashMap<Coords, Component>) -> Option<Component> {
    // ⚠️ Probably over-engineering but this could get really nasty if we 
    // have to deal with vertical lines and Chebyshev distance in part 2.
    // If that were to happen we would just beef up this impl.

    let _uln_diag = uln.value;
    let _pts_diag = uln.places_to_scan.clone();

    for loc in uln.places_to_scan.iter() {
        // Search pattern
        // X/Y | -1 | 0  | +1
        // -1  | UL | UM | UR
        // 0   | ML | ?? | MR (MM is loc)
        // +1  | LL | LM | LR

        for xmod in loc.x -1..loc.x+2 {
            for ymod in loc.y - 1..loc.y+2 {
                if xmod == loc.x && ymod == loc.y {
                    continue
                }

                match coords.get(
                    &Coords { x: xmod, y: ymod }
                ) {
                    Some(component_hit) => {
                        return Some(component_hit.clone())
                    },
                    _ => (),
                }
            }
        }
    }

    None
}

fn link(input: ParseResult) -> Vec<Component> {
    // Build with HashMap for O(1), then to Vec at the end.
    let mut coords: HashMap<Coords, Component> = HashMap::new();

    for comp in input.components.iter() {
        // This allows us to quickly scan for nearby components when iterating over
        // unlinked numbers.

        // We could avoid this allocation by scanning before and after the line,
        // but it might be harder to understand than reasoning about the types.

        coords.insert(
            comp.coordinates.c.clone(), 
            Component { 
                coordinates: comp.coordinates.clone(), 
                nearby_nums: Vec::new()
            },
        );
    }

    let mut bindings: HashMap<ComponentCoordinates, Vec<i64>> = HashMap::new();

    for uln in input.numbers.iter() {
        // This clone probably kills performance
        match chebyshev(uln.clone(), coords.clone()) {
            Some(component_hit) => {
                let key = component_hit.coordinates.clone();

                match bindings.get_mut(&key) {
                    Some(bindings) => {
                        bindings.push(uln.value);
                    },
                    None => {
                        bindings.insert(key, vec![uln.value]);
                    }
                }
            },
            None => {
                // Not a part number; throw it on the floor.
            }
        }
    }

    bindings.into_iter().map(|(coords, vals)| {
        Component {
            coordinates: coords,
            nearby_nums: vals,
        }
    }).collect_vec()
}


fn parse_lines(input: &str) -> ParseResult {
    // Digits again becuase e.g. `+101`, `-101` (could also possibly use abs)
    let digits: Vec<char> = "1234567890".chars().collect();

    // One of the lines of the input ends with a number so let's make things simpler
    let terminator: &str = "x";

    let mut x: i64;
    let mut y = -1;

    let mut unlinked_components: Vec<UnlinkedComponent> = Vec::new();
    let mut unlinked_numbers: Vec<UnlinkedNumber> = Vec::new();

    for line in input.lines() {
        x = -1;
        y += 1;

        let mut building_part_number = false;
        let mut built_part_number = "".to_string();
        let mut places_to_scan = Vec::new();

        let terminated_line = line.to_owned() + terminator;

        for c in terminated_line.chars() {
            x += 1;
            if digits.contains(&c) {
                building_part_number = true;
                built_part_number.push(c);
                places_to_scan.push(Coords{
                    x,
                    y,
                })
            } else {
                if building_part_number {
                    unlinked_numbers.push(
                        UnlinkedNumber {
                            places_to_scan: places_to_scan.clone(),
                            value: built_part_number.parse::<i64>().unwrap_or_default(),  
                        },
                    );

                    building_part_number = false;
                    built_part_number = "".to_string();
                    places_to_scan = Vec::new();
                }

                if c == terminator.chars().next().unwrap_or_default() {
                    break;
                }

                if c != '.' {
                    // Build a component
                    unlinked_components.push(
                        UnlinkedComponent { 
                            coordinates: ComponentCoordinates { 
                                c: Coords { 
                                    x, 
                                    y 
                                },
                                symbol: c.to_string(),
                            }, 
                        }
                    )
                }
            }
        }
    }

    ParseResult {
        components: unlinked_components,
        numbers: unlinked_numbers,
    }
}

pub fn fn1(input: &str) -> i64 {
    let mut result = 0;

    let pr = parse_lines(input);
    let linked = link(pr);

    for component in linked.iter() {
        // All detected characters (might want to do better)

        // Part 1
        // CLUE: Probably going to be asked to filter since the symbols are different

        // *:467
        // *:35
        // #:633
        // *:617
        // +:592
        // *:755
        // *:598
        // $:664
        
        // CLUE: Two of the * in the sample are near two part numbers. 617 is the only match in the example that's not near 2 part numbers...
        // ... * means multiply in several languages...
        // ... PUZZLE IS CALLED GEAR RATIOS

        // Let's guess Part 2. First and only time I reckon I can do that this AoC (can always take the minute to read the question if I'm wrong).

        if [
            "*", "@", "+", "/", "=", "$", "&", "-", "#", "%"
        ]
            .iter()
            .map(|x| x.to_string())
            .collect_vec()
            .contains(&component.coordinates.symbol) {
                if component.nearby_nums.len() > 0 {
                    for part_number in component.nearby_nums.iter() {
                        result += part_number
                    }
                }
        }
    }
    
    result
}

pub fn fn2(input: &str) -> i64 {
    let mut result = 0;

    let pr = parse_lines(input);
    let linked = link(pr);

    for component in linked.iter() {
        // All detected characters (might want to do better)

        if [
            "*",
        ]
            .iter()
            .map(|x| x.to_string())
            .collect_vec()
            .contains(&component.coordinates.symbol) {
                if component.nearby_nums.len() == 2 { // LOOK AT THE EVIL 617. Called it!
                    let mut st = 1;
                    for part_number in component.nearby_nums.iter() {
                        st *= part_number
                    }

                    result += st
                }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    #[test]
    fn test_fn1_example() {
        assert_eq!(fn1(include_str!("example.txt")), load_spec(include_str!("example-spec.1.txt")));
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(fn1(include_str!("input.txt")), load_spec(include_str!("input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(fn2(include_str!("example.txt")), load_spec(include_str!("example-spec.2.txt")));
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(fn2(include_str!("input.txt")), load_spec(include_str!("input-spec.2.txt")));
    }
}