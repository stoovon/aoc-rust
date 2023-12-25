extern crate core;

const ARROWS: [u8; 4] = *b"^>v<";

pub struct Grid {
    data: Box<[u8]>,
    offset: usize,
}
impl Grid {
    fn from_str(s: &str) -> Self {
        let mut lines = s.lines().peekable();
        let line_len = lines.peek().map_or(0, |l| l.len());
        Self {
            data: lines.flat_map(str::as_bytes).copied().collect::<Box<_>>(),
            offset: line_len,
        }
    }

    const fn next_pos(&self, p: usize, dir: u8) -> Option<usize> {
        // URDL as normal
        Some(match dir {
            0 if p >= self.offset => p - self.offset,
            1 if (p + 1) % self.offset != 0 => p + 1,
            2 if p < self.data.len() - self.offset => p + self.offset,
            3 if p % self.offset != 0 => p - 1,
            _ => { return None }
        })
    }
    fn intersections_and_distances(&self, slopes: bool) -> Vec<Vec<(usize, usize)>> {
        // Simplify DFs with intersection indices and distances
        let mut vpts = vec![1, self.data.len() - 2];
        for (i, &c) in self.data.iter().enumerate() {
            let mut f = 0;
            for d in 0..4u8 {
                match self.next_pos(i, d).and_then(|j| self.data.get(j)) {
                    Some(&aa) if aa != b'#' => { f += 1; }
                    _ => {}
                }
            }
            if f > 2 && c != b'#' {
                vpts.push(i);
            }
        }
        vpts.sort_unstable();
        let mut rmap = vec![vec![]; vpts.len()];
        let mut q = Vec::with_capacity(4);
        for (id, &ii) in vpts.iter().enumerate() {
            let iv = &mut rmap[id];
            let mut visited = vec![false; self.data.len()];
            visited[ii] = true;
            q.push((ii, 0));
            while let Some((i, d)) = q.pop() {
                let ai = ARROWS.iter().position(|&a| self.data[i] == a).unwrap_or(4) as u8;
                for di in 0..4u8 {
                    if slopes && ai < 4 && ai != di {
                        continue;
                    }
                    let Some(np) = self.next_pos(i, di) else { continue };
                    match self.data.get(np) {
                        Some(&aa) if aa != b'#' => {
                            if !visited[np] {
                                visited[np] = true;
                                vpts.binary_search(&np).map_or_else(
                                    |_| { q.push((np, d+1)); },
                                    |idx| { iv.push((idx, d+1)); }
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        rmap
    }
}

fn dfs(i: usize, d: usize, rmap: &[Vec<(usize, usize)>], visited: &mut [bool], end: usize, r: &mut usize) {
    visited[i] = true;
    if i == end {
        *r = (*r).max(d);
    }
    for &(j, md) in &rmap[i] {
        if !visited[j] {
            dfs(j, md + d, rmap, visited, end, r);
        }
    }
    visited[i] = false;
}

pub fn fn1(input: &str) -> i64 {
    let grid = Grid::from_str(input);
    let rmap = grid.intersections_and_distances(true);
    let (mut dvisited, mut r) = (vec![false; rmap.len()], 0);
    dfs(0, 0, &rmap, &mut dvisited, rmap.len() - 1, &mut r);
    r as i64
}

pub fn fn2(input: &str) -> i64 {
    let grid = Grid::from_str(input);
    let rmap = grid.intersections_and_distances(false);
    let (mut dvisited, mut r) = (vec![false; rmap.len()], 0);
    dfs(0, 0, &rmap, &mut dvisited, rmap.len() - 1, &mut r);
    r as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    #[test]
    fn test_fn1_example() {
        assert_eq!(fn1(include_str!("../../../input/2023/d23/example.txt")), load_spec(include_str!("../../../input/2023/d23/example-spec.1.txt")));
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(fn1(include_str!("../../../input/2023/d23/input.txt")), load_spec(include_str!("../../../input/2023/d23/input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(fn2(include_str!("../../../input/2023/d23/example.txt")), load_spec(include_str!("../../../input/2023/d23/example-spec.2.txt")));
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(fn2(include_str!("../../../input/2023/d23/input.txt")), load_spec(include_str!("../../../input/2023/d23/input-spec.2.txt")));
    }

}