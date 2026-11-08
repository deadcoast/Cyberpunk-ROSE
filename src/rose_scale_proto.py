"""
Cyberpunk: ROSE — scale prototype: a grown Mat carrying a logistics layer and thousands of autonomous slimos.

Builds on mat_sim_proto.MatSim (same rules, same palette). Adds:
  * Relays   — extra signal sources so one stratum can be covered by one strain
  * Sites    — Markets at grates, Pits, Pods, placed on Nodes
  * Packets  — scrap cargo spawned by hauls at Markets, routed along hyphae to the Pit with the most demand
  * Agents   — N slimos, each with a state machine (home -> market -> pit -> home), moving 1 tile per tick
               along flow fields (multi-source BFS distance maps), vectorised over all agents at once
Renders a Stack view (whole stratum, flows as light) and a Street view (crop at 8x, agents as glyphs).

Usage: python rose_scale_proto.py [agents] [seed]
"""
import os
import sys
import time
from collections import deque
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import src.mat_sim_proto as m

OUT = m.OUT
W, H = 192, 48

# ---------------------------------------------------------------- relays: extend MatSim.compute_signal
class MatSimR(m.MatSim):
    def __init__(self, *a, **k):
        super().__init__(*a, **k)
        self.relays = []

    def place_relay(self, x, y):
        self.sub[y, x] = m.OPEN
        self.H[y, x] = m.NODE
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            if 0 <= x + dx < self.w and 0 <= y + dy < self.h and m.GROWABLE[self.sub[y + dy, x + dx]] and self.H[y + dy, x + dx] == m.EMPTY:
                self.H[y + dy, x + dx] = m.TIP
        self.N[y - 1:y + 2, x - 1:x + 2] = np.maximum(self.N[y - 1:y + 2, x - 1:x + 2], 0.5)
        self.relays.append((x, y))

    def compute_signal(self):
        self.sig[:] = 0.0
        hyph = self.hyphal_mask()
        R = self.P["heart_range"]
        dist = np.full((self.h, self.w), -1.0, np.float32)
        dq = deque()
        for (x, y) in [self.heart] + self.relays:
            if x is not None and hyph[y, x]:
                dist[y, x] = 0.0
                dq.append((x, y))
        cost = np.where(self.H == m.NODE, 0.5, np.where(self.H == m.TIP, 2.0, 1.0)).astype(np.float32)
        while dq:
            x, y = dq.popleft()
            d0 = dist[y, x]
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < self.w and 0 <= ny < self.h and hyph[ny, nx]:
                    nd = d0 + cost[ny, nx]
                    if nd < R and (dist[ny, nx] < 0 or nd < dist[ny, nx]):
                        dist[ny, nx] = nd
                        dq.append((nx, ny))
        self.sig = np.where(dist >= 0, np.maximum(0.0, 1.0 - dist / R), 0.0).astype(np.float32)

    def step_hyphae(self):
        rel = super().step_hyphae()
        for (x, y) in self.relays:                    # a Relay is a built nerve; it does not starve
            if self.H[y, x] != m.NODE:
                self.H[y, x] = m.NODE
        return rel


# ---------------------------------------------------------------- flow fields
def flow_field(passable, sources):
    """Multi-source BFS. Returns (dist, next_idx): dist per cell (-1 unreachable), next cell index toward the nearest source."""
    h, w = passable.shape
    dist = np.full((h, w), -1, np.int32)
    nxt = np.arange(h * w, dtype=np.int32).reshape(h, w)
    dq = deque()
    for (x, y) in sources:
        if passable[y, x]:
            dist[y, x] = 0
            dq.append((x, y))
    while dq:
        x, y = dq.popleft()
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = x + dx, y + dy
            if 0 <= nx < w and 0 <= ny < h and passable[ny, nx] and dist[ny, nx] < 0:
                dist[ny, nx] = dist[y, x] + 1
                nxt[ny, nx] = y * w + x
                dq.append((nx, ny))
    return dist, nxt.ravel()


# ---------------------------------------------------------------- the scale sim
class City:
    HOME, TO_MARKET, TO_PIT, TO_HOME = 0, 1, 2, 3

    def __init__(self, sim, n_agents, rng):
        self.sim, self.rng = sim, rng
        hyph = sim.hyphal_mask()
        nodes = np.argwhere(sim.H == m.NODE)
        # sites: markets at the two grate columns on the top culvert; pits = heart + relays; pods = sample of nodes
        # markets sit under grates between the relays, so cargo has 30-40 tiles of cord to travel
        cands = [(60, 12), (130, 12), (96, 38)]
        self.markets = []
        hy, hx = np.nonzero(hyph)
        for (x, y0) in cands:                         # nearest hyphal cell to the grate, within 8 tiles
            d = np.abs(hx - x) + np.abs(hy - y0)
            i = int(np.argmin(d))
            if d[i] <= 8:
                self.markets.append((int(hx[i]), int(hy[i])))
        self.pits = [sim.heart] + list(sim.relays)
        pod_pick = nodes[rng.choice(len(nodes), size=min(60, len(nodes)), replace=False)]
        self.pods = [(int(x), int(y)) for y, x in pod_pick]
        self.passable = hyph | (m.COND[sim.sub] > 0)           # slimos walk open ground and ride cords
        self.cords_only = hyph                                  # cargo rides the Mat only
        self.rebuild_fields()
        # agents
        n = n_agents
        homes = rng.integers(0, len(self.pods), size=n)
        px = np.array([self.pods[i][0] for i in homes]); py = np.array([self.pods[i][1] for i in homes])
        self.pos = (py * W + px).astype(np.int32)
        self.home = self.pos.copy()
        self.state = np.full(n, self.HOME, np.int8)
        self.dwell = rng.integers(0, 40, size=n).astype(np.int16)
        self.speed_phase = rng.integers(0, 2, size=n).astype(np.int32)   # half the agents move on odd ticks: 0.5 tile/tick walking
        # cargo packets on cords
        self.pk_pos = np.zeros(0, np.int32)
        self.pk_tier = np.zeros(0, np.int8)
        self.pit_stock = {p: 0.0 for p in self.pits}
        self.delivered = 0
        self.hauls = 0
        self.tick = 0

    def rebuild_fields(self):
        self.d_market, self.n_market = flow_field(self.passable, self.markets)
        self.d_pit, self.n_pit = flow_field(self.cords_only, self.pits)
        self.d_pod, self.n_pod = flow_field(self.passable, self.pods)
        # home field: each agent walks toward the nearest pod, then jumps to its own home when adjacent (good enough at this LOD)
        self.market_idx = np.array([y * W + x for (x, y) in self.markets], np.int32)
        self.pit_idx = np.array([y * W + x for (x, y) in self.pits], np.int32)

    def step(self):
        self.tick += 1
        rng = self.rng
        pos, st = self.pos, self.state
        moving = (self.tick + self.speed_phase) % 2 == 0
        # --- movement by state
        go_m = moving & (st == self.TO_MARKET)
        go_p = moving & (st == self.TO_PIT)
        go_h = moving & (st == self.TO_HOME)
        pos[go_m] = self.n_market[pos[go_m]]
        pos[go_p] = self.n_pit[pos[go_p]]
        pos[go_h] = self.n_pod[pos[go_h]]
        # --- arrivals
        at_m = (st == self.TO_MARKET) & (self.d_market.ravel()[pos] == 0)
        st[at_m] = self.TO_PIT
        at_p = (st == self.TO_PIT) & (self.d_pit.ravel()[pos] == 0)
        if at_p.any():
            self.delivered += int(at_p.sum())
            for i in np.flatnonzero(at_p)[:200]:
                key = (int(pos[i] % W), int(pos[i] // W))
                if key in self.pit_stock:
                    self.pit_stock[key] += 0.3
        st[at_p] = self.TO_HOME
        at_h = (st == self.TO_HOME) & (self.d_pod.ravel()[pos] == 0)
        st[at_h] = self.HOME
        pos[at_h] = self.home[at_h]
        self.dwell[at_h] = rng.integers(20, 80, size=int(at_h.sum()))
        # --- resting at home, then go to work
        home = st == self.HOME
        self.dwell[home] -= 1
        wake = home & (self.dwell <= 0)
        st[wake] = self.TO_MARKET
        # --- hauls: every 15 ticks each market lands 12-30 scrap packets that ride the cords to the hungriest pit
        if self.tick % 15 == 0 and len(self.markets):
            for mi in self.market_idx:
                k = int(rng.integers(12, 31))
                self.pk_pos = np.concatenate([self.pk_pos, np.full(k, mi, np.int32)])
                self.pk_tier = np.concatenate([self.pk_tier, rng.choice([1, 1, 1, 1, 2, 2, 3], size=k).astype(np.int8)])
                self.hauls += 1
        # --- packets move 1 tile/tick along the cords toward the nearest pit; arrivals feed the pit
        if len(self.pk_pos):
            self.pk_pos = self.n_pit[self.pk_pos]
            arrived = self.d_pit.ravel()[self.pk_pos] == 0
            if arrived.any():
                for i in np.flatnonzero(arrived):
                    key = (int(self.pk_pos[i] % W), int(self.pk_pos[i] // W))
                    self.sim.feed(*key, (0.18, 0.30, 0.45)[self.pk_tier[i] - 1])
                self.pk_pos = self.pk_pos[~arrived]
                self.pk_tier = self.pk_tier[~arrived]
        # the Mat keeps living underneath, every tick
        self.sim.step()
        if self.tick % 20 == 0:
            self.passable = self.sim.hyphal_mask() | (m.COND[self.sim.sub] > 0)
            self.cords_only = self.sim.hyphal_mask()
            self.rebuild_fields()

    # ------------------------------------------------------------ render
    def flow_heat(self):
        heat = np.zeros(W * H, np.float32)
        np.add.at(heat, self.pk_pos, 1.0)
        return heat.reshape(H, W)

    def render_stack(self, scale=4):
        """Stack view: the stratum as a circuit board. Substrate dim, cords by signal, traffic as light."""
        sim = self.sim
        sub = sim.sub
        base = np.zeros((H, W, 3), np.uint8)
        base[:] = m.PAL["sump"]
        base[sub == m.ROCK] = m.PAL["culvert"]
        base[sub == m.BRICK] = (42, 31, 25)                   # Silt: brick lining, dim at this zoom
        base[sub == m.WATER] = (18, 34, 56)                   # Drain Blue
        hyph = sim.hyphal_mask()
        base[hyph & (sim.sig <= 0)] = m.PAL["matdark"]
        base[hyph & (sim.sig > 0)] = m.PAL["mat"]
        # traffic: slimos + cargo per cell -> brightness steps
        heat = np.zeros(W * H, np.float32)
        np.add.at(heat, self.pos, 1.0)
        np.add.at(heat, self.pk_pos, 2.0)
        heat = heat.reshape(H, W)
        base[(heat >= 1) & (heat < 3)] = m.PAL["matlit"]
        base[(heat >= 3) & (heat < 7)] = m.PAL["matglow"]
        base[heat >= 7] = m.PAL["chalk"]
        for (x, y) in self.pods:
            base[y, x] = (154, 112, 73)                        # Oxide: a Pod
        for (x, y) in self.pits:
            base[y, x] = m.PAL["acid"]
        for (x, y) in self.markets:
            base[y, x] = m.PAL["pink"]
        return Image.fromarray(base, "RGB").resize((W * scale, H * scale), Image.NEAREST)

    def render_street(self, x0, y0, w=48, h=14, scale=16):
        """Street view crop: agents as 2x3 glyphs with a Void outline, cargo as pulses."""
        sim = self.sim
        tile = sim.render(scale=1, pits=[]).convert("RGB").crop((x0, y0, x0 + w, y0 + h))
        im = tile.resize((w * scale, h * scale), Image.NEAREST)
        d = ImageDraw.Draw(im)
        heat = self.flow_heat()
        for yy in range(y0, y0 + h):
            for xx in range(x0, x0 + w):
                if heat[yy, xx] > 0:
                    cx, cy = (xx - x0) * scale + scale // 2, (yy - y0) * scale + scale // 2
                    d.rectangle([cx - 2, cy - 2, cx + 2, cy + 2], fill=(25, 230, 255))   # Cyan cargo pulse
        px = self.pos % W; py = self.pos // W
        sel = (px >= x0) & (px < x0 + w) & (py >= y0) & (py < y0 + h)
        # one glyph per agent; several agents on a tile fan out across it so a crowd reads as a crowd
        slot = {}
        for i in np.flatnonzero(sel):
            key = int(self.pos[i]); k = slot.get(key, 0); slot[key] = k + 1
            if k >= 4:
                continue
            gx = (px[i] - x0) * scale + 2 + (k % 2) * 7
            gy = (py[i] - y0) * scale + 1 + (k // 2) * 7
            carrying = self.state[i] == self.TO_PIT
            body = (217, 154, 166) if carrying else (154, 96, 112)
            d.rectangle([gx - 1, gy - 1, gx + 5, gy + 7], fill=m.PAL["void"])     # ink outline (the Ink rule)
            d.rectangle([gx, gy, gx + 4, gy + 6], fill=body)
            d.point((gx + 2, gy + 3), fill=m.PAL["matglow"])                      # vein glow
            if carrying:
                d.rectangle([gx + 4, gy - 1, gx + 5, gy], fill=m.PAL["acid"])      # the cargo on its back
        return im


def main(n_agents=3000, seed=11):
    rng = np.random.default_rng(seed)
    G = [0.5] * 10; G[0] = 0.8; G[8] = 0.8                     # a vigorous, conductive strain so one stratum fills
    sim = MatSimR(seed=seed, genome=G)
    sim.seed_heart(96, 25)
    relays = [(40, 25), (150, 25), (20, 12), (100, 12), (170, 12), (60, 38), (130, 38)]
    for r in relays:
        sim.place_relay(*r)
    t0 = time.perf_counter()
    for t in range(1, 501):
        plan = [(96, 25, 0.6)] + [(x, y, 0.4) for (x, y) in relays]
        sim.step(plan)
    grow_ms = (time.perf_counter() - t0) / 500 * 1000
    cells = int(sim.hyphal_mask().sum())
    city = City(sim, n_agents, rng)
    t0 = time.perf_counter()
    for t in range(300):
        city.step()
    city_ms = (time.perf_counter() - t0) / 300 * 1000
    stack = city.render_stack()
    street = city.render_street(76, 19)
    stats = dict(cells=cells, nodes=int((sim.H == m.NODE).sum()), agents=n_agents, packets_in_flight=int(len(city.pk_pos)),
                 deliveries=city.delivered, hauls=city.hauls, grow_ms=round(grow_ms, 1), city_ms=round(city_ms, 1))
    # sheet
    try:
        f = ImageFont.load_default(size=13)
    except TypeError:
        f = ImageFont.load_default()
    pad = 12
    Wd = max(stack.width, street.width, 900) + 2 * pad
    Hd = pad + stack.height + 42 + pad + street.height + 26 + pad
    out = Image.new("RGB", (Wd, Hd), m.PAL["sump"])
    d = ImageDraw.Draw(out)
    out.paste(stack, (pad, pad))
    d.text((pad, pad + stack.height + 5),
           f"STACK VIEW   one stratum 192x48 at 4x   Mat {stats['cells']} cells, {stats['nodes']} nodes   {stats['agents']} slimos   "
           f"{stats['deliveries']} deliveries in 300 ticks   traffic drawn as light: Mat Lit 1-2 per tile, Mat Glow 3-6, Chalk 7+",
           fill=m.PAL["chalk"], font=f)
    d.text((pad, pad + stack.height + 21),
           f"pink = Seep Market under a grate   yellow = Pit / Relay   oxide = Hab-Pod   {stats['city_ms']} ms per tick for agents + cargo + CA (numpy, single thread)",
           fill=(142, 144, 151), font=f)
    y2 = pad + stack.height + 42 + pad
    out.paste(street, (pad, y2))
    d.text((pad, y2 + street.height + 5),
           "STREET VIEW   48x14 tiles at 16x   slimo glyphs with the 1 px ink outline (bright = carrying, yellow pack = cargo)   cyan = cargo pulse riding a cord",
           fill=m.PAL["chalk"], font=f)
    out.save(os.path.join(OUT, "rose_scale.png"))
    stack.save(os.path.join(OUT, "rose_scale_stack.png"))
    street.save(os.path.join(OUT, "rose_scale_street.png"))
    print(stats)


if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 3000
    s = int(sys.argv[2]) if len(sys.argv) > 2 else 11
    main(n, s)
