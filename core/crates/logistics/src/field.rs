//! Signal-weighted distance fields over the Mat (Full Design v3.11, Cargo: Routing). A tile costs 1 + 2 × (1 − σ),
//! kept in tenths so the field is integer: a packet takes the bright trunk over the dim shortcut.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const UNREACHED: u32 = u32::MAX;

/// Cost of stepping onto a tile, in tenths of a tile: 10 at full signal, 30 at none.
#[inline]
pub fn tile_cost(sig: f32) -> u32 {
    10 + (20.0 * (1.0 - sig.clamp(0.0, 1.0))).round() as u32
}

#[derive(Clone, Debug)]
pub struct Field {
    /// weighted distance to the target in tenths of a tile; UNREACHED off the Mat or cut off
    pub dist: Vec<u32>,
    /// the neighbour one step toward the target (itself at the target or when unreached)
    pub next: Vec<u32>,
}

impl Field {
    /// Dijkstra outward from `target` over `mask`, paying the cost of the tile stepped onto (toward the target).
    pub fn build(w: usize, h: usize, mask: &[bool], sig: &[f32], target: usize) -> Field {
        let n = w * h;
        let mut dist = vec![UNREACHED; n];
        let mut next: Vec<u32> = (0..n as u32).collect();
        if target >= n || !mask[target] {
            return Field { dist, next };
        }
        let mut heap = BinaryHeap::new();
        dist[target] = 0;
        heap.push(Reverse((0u32, target as u32)));
        while let Some(Reverse((d, i))) = heap.pop() {
            let i = i as usize;
            if d > dist[i] {
                continue;
            }
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            // a mover at j steps onto i, so j's distance adds the cost of i
            let step = tile_cost(sig[i]);
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let j = ny as usize * w + nx as usize;
                if !mask[j] {
                    continue;
                }
                let nd = d + step;
                if nd < dist[j] {
                    dist[j] = nd;
                    next[j] = i as u32;
                    heap.push(Reverse((nd, j as u32)));
                }
            }
        }
        Field { dist, next }
    }

    #[inline]
    pub fn reachable(&self, i: usize) -> bool {
        self.dist[i] != UNREACHED
    }

    /// Weighted tiles from `i` to the target.
    #[inline]
    pub fn tiles(&self, i: usize) -> f32 {
        self.dist[i] as f32 / 10.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bright_trunk_beats_the_dim_shortcut() {
        // 7 x 3: a straight dim row 1 from (0,1) to (6,1), and a bright detour over rows 0 and 2 is not offered;
        // instead compare two corridors: row 0 bright and long (via the ends), row 1 dim and short.
        let (w, h) = (7, 3);
        let mut mask = vec![false; w * h];
        let mut sig = vec![0.0f32; w * h];
        for x in 0..w {
            mask[w + x] = true; // row 1, dim
            mask[x] = true; // row 0, bright
            sig[x] = 1.0;
        }
        let f = Field::build(w, h, &mask, &sig, w + 6);
        // from (0,1): straight along the dim row costs 6 × 30 = 180; up, along the bright row, down costs
        // 10 + 6 × 10 + 30 = 100, so the first step is up
        assert_eq!(f.next[w] as usize, 0, "takes the bright row");
        assert_eq!(f.dist[w], 100);
        assert!(!f.reachable(2 * w), "row 2 is off the Mat");
    }
}
