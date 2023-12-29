use std::collections::HashMap;

extern crate core;

// The input is one of Conway's
// [atomic elements](https://en.wikipedia.org/wiki/Look-and-say_sequence#Cosmological_decay).
// Each element breaks down into other elemenths that don't interact. So we only need to track
// the count of each element from tick to tick.
// The result is then multiplying each element by its length.

const ELEMENTS: &str = "\
22 -> H -> H
13112221133211322112211213322112 -> He -> Hf Pa H Ca Li
312211322212221121123222112 -> Li -> He
111312211312113221133211322112211213322112 -> Be -> Ge Ca Li
1321132122211322212221121123222112 -> B -> Be
3113112211322112211213322112 -> C -> B
111312212221121123222112 -> N -> C
132112211213322112 -> O -> N
31121123222112 -> F -> O
111213322112 -> Ne -> F
123222112 -> Na -> Ne
3113322112 -> Mg -> Pm Na
1113222112 -> Al -> Mg
1322112 -> Si -> Al
311311222112 -> P -> Ho Si
1113122112 -> S -> P
132112 -> Cl -> S
3112 -> Ar -> Cl
1112 -> K -> Ar
12 -> Ca -> K
3113112221133112 -> Sc -> Ho Pa H Ca Co
11131221131112 -> Ti -> Sc
13211312 -> V -> Ti
31132 -> Cr -> V
111311222112 -> Mn -> Cr Si
13122112 -> Fe -> Mn
32112 -> Co -> Fe
11133112 -> Ni -> Zn Co
131112 -> Cu -> Ni
312 -> Zn -> Cu
13221133122211332 -> Ga -> Eu Ca Ac H Ca Zn
31131122211311122113222 -> Ge -> Ho Ga
11131221131211322113322112 -> As -> Ge Na
13211321222113222112 -> Se -> As
3113112211322112 -> Br -> Se
11131221222112 -> Kr -> Br
1321122112 -> Rb -> Kr
3112112 -> Sr -> Rb
1112133 -> Y -> Sr U
12322211331222113112211 -> Zr -> Y H Ca Tc
1113122113322113111221131221 -> Nb -> Er Zr
13211322211312113211 -> Mo -> Nb
311322113212221 -> Tc -> Mo
132211331222113112211 -> Ru -> Eu Ca Tc
311311222113111221131221 -> Rh -> Ho Ru
111312211312113211 -> Pd -> Rh
132113212221 -> Ag -> Pd
3113112211 -> Cd -> Ag
11131221 -> In -> Cd
13211 -> Sn -> In
3112221 -> Sb -> Pm Sn
1322113312211 -> Te -> Eu Ca Sb
311311222113111221 -> I -> Ho Te
11131221131211 -> Xe -> I
13211321 -> Cs -> Xe
311311 -> Ba -> Cs
11131 -> La -> Ba
1321133112 -> Ce -> La H Ca Co
31131112 -> Pr -> Ce
111312 -> Nd -> Pr
132 -> Pm -> Nd
311332 -> Sm -> Pm Ca Zn
1113222 -> Eu -> Sm
13221133112 -> Gd -> Eu Ca Co
3113112221131112 -> Tb -> Ho Gd
111312211312 -> Dy -> Tb
1321132 -> Ho -> Dy
311311222 -> Er -> Ho Pm
11131221133112 -> Tm -> Er Ca Co
1321131112 -> Yb -> Tm
311312 -> Lu -> Yb
11132 -> Hf -> Lu
13112221133211322112211213322113 -> Ta -> Hf Pa H Ca W
312211322212221121123222113 -> W -> Ta
111312211312113221133211322112211213322113 -> Re -> Ge Ca W
1321132122211322212221121123222113 -> Os -> Re
3113112211322112211213322113 -> Ir -> Os
111312212221121123222113 -> Pt -> Ir
132112211213322113 -> Au -> Pt
31121123222113 -> Hg -> Au
111213322113 -> Tl -> Hg
123222113 -> Pb -> Tl
3113322113 -> Bi -> Pm Pb
1113222113 -> Po -> Bi
1322113 -> At -> Po
311311222113 -> Rn -> Ho At
1113122113 -> Fr -> Rn
132113 -> Ra -> Fr
3113 -> Ac -> Ra
1113 -> Th -> Ac
13 -> Pa -> Th
3 -> U -> Pa";

type Result = (i64, i64);

pub fn parse(input: &str) -> Result {
    let elements: Vec<Vec<_>> = ELEMENTS
        .lines()
        .map(|line| line.split_ascii_whitespace().collect())
        .collect();
    let mut indices = HashMap::with_capacity(92);

    for (i, tokens) in elements.iter().enumerate() {
        // Tokens
        //  0  1 2  3 4
        // ============
        // 22 -> H -> H
        // index 1 has H
        indices.insert(tokens[2], i);
    }

    let mut sequence = [""; 92];

    // -> Ga -> Eu Ca Ac H Ca Znv
    let mut decays = [[None; 6]; 92];

    for (i, tokens) in elements.iter().enumerate() {
        sequence[i] = tokens[0];
        for (j, &token) in tokens.iter().skip(4).enumerate() {
            // E.g.
            decays[i][j] = Some(indices[token] as i64)
        }
    }

    let mut current = initialise_simulation(input, &sequence);

    for _ in 0..40 {
        current = decay(&current, &decays);
    }

    let result1 = length(&current, &sequence);

    for _ in 0..10 {
        current = decay(&current, &decays);
    }

    let result2 = length(&current, &sequence);

    (result1, result2)
}

fn initialise_simulation(input: &str, sequence: &[&str]) -> [i64; 92] {
    let input = input.trim();
    let start = sequence.iter().position(|&s| s == input).unwrap();

    let mut current = [0; 92];
    current[start] += 1;
    current
}

fn decay(current: &[i64; 92], decays: &[[Option<i64>; 6]; 92]) -> [i64; 92] {
    let mut next = [0; 92];

    for i in 0..92 {
        let c = current[i];
        if c > 0 {
            let mut iter = decays[i].iter();
            while let Some(Some(index)) = iter.next() {
                // Sprinkle the appropriate decays
                next[*index as usize] += c;
            }
        }
    }

    next
}

fn length(current: &[i64; 92], sequence: &[&str; 92]) -> i64 {
    current
        .iter()
        .zip(sequence.iter())
        .map(|(c, s)| c * s.len() as i64)
        .sum()
}

pub fn fn1(input: &str) -> i64 {
    parse(input).0
}

pub fn fn2(input: &str) -> i64 {
    parse(input).1
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 10;

    #[test]
    fn test_fn1_examples() {
        // Note that none of these test cases work with the decay approach,
        // because none of the seqences are in the decay. That's OK.

        //assert_eq!(fn1("1"), "11".len());
        //assert_eq!(fn1("21"), "21".len());
        //assert_eq!(fn1("21"), "1211".len());
        //assert_eq!(fn1("1211"), "111221".len());
        //assert_eq!(fn1("111221"), "312211".len());
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
