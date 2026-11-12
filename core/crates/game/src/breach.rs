//! The stack and the breach (Full Design v3.19, Progression): a built Relay within BREACH_REACH of a sealed crossing
//! opens it at a shift's end, or a stratum's grate key spent on it from its card; the city's stratum is the highest the
//! Mat joined to the Heart holds a tile of. The bridge and the twenty-shift city both call `shift_end`.

use colony::{Colony, StructureFile};
use inventory::Stash;
use items::ItemTable;
use logistics::{City, StrataFile};
use threat::Threats;

use crate::log::{CityLog, Class};

/// A Relay this near a slab breaks it at the shift's end (Chebyshev tiles; this design's).
pub const BREACH_REACH: i32 = 4;

/// A crossing between strata as the cards show it.
#[derive(Clone, Debug, PartialEq)]
pub struct Crossing {
    /// its left tile (the slab's, then the open floor's)
    pub tile: usize,
    /// the stratum below it, whose ceiling it is in
    pub below: u8,
    pub sealed: bool,
    /// the grate key that opens it ("" none)
    pub key: String,
}

/// Every crossing of the stack.
pub fn crossings(sim: &mat::MatSim) -> Vec<Crossing> {
    let f = StrataFile::shipped();
    (1..sim.strata).flat_map(|s| sim.crossings_of(s).into_iter().map(move |c| (s, c))).map(|(s, c)| Crossing { tile: c, below: s, sealed: sim.sealed(c), key: f.get(s).map(|d| d.key.clone()).unwrap_or_default() }).collect()
}

/// The highest stratum the Mat joined to the Heart holds a tile of (1 with no Heart).
pub fn top_stratum(sim: &mat::MatSim) -> u8 {
    let joined = logistics::joined_to_heart(sim);
    joined.iter().enumerate().filter(|(_, &j)| j).map(|(i, _)| sim.stratum_at(i)).max().unwrap_or(1)
}

/// What a breach changes beyond the substrate: the cargo's fields and the threats' zones see the open ground.
fn opened(city: &mut City, threats: Option<&mut Threats>) {
    city.rebuild_fields();
    if let Some(t) = threats {
        t.rezone(&city.sim);
    }
}

/// The shift's end: Relays break the slabs near them, and the city's stratum follows the Mat. Returns the stratum newly
/// reached, if one was.
pub fn shift_end(city: &mut City, colony: &Colony, defs: &StructureFile, mut threats: Option<&mut Threats>, log: &mut CityLog) -> Option<u8> {
    let w = city.sim.w;
    let relays: Vec<(i32, i32)> = colony.structures.iter().filter(|s| s.built && defs.structure[s.def].id == "relay").map(|s| (s.x as i32, s.y as i32)).collect();
    let mut any = false;
    for c in crossings(&city.sim).into_iter().filter(|c| c.sealed) {
        let (cx, cy) = ((c.tile % w) as i32, (c.tile / w) as i32);
        let near = relays.iter().find(|&&(x, y)| (x - cx).abs().min((x - cx - 1).abs()).max((y - cy).abs()) <= BREACH_REACH);
        if let Some(&(rx, ry)) = near {
            if city.sim.open_slab(c.tile) {
                any = true;
                let name = StrataFile::shipped().get(c.below + 1).map(|d| d.name.clone()).unwrap_or_default();
                log.push(city.tick, Class::Landing, format!("the Relay at {rx},{ry} broke the slab at {cx},{cy}: the way into {name} is open"));
            }
        }
    }
    if any {
        opened(city, threats.as_deref_mut());
    }
    reach(city, threats, log)
}

/// The city's stratum after the Mat's growth: a stratum newly reached is the alert, Ivan's line and a chapter's
/// condition (Full Design v3.19).
pub fn reach(city: &City, threats: Option<&mut Threats>, log: &mut CityLog) -> Option<u8> {
    let top = top_stratum(&city.sim);
    let t = threats?;
    if top <= t.stratum {
        return None;
    }
    t.stratum = top;
    let f = StrataFile::shipped();
    let name = f.get(top).map(|d| d.name.clone()).unwrap_or_default();
    log.push(city.tick, Class::Landing, format!("the Mat reached {name} (S{top})"));
    Some(top)
}

/// Spend the stratum's grate key on a crossing (its card's verb): the key leaves the Stash and the slab opens now.
pub fn open_with_key(city: &mut City, threats: Option<&mut Threats>, stash: &mut Stash, table: &ItemTable, crossing: usize, log: &mut CityLog) -> Result<String, String> {
    let c = crossings(&city.sim).into_iter().find(|c| c.tile == crossing).ok_or("no crossing there")?;
    if !c.sealed {
        return Err("already open".into());
    }
    if c.key.is_empty() {
        return Err("no key opens it".into());
    }
    let idx = table.id_of(&c.key).ok_or("the key is not in the item table")?;
    let h = stash.handles().into_iter().find(|(_, _, it)| it.def == idx).map(|(h, _, _)| h).ok_or_else(|| format!("needs the {}", table.def(idx).name))?;
    stash.take(h);
    city.sim.open_slab(crossing);
    opened(city, threats);
    let name = StrataFile::shipped().get(c.below + 1).map(|d| d.name.clone()).unwrap_or_default();
    let line = format!("the {} opened the slab at {},{}: the way into {name} is open", table.def(idx).name, crossing % city.sim.w, crossing / city.sim.w);
    log.push(city.tick, Class::Decision, line.clone());
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use items::{ItemInstance, ItemTable, ITEMS_TOML};
    use mat::sim::{hy, sub};

    /// Lay Cord from the founding's market cell to its grate and up the shaft `up` tiles past the crossing.
    fn lay_up_the_shaft(w: &mut crate::World, g: usize, cross: usize, up: usize) {
        let sw = w.city.sim.w;
        let way = logistics::ground_distance(&w.city.sim, g);
        let mut at = w.city.market_cells[0].expect("the founding reaches its grate");
        while way[at] > 0 {
            if !w.city.sim.is_hyphal_at(at) {
                w.city.sim.hy[at] = hy::CORD;
            }
            let (x, y) = (at % sw, at / sw);
            at = [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)].into_iter().filter(|&(nx, ny)| nx < sw && ny < w.city.sim.h).map(|(nx, ny)| ny * sw + nx).filter(|&j| way[j] < way[at]).min_by_key(|&j| (way[j], j)).unwrap();
        }
        let (cx, cy) = (cross % sw, cross / sw);
        for y in cy - up..=g / sw + 3 {
            let i = y * sw + cx;
            if sub::GROWABLE[w.city.sim.sub[i] as usize] && !w.city.sim.is_hyphal_at(i) {
                w.city.sim.hy[i] = hy::CORD;
            }
        }
        w.city.sim.compute_signal();
    }

    #[test]
    fn a_relay_by_the_slab_or_the_key_opens_it_and_the_mat_through_it_reaches_the_culverts() {
        let mut w = crate::World::found(11, ItemTable::from_toml(ITEMS_TOML).unwrap());
        w.colony.progress.tech = progress::Tech::every_structure();
        assert_eq!(top_stratum(&w.city.sim), 1);
        assert_eq!(crossings(&w.city.sim).len(), 40, "eight shafts out of each of five ceilings");
        assert!(crossings(&w.city.sim).iter().all(|c| c.sealed));
        let g = crate::city::first_grate(&w).unwrap();
        let sw = w.city.sim.w;
        let cross = w.city.sim.crossings_of(1).into_iter().find(|&c| c % sw == g % sw).unwrap();
        // the Mat to the grate and up to the slab; a Relay on it beside the shaft breaks the slab at the shift's end
        lay_up_the_shaft(&mut w, g, cross, 0);
        let (gx, gy) = (g % sw, g / sw);
        // the Relays the player chains toward a grate carry the signal there (Full Design: the Relay)
        w.city.sim.place_relay(gx + 1, gy + 3);
        w.city.sim.compute_signal();
        let tries: Vec<Result<u32, colony::PlaceError>> = (gy + 1..gy + 4).map(|y| w.colony.place_built(&w.defs, &w.city.sim, "relay", gx as u16, y as u16, 1)).collect();
        assert!(tries.iter().any(|r| r.is_ok()), "a Relay stands in the shaft under the grate: {tries:?}");
        assert_eq!(shift_end(&mut w.city, &w.colony, &w.defs, Some(&mut w.threats), &mut w.log), None, "open, not yet reached");
        assert!(!w.city.sim.sealed(cross));
        assert!(w.log.events.iter().any(|e| e.what.contains("broke the slab") && e.what.contains("the Culverts")));
        // the Mat up the open shaft: the city stands in the Culverts
        lay_up_the_shaft(&mut w, g, cross, 3);
        assert_eq!(shift_end(&mut w.city, &w.colony, &w.defs, Some(&mut w.threats), &mut w.log), Some(2));
        assert_eq!(w.threats.stratum, 2);
        assert!(w.log.events.iter().any(|e| e.what == "the Mat reached the Culverts (S2)"));
        // another of the Sump's crossings by its key; without the key, refused
        let other = w.city.sim.crossings_of(1).into_iter().find(|&c| w.city.sim.sealed(c)).unwrap();
        assert!(open_with_key(&mut w.city, Some(&mut w.threats), &mut w.stash, &w.table, other, &mut w.log).unwrap_err().contains("Sump Grate Key"));
        let key = w.table.id_of("key_grate_sump").unwrap();
        assert!(w.stash.place_or_floor(&w.table, ItemInstance::new(key)));
        open_with_key(&mut w.city, Some(&mut w.threats), &mut w.stash, &w.table, other, &mut w.log).unwrap();
        assert!(!w.city.sim.sealed(other));
        assert!(w.stash.handles().iter().all(|(_, _, it)| it.def != key), "the key is spent");
    }
}
