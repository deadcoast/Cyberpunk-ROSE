//! Fog of war: three states per tile — unseen, remembered, visible — and the per-route memory that persists
//! across runs as a bitset (what the route card shows dim; a death still pays in map).

pub const UNSEEN: u8 = 0;
pub const REMEMBERED: u8 = 1;
pub const VISIBLE: u8 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fog {
    pub w: usize,
    pub h: usize,
    pub state: Vec<u8>,
    visible_now: Vec<usize>,
}

impl Fog {
    pub fn new(w: usize, h: usize) -> Fog {
        Fog { w, h, state: vec![UNSEEN; w * h], visible_now: Vec::new() }
    }

    /// Start from a previous run's memory of this route (bitset from `memory_bits`); a size mismatch is ignored.
    pub fn with_memory(w: usize, h: usize, bits: &[u8]) -> Fog {
        let mut f = Fog::new(w, h);
        if bits.len() == (w * h).div_ceil(8) {
            for i in 0..w * h {
                if bits[i / 8] & (1 << (i % 8)) != 0 {
                    f.state[i] = REMEMBERED;
                }
            }
        }
        f
    }

    /// Apply this tick's visibility mask: visible tiles become VISIBLE, tiles that were visible and are not now
    /// become REMEMBERED.
    pub fn update(&mut self, visible: &[bool]) {
        for &i in &self.visible_now {
            self.state[i] = REMEMBERED;
        }
        self.visible_now.clear();
        for (i, &v) in visible.iter().enumerate() {
            if v {
                self.state[i] = VISIBLE;
                self.visible_now.push(i);
            }
        }
    }

    #[inline]
    pub fn at(&self, x: usize, y: usize) -> u8 {
        self.state[y * self.w + x]
    }

    /// Everything seen this run or remembered from before, one bit per tile, for the save.
    pub fn memory_bits(&self) -> Vec<u8> {
        let mut bits = vec![0u8; (self.w * self.h).div_ceil(8)];
        for (i, &s) in self.state.iter().enumerate() {
            if s != UNSEEN {
                bits[i / 8] |= 1 << (i % 8);
            }
        }
        bits
    }

    pub fn seen_count(&self) -> usize {
        self.state.iter().filter(|&&s| s != UNSEEN).count()
    }

    pub fn visible_count(&self) -> usize {
        self.visible_now.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_progress_and_memory_round_trips() {
        let mut f = Fog::new(10, 4);
        let mut vis = vec![false; 40];
        vis[5] = true;
        vis[6] = true;
        f.update(&vis);
        assert_eq!(f.at(5, 0), VISIBLE);
        assert_eq!(f.visible_count(), 2);
        vis[5] = false;
        f.update(&vis);
        assert_eq!(f.at(5, 0), REMEMBERED);
        assert_eq!(f.at(6, 0), VISIBLE);
        assert_eq!(f.at(7, 0), UNSEEN);
        let bits = f.memory_bits();
        assert_eq!(bits.len(), 5);
        let g = Fog::with_memory(10, 4, &bits);
        assert_eq!(g.at(5, 0), REMEMBERED);
        assert_eq!(g.at(6, 0), REMEMBERED);
        assert_eq!(g.seen_count(), 2);
        // a mismatched memory is ignored rather than misapplied
        let h = Fog::with_memory(11, 4, &bits);
        assert_eq!(h.seen_count(), 0);
    }
}
