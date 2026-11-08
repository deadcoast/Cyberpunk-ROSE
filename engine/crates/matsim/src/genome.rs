//! Ten genes in [0, 1] and their linear expression into CA parameters.
//! Same mapping as the GDD table (Mat spec, Genome) and the Python prototype.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Genome(pub [f32; 10]);

pub const VIGOR: usize = 0;
pub const BRANCHING: usize = 1;
pub const THRIFT: usize = 2;
pub const APPETITE: usize = 3;
pub const LUME: usize = 4;
pub const TOXICITY: usize = 5;
pub const CHROME: usize = 6;
pub const FERAL: usize = 7;
pub const CONDUCTANCE: usize = 8;
pub const PLASTICITY: usize = 9;

pub const GENE_NAMES: [&str; 10] = [
    "Vigor", "Branching", "Thrift", "Appetite", "Lume", "Toxicity", "Chrome Affinity", "Feral Drive", "Conductance", "Plasticity",
];

impl Default for Genome {
    fn default() -> Self {
        Genome([0.5; 10])
    }
}

impl Genome {
    pub fn with(mut self, gene: usize, value: f32) -> Self {
        self.0[gene] = value.clamp(0.0, 1.0);
        self
    }

    pub fn params(&self) -> Params {
        let g = &self.0;
        Params {
            theta_grow: 0.35 - 0.25 * g[VIGOR],
            branch_p: 0.08 + 0.42 * g[BRANCHING],
            theta_starve: 0.05 + 0.10 * (1.0 - g[THRIFT]),
            maint: 0.0006 * (1.4 - 0.8 * g[THRIFT]),
            appetite: 0.02 + 0.30 * g[APPETITE],
            lume_r: 1.0 + 6.0 * g[LUME],
            heart_range: 24.0 + 32.0 * g[CONDUCTANCE],
            cord_k: 0.6 + 0.4 * g[CONDUCTANCE],
            pump: 0.30 + 0.40 * g[CONDUCTANCE],
            feral_rate: 0.01 + 0.09 * g[FERAL],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Params {
    /// nutrient a hypha needs around it before it will extend
    pub theta_grow: f32,
    /// fork chance on extension; lateral sprout rate scales on it
    pub branch_p: f32,
    /// below this a hyphal cell starves
    pub theta_starve: f32,
    /// per-tick upkeep drawn by a hyphal cell
    pub maint: f32,
    pub appetite: f32,
    pub lume_r: f32,
    /// signal range in cord steps
    pub heart_range: f32,
    /// cord-to-cord conductance (<= 1 keeps explicit diffusion stable)
    pub cord_k: f32,
    /// share of surplus a signalled hyphal cell pushes outward per tick, split across outward links
    pub pump: f32,
    pub feral_rate: f32,
}
