//! Flow fields: multi-source breadth-first distance maps over a passable mask.
//! Thousands of movers each step to `next[pos]`; nobody pathfinds on their own.

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct FlowField {
    pub w: usize,
    pub h: usize,
    /// -1 = unreachable
    pub dist: Vec<i32>,
    /// next cell index toward the nearest source (self when at a source or unreachable)
    pub next: Vec<u32>,
}

impl FlowField {
    pub fn build(w: usize, h: usize, passable: &[bool], sources: &[usize]) -> Self {
        let n = w * h;
        let mut dist = vec![-1i32; n];
        let mut next: Vec<u32> = (0..n as u32).collect();
        let mut dq = VecDeque::new();
        for &s in sources {
            if s < n && passable[s] && dist[s] < 0 {
                dist[s] = 0;
                dq.push_back(s);
            }
        }
        while let Some(i) = dq.pop_front() {
            let x = (i % w) as i32;
            let y = (i / w) as i32;
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let j = ny as usize * w + nx as usize;
                if passable[j] && dist[j] < 0 {
                    dist[j] = dist[i] + 1;
                    next[j] = i as u32;
                    dq.push_back(j);
                }
            }
        }
        FlowField { w, h, dist, next }
    }

    #[inline]
    pub fn at_source(&self, i: usize) -> bool {
        self.dist[i] == 0
    }

    #[inline]
    pub fn reachable(&self, i: usize) -> bool {
        self.dist[i] >= 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distances_and_next_on_a_corridor() {
        // 5x3, middle row passable, source at (0,1)
        let w = 5;
        let h = 3;
        let passable: Vec<bool> = (0..w * h).map(|i| i / w == 1).collect();
        let f = FlowField::build(w, h, &passable, &[w]);
        assert_eq!(f.dist[w], 0);
        assert_eq!(f.dist[w + 4], 4);
        assert_eq!(f.next[w + 4] as usize, w + 3);
        assert_eq!(f.dist[0], -1);
        assert!(!f.reachable(0));
    }
}
