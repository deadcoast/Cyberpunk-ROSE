//! Symmetric shadowcasting (Albert Ford's formulation) on the route grid: if A sees B then B sees A, so what can
//! notice the Runner is exactly what a Runner looking that way can see. Slopes are exact rationals; no floats.

/// Visit every tile visible from `origin` within `radius` (Chebyshev), calling `mark(x, y)`.
/// `opaque(x, y)` is true for tiles that block sight; out-of-bounds tiles are treated as opaque.
pub fn compute<F, M>(origin: (i32, i32), radius: i32, w: i32, h: i32, opaque: F, mut mark: M)
where
    F: Fn(i32, i32) -> bool,
    M: FnMut(i32, i32),
{
    mark(origin.0, origin.1);
    let blocked = |x: i32, y: i32| -> bool { x < 0 || y < 0 || x >= w || y >= h || opaque(x, y) };
    for q in 0..4 {
        let mut rows: Vec<Row> = vec![Row { depth: 1, start: Slope(-1, 1), end: Slope(1, 1) }];
        while let Some(row) = rows.pop() {
            if row.depth > radius {
                continue;
            }
            let mut prev: Option<bool> = None; // was the previous tile a wall
            let mut cur = row;
            let (min_col, max_col) = cur.cols();
            for col in min_col..=max_col {
                let (x, y) = transform(q, origin, cur.depth, col);
                let wall = blocked(x, y);
                let tile_slope = Slope(2 * col - 1, 2 * cur.depth);
                if (wall || cur.is_symmetric(col)) && x >= 0 && y >= 0 && x < w && y < h {
                    mark(x, y);
                }
                if prev == Some(true) && !wall {
                    cur.start = tile_slope;
                }
                if prev == Some(false) && wall {
                    let mut next = cur.next();
                    next.end = tile_slope;
                    rows.push(next);
                }
                prev = Some(wall);
            }
            if prev == Some(false) {
                rows.push(cur.next());
            }
        }
    }
}

/// A rational slope num/den with den > 0.
#[derive(Clone, Copy, Debug)]
struct Slope(i32, i32);

#[derive(Clone, Copy, Debug)]
struct Row {
    depth: i32,
    start: Slope,
    end: Slope,
}

impl Row {
    fn next(&self) -> Row {
        Row { depth: self.depth + 1, start: self.start, end: self.end }
    }

    /// Column range of this row: min = round_ties_up(depth * start), max = round_ties_down(depth * end).
    fn cols(&self) -> (i32, i32) {
        (round_ties_up(self.depth * self.start.0, self.start.1), round_ties_down(self.depth * self.end.0, self.end.1))
    }

    /// A floor tile is revealed only when its centre lies between the row's start and end slopes.
    fn is_symmetric(&self, col: i32) -> bool {
        // col >= depth * start  &&  col <= depth * end   (compare as rationals)
        col * self.start.1 >= self.depth * self.start.0 && col * self.end.1 <= self.depth * self.end.0
    }
}

/// floor(n/d + 1/2) for d > 0.
fn round_ties_up(n: i32, d: i32) -> i32 {
    (2 * n + d).div_euclid(2 * d)
}

/// ceil(n/d - 1/2) for d > 0.
fn round_ties_down(n: i32, d: i32) -> i32 {
    -(((-2 * n) + d).div_euclid(2 * d))
}

fn transform(quadrant: usize, origin: (i32, i32), depth: i32, col: i32) -> (i32, i32) {
    match quadrant {
        0 => (origin.0 + col, origin.1 - depth), // north
        1 => (origin.0 + depth, origin.1 + col), // east
        2 => (origin.0 + col, origin.1 + depth), // south
        _ => (origin.0 - depth, origin.1 + col), // west
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn fov(grid: &[&str], origin: (i32, i32), radius: i32) -> HashSet<(i32, i32)> {
        let h = grid.len() as i32;
        let w = grid[0].len() as i32;
        let mut seen = HashSet::new();
        compute(origin, radius, w, h, |x, y| grid[y as usize].as_bytes()[x as usize] == b'#', |x, y| {
            seen.insert((x, y));
        });
        seen
    }

    #[test]
    fn open_room_is_fully_visible() {
        let g = ["#######", "#.....#", "#.....#", "#.....#", "#######"];
        let s = fov(&g, (3, 2), 10);
        for y in 0..5 {
            for x in 0..7 {
                assert!(s.contains(&(x, y)), "{x},{y}");
            }
        }
    }

    #[test]
    fn walls_hide_what_is_behind_them() {
        let g = ["#########", "#...#...#", "#...#...#", "#...#...#", "#########"];
        let s = fov(&g, (2, 2), 10);
        assert!(s.contains(&(4, 2)));
        assert!(!s.contains(&(6, 2)));
        assert!(!s.contains(&(7, 1)));
    }

    #[test]
    fn symmetric_between_floor_tiles() {
        let g = [
            "##############",
            "#......#.....#",
            "#......#.....#",
            "#..#.........#",
            "#......###...#",
            "#............#",
            "##############",
        ];
        let floors: Vec<(i32, i32)> = (0..7).flat_map(|y| (0..14).map(move |x| (x, y))).filter(|&(x, y)| g[y as usize].as_bytes()[x as usize] == b'.').collect();
        for &a in &floors {
            let sa = fov(&g, a, 20);
            for &b in &floors {
                let sb = fov(&g, b, 20);
                assert_eq!(sa.contains(&b), sb.contains(&a), "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn radius_caps_the_sweep() {
        let g = ["....................."];
        let s = fov(&g, (10, 0), 3);
        assert!(s.contains(&(13, 0)) && !s.contains(&(14, 0)));
    }
}
