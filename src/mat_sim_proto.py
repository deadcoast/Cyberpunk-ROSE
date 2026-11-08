"""
Cyberpunk: ROSE — Mat cellular automaton, headless prototype (M0 evidence).

One stratum, 192x48 tiles. Nutrient field (explicit diffusion + decay + uptake + Heart pump),
hyphal states (Empty/Tip/Cord/Node/Necrotic), signal BFS from the Heart, genome -> parameters.
Renders snapshots to PNG with the SEEPAGE palette. Seeded, deterministic.

Usage: python mat_sim_proto.py [seed] [ticks]
"""
import sys
from collections import deque
import numpy as np
from PIL import Image, ImageDraw, ImageFont

import os
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "out")
os.makedirs(OUT, exist_ok=True)

# ---------------------------------------------------------------- palette (SEEPAGE)
PAL = {
    "void": (7, 7, 12), "sump": (14, 16, 24), "culvert": (23, 26, 38), "concrete": (35, 39, 55),
    "rustgray": (74, 79, 94), "rebar": (107, 111, 123), "rust": (110, 78, 52), "oxide": (154, 112, 73),
    "silt": (42, 31, 25), "bruise": (91, 58, 74), "cistern": (31, 61, 99),
    "matdark": (15, 46, 28), "mat": (31, 110, 58), "matlit": (63, 214, 110), "matglow": (182, 255, 200),
    "acid": (246, 255, 46), "chalk": (181, 180, 176), "dust": (142, 144, 151), "pink": (255, 46, 151),
}

# ---------------------------------------------------------------- substrate
ROCK, OPEN, PIPE, BRICK, DEPOSIT, WATER = 0, 1, 2, 3, 4, 5
COND = np.array([0.0, 0.40, 0.60, 0.08, 0.40, 0.25], dtype=np.float32)   # nutrient conductance per substrate
GROWABLE = np.array([0, 1, 1, 1, 1, 1], dtype=bool)                       # hyphae may enter

# ---------------------------------------------------------------- hyphal states
EMPTY, TIP, CORD, NODE, NECRO = 0, 1, 2, 3, 4

# direction slices: (source slice, destination slice) for a value moving from a cell to its neighbour
DIRS = (
    ((slice(1, None), slice(None)), (slice(None, -1), slice(None))),   # to y-1 (up)
    ((slice(None, -1), slice(None)), (slice(1, None), slice(None))),   # to y+1 (down)
    ((slice(None), slice(1, None)), (slice(None), slice(None, -1))),   # to x-1 (left)
    ((slice(None), slice(None, -1)), (slice(None), slice(1, None))),   # to x+1 (right)
)


def genome_params(G):
    """10 genes in [0,1] -> CA parameters. Same mapping as the GDD table."""
    vigor, branch, thrift, appetite, lume, tox, chrome, feral, conduct, plastic = G
    return dict(
        theta_grow=0.35 - 0.25 * vigor,               # nutrient a tip needs ahead of it to advance
        branch_p=0.08 + 0.42 * branch,                # fork chance on extension; lateral sprout rate scales on it
        theta_starve=0.05 + 0.10 * (1.0 - thrift),    # below this a hyphal cell starves
        maint=0.0006 * (1.4 - 0.8 * thrift),          # per-tick upkeep drawn by a hyphal cell
        appetite=0.02 + 0.30 * appetite,
        lume_r=1 + 6 * lume,
        heart_range=24 + 32 * conduct,                # signal range in cord steps
        cord_k=0.6 + 0.4 * conduct,                   # cord->cord conductance (<=1 keeps diffusion stable)
        pump=0.30 + 0.40 * conduct,                   # share of N a signalled hyphal cell pushes outward per tick (split across outward links)
        feral_rate=0.01 + 0.09 * feral,
    )


class MatSim:
    def __init__(self, w=192, h=48, seed=7, genome=None):
        self.w, self.h = w, h
        self.rng = np.random.default_rng(seed)
        self.G = np.full(10, 0.5) if genome is None else np.asarray(genome, dtype=float)
        self.P = genome_params(self.G)
        self.sub = self._make_substrate()
        self.N = np.zeros((h, w), np.float32)
        self.H = np.zeros((h, w), np.uint8)
        self.age = np.zeros((h, w), np.int32)        # ticks in current state
        self.stall = np.zeros((h, w), np.int32)      # tip ticks without extension / cord ticks node-eligible
        self.starve = np.zeros((h, w), np.int32)     # ticks under theta_starve
        self.lonely = np.zeros((h, w), np.int32)     # ticks with no hyphal neighbour
        self.sig = np.zeros((h, w), np.float32)
        self.heart = None
        self.pits = {}                               # (x,y) -> scrap stock waiting in the Pit
        self.tick = 0
        self.stats = []
        self.grow_cost = 0.04
        self.tip_starter = 0.12
        self.uptake = 0.25
        self.pit_rate = 0.30                         # max nutrient a footprint cell draws from the Pit per tick                           # share of a neighbouring empty cell's N a hyphal cell absorbs per tick
        self.D, self.decay = 0.11, 0.001
        from collections import Counter
        self.ev = Counter()

    # ------------------------------------------------------------ world gen
    def _make_substrate(self):
        h, w = self.h, self.w
        sub = np.zeros((h, w), np.uint8)
        for y in (12, 25, 38):                       # three culvert rows, a pipe run above each
            sub[y - 1:y + 2, :] = OPEN
            sub[y - 2, :] = PIPE
        for x in range(10, w, 23):                   # vertical shafts
            sub[:, x:x + 2] = OPEN
        n = self.rng.random((h, w)).astype(np.float32)
        for _ in range(3):
            n = (n + np.roll(n, 1, 0) + np.roll(n, -1, 0) + np.roll(n, 1, 1) + np.roll(n, -1, 1)) / 5.0
        sub[n > 0.56] = OPEN                         # collapsed caves
        openm = sub == OPEN
        ring = (np.roll(openm, 1, 0) | np.roll(openm, -1, 0) | np.roll(openm, 1, 1) | np.roll(openm, -1, 1)) & ~openm & (sub == ROCK)
        sub[ring] = BRICK                            # brick lining
        dep = (self.rng.random((h, w)) < 0.025) & (sub == OPEN)
        sub[dep] = DEPOSIT                           # scrap deposits
        sub[39, :] = np.where(sub[39, :] == OPEN, WATER, sub[39, :])
        return sub

    # ------------------------------------------------------------ helpers
    @staticmethod
    def _nbrs(a):
        up = np.zeros_like(a); up[1:, :] = a[:-1, :]
        dn = np.zeros_like(a); dn[:-1, :] = a[1:, :]
        lf = np.zeros_like(a); lf[:, 1:] = a[:, :-1]
        rt = np.zeros_like(a); rt[:, :-1] = a[:, 1:]
        return up, dn, lf, rt

    def hyphal_mask(self):
        return (self.H == TIP) | (self.H == CORD) | (self.H == NODE)

    def hyphal_neighbour_count(self):
        return sum(self._nbrs(self.hyphal_mask().astype(np.int8)))

    def _transfer(self, N, give_masks, amounts):
        """Move amounts[d] from each cell to its neighbour in direction d (conserving mass)."""
        out = np.zeros_like(N)
        inn = np.zeros_like(N)
        for (sl_from, sl_to), give, amt in zip(DIRS, give_masks, amounts):
            a = np.where(give, amt, 0.0).astype(np.float32)
            out += a
            inn[sl_to] += a[sl_from]
        return inn - out

    # ------------------------------------------------------------ public actions
    def seed_heart(self, x, y):
        self.sub[y, x] = OPEN
        self.H[y, x] = NODE
        self.heart = (x, y)
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            if 0 <= x + dx < self.w and 0 <= y + dy < self.h and GROWABLE[self.sub[y + dy, x + dx]]:
                self.H[y + dy, x + dx] = TIP
        self.N[y - 1:y + 2, x - 1:x + 2] = 0.6

    def feed(self, x, y, amount):
        """Drop scrap into a Pit. It sits there as stock; the Mat draws it down at its own intake rate (no waste)."""
        self.pits[(x, y)] = self.pits.get((x, y), 0.0) + amount

    def _draw_pits(self, N):
        for (x, y), stock in list(self.pits.items()):
            if stock <= 0:
                continue
            y0, y1 = max(0, y - 1), min(self.h, y + 2)
            x0, x1 = max(0, x - 1), min(self.w, x + 2)
            patch = N[y0:y1, x0:x1]
            room = np.where(COND[self.sub[y0:y1, x0:x1]] > 0, np.minimum(self.pit_rate, 1.0 - patch), 0.0)
            room = np.maximum(room, 0.0)
            want = float(room.sum())
            if want <= 0:
                continue
            scale = min(1.0, stock / want)
            patch += (room * scale).astype(np.float32)
            self.pits[(x, y)] = stock - want * scale

    def sever(self, x, y):
        if self.H[y, x] != EMPTY:
            self.H[y, x] = EMPTY
            self.N[y, x] *= 0.5

    # ------------------------------------------------------------ signal (BFS from the Heart along hyphae)
    def compute_signal(self):
        self.sig[:] = 0.0
        if self.heart is None:
            return
        hyph = self.hyphal_mask()
        R = self.P["heart_range"]
        dist = np.full((self.h, self.w), -1.0, np.float32)
        hx, hy = self.heart
        if not hyph[hy, hx]:
            return
        dq = deque([(hx, hy)])
        dist[hy, hx] = 0.0
        cost = np.where(self.H == NODE, 0.5, np.where(self.H == TIP, 2.0, 1.0)).astype(np.float32)
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

    # ------------------------------------------------------------ nutrient
    def step_nutrient(self, release):
        hyph = self.hyphal_mask()
        kS = COND[self.sub]
        N = self.N
        self._draw_pits(N)
        # 1. diffusion (explicit; D*sum(k) <= 0.5 keeps it stable: 0.11*4 = 0.44)
        total = np.zeros_like(N)
        for Nn, hn, kn in zip(self._nbrs(N), self._nbrs(hyph), self._nbrs(kS)):
            k = np.where(hyph & hn, self.P["cord_k"], np.minimum(kS, kn))
            total += k * (Nn - N)
        N += self.D * total
        # 2. uptake: an empty cell next to hyphae loses a share of its N to that hyphal neighbour
        hn4 = self._nbrs(hyph)
        gives = [(~hyph) & hn & (kS > 0) for hn in hn4]
        N += self._transfer(N, gives, [self.uptake * N] * 4)
        # 3. Heart pump: a signalled hyphal cell pushes N to hyphal neighbours farther from the Heart
        sn4 = self._nbrs(self.sig)
        gives = [hyph & hn & (sn < self.sig) & (self.sig > 0) for hn, sn in zip(hn4, sn4)]
        c = np.maximum(1, sum(g.astype(np.int8) for g in gives))      # split the pumped share across outward links
        Nn4 = self._nbrs(N)
        # a full neighbour accepts nothing (1 - N_nbr): dead ends saturate and the flow re-routes to branches with demand
        surplus = np.maximum(0.0, N - (self.P["theta_starve"] + 0.05))  # a cell keeps a working reserve; only the surplus travels
        # a gift never exceeds the receiver's free capacity: dead ends saturate and stop drawing, the flow re-routes
        N += self._transfer(N, gives, [np.minimum(self.P["pump"] * surplus / c, np.maximum(0.0, 1.0 - Nn)) for Nn in Nn4])
        # 4. upkeep, decay, deposits, recycled necrosis
        N -= np.where(hyph, self.P["maint"], 0.0)
        N -= self.decay * N
        N += np.where(self.sub == DEPOSIT, 0.004, 0.0)
        N += release
        N[kS == 0] = 0.0
        np.clip(N, 0.0, 1.0, out=N)

    # ------------------------------------------------------------ hyphae
    def _try_sprout(self, H, N, ncount, occupied, cx, cy, max_nbrs):
        """Place a new tip in a growable empty neighbour of (cx,cy); filament rule: target has <= max_nbrs hyphal nbrs."""
        P, rng = self.P, self.rng
        # a hypha extends on its own reserve: it must hold theta_grow + the extension cost + the new tip's starter
        need = P["theta_grow"] + self.grow_cost + self.tip_starter   # reserve a hypha must hold before it will extend
        if N[cy, cx] < need:
            self.ev["fail_N"] += 1; return False
        if rng.random() > 0.7:
            self.ev["fail_rand"] += 1; return False
        cands, weights = [], []
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            ex, ey = cx + dx, cy + dy
            if not (0 <= ex < self.w and 0 <= ey < self.h):
                continue
            if occupied[ey, ex] or H[ey, ex] != EMPTY or not GROWABLE[self.sub[ey, ex]]:
                continue
            ne = ncount[ey, ex]
            if ne > max_nbrs:
                continue
            w = 0.25 + N[ey, ex]                     # chemotropism: richer ground ahead is favoured, never required
            if ne == 2:
                w *= 0.15                            # closing a loop is rare (anastomosis)
            cands.append((ex, ey))
            weights.append(w)
        if not cands:
            self.ev["fail_nocand"] += 1; return False
        weights = np.asarray(weights) / sum(weights)
        ex, ey = cands[rng.choice(len(cands), p=weights)]
        H[ey, ex] = TIP
        self.age[ey, ex] = 0
        self.stall[ey, ex] = 0
        occupied[ey, ex] = True
        N[cy, cx] -= self.grow_cost + self.tip_starter   # extension burns reserve and seeds the new tip
        N[ey, ex] = min(1.0, N[ey, ex] + self.tip_starter)
        return True

    def step_hyphae(self):
        P, rng = self.P, self.rng
        H_old = self.H.copy()
        H, N = self.H, self.N
        hyph = (H_old == TIP) | (H_old == CORD) | (H_old == NODE)
        ncount = self.hyphal_neighbour_count()
        release = np.zeros_like(N)
        self.age += 1

        # starvation: cords/nodes 24 ticks, tips 8 ticks under theta_starve -> necrotic (the Heart never starves)
        under = (N < P["theta_starve"]) & hyph
        self.starve = np.where(under, self.starve + 1, 0)
        die = ((self.starve >= 24) & ((H_old == CORD) | (H_old == NODE))) | ((self.starve >= 8) & (H_old == TIP))
        # isolation: 12 ticks with no hyphal neighbour -> necrotic
        self.lonely = np.where(hyph & (ncount == 0), self.lonely + 1, 0)
        die |= (self.lonely >= 12) & hyph
        if self.heart is not None:
            die[self.heart[1], self.heart[0]] = False
        self.ev['tip_starve'] += int((die & (H_old == TIP)).sum()); self.ev['cord_die'] += int((die & (H_old != TIP)).sum())
        H[die] = NECRO
        self.age[die] = 0
        self.starve[die] = 0

        # necrotic -> empty after 30 ticks; the dead cell returns 0.35 N to itself and 0.1 to each neighbour
        gone = (H_old == NECRO) & (self.age >= 30)
        rel = np.where(gone, 0.35, 0.0).astype(np.float32)
        release += rel
        for r in self._nbrs(rel):
            release += 0.1 * r
        H[gone] = EMPTY

        # maturation: a junction cord (>=3 hyphal nbrs) holding N >= 0.25 for 10 ticks becomes a Node (p=0.2/tick)
        elig = (H_old == CORD) & (ncount >= 3) & (N >= 0.25)
        self.stall = np.where(elig, self.stall + 1, np.where(H_old == CORD, 0, self.stall))
        mature = elig & (self.stall >= 10) & (rng.random(H.shape) < 0.2)
        H[mature] = NODE
        self.age[mature] = 0

        # anastomosis: a tip that touches two hyphal cells has met another strand -> cord (loop closed)
        fuse = (H_old == TIP) & (ncount >= 2)
        self.ev['fuse'] += int(fuse.sum())
        H[fuse] = CORD
        self.age[fuse] = 0

        occupied = hyph | (H == NECRO)

        # lateral branching: a well-fed strand cell (exactly 2 hyphal nbrs) sprouts a side tip
        lat = (H_old == CORD) & (ncount == 2) & (rng.random(H.shape) < P["branch_p"] * 0.05)
        for cy, cx in np.argwhere(lat):
            self.ev['lat_try'] += 1
            if self._try_sprout(H, N, ncount, occupied, cx, cy, max_nbrs=1): self.ev['lat_ok'] += 1

        # tip extension: one step per tip per tick, into a cell that touches at most 1 other hyphal cell
        tips = np.argwhere((H_old == TIP) & ~fuse & ~die)
        rng.shuffle(tips)
        for ty, tx in tips:
            self.ev['tip_try'] += 1
            if self._try_sprout(H, N, ncount, occupied, tx, ty, max_nbrs=2):
                self.ev['tip_ok'] += 1
                if rng.random() > P["branch_p"]:     # parent matures to cord unless it forks and stays a tip
                    H[ty, tx] = CORD
                    self.age[ty, tx] = 0
                self.stall[ty, tx] = 0
            else:
                self.stall[ty, tx] += 1
                if self.stall[ty, tx] >= 14:         # a tip that cannot advance for 10 ticks matures in place
                    self.ev['tip_stalled'] += 1
                    H[ty, tx] = CORD
                    self.age[ty, tx] = 0
                    self.stall[ty, tx] = 0
        return release

    # ------------------------------------------------------------ tick
    def step(self, feed_plan=None):
        self.tick += 1
        self.compute_signal()
        release = self.step_hyphae()
        if feed_plan:
            for (x, y, amt) in feed_plan:
                self.feed(x, y, amt)
        self.step_nutrient(release)
        s = dict(tick=self.tick, tips=int((self.H == TIP).sum()), cords=int((self.H == CORD).sum()),
                 nodes=int((self.H == NODE).sum()), necro=int((self.H == NECRO).sum()),
                 N=round(float(self.N.sum()), 1), reach=int(self._reach()))
        self.stats.append(s)
        return s

    def _reach(self):
        if self.heart is None:
            return 0
        ys, xs = np.nonzero(self.hyphal_mask())
        if len(xs) == 0:
            return 0
        return int(np.max(np.abs(xs - self.heart[0]) + np.abs(ys - self.heart[1])))

    # ------------------------------------------------------------ render
    def render(self, scale=4, pits=()):
        h, w = self.h, self.w
        img = np.zeros((h, w, 3), np.uint8)
        sub = self.sub
        img[:] = PAL["sump"]
        img[sub == ROCK] = PAL["culvert"]
        img[sub == BRICK] = PAL["rust"]
        img[sub == PIPE] = PAL["rustgray"]
        img[sub == WATER] = PAL["cistern"]
        img[sub == DEPOSIT] = PAL["rebar"]
        yy, xx = np.indices((h, w))
        d1 = ((xx + yy) % 2 == 0)
        empty = self.H == EMPTY
        glow = (self.N > 0.12) & (sub != ROCK) & empty
        strong = (self.N > 0.35) & (sub != ROCK) & empty
        img[glow & d1] = PAL["matdark"]
        img[strong] = PAL["matdark"]
        nec = self.H == NECRO
        img[nec] = np.where(self.age[nec][:, None] < 15, PAL["bruise"], PAL["silt"])
        img[self.H == CORD] = PAL["mat"]
        pulse = (self.H == CORD) & (((xx + yy + self.tick) % 7) == 0) & (self.sig > 0)
        img[pulse] = PAL["matlit"]
        img[self.H == TIP] = PAL["matlit"]
        img[self.H == NODE] = PAL["matglow"]
        for (px, py) in pits:
            img[py, px] = PAL["pink"]
        if self.heart is not None:
            hx, hy = self.heart
            img[hy, hx] = PAL["acid"]
        return Image.fromarray(img, "RGB").resize((w * scale, h * scale), Image.NEAREST)


def run(seed=7, ticks=600, cutoff=350):
    sim = MatSim(seed=seed)
    sim.seed_heart(96, 25)
    snaps = {}
    want = (40, 150, cutoff, ticks)
    for t in range(1, ticks + 1):
        plan = []
        if t <= cutoff:
            plan.append((96, 25, 0.60))                   # steady feed into the Heart Pit (0.6 N/tick, about one T2 unit per 2 ticks)
        if 120 <= t <= cutoff:
            plan.append((150, 12, 0.30))                  # second Pit, junk tier, north-east culvert
        s = sim.step(plan)
        if t in want:
            pits = [(96, 25)] + ([(150, 12)] if 120 <= t <= cutoff else [])
            snaps[t] = (sim.render(pits=pits), dict(s))
    return sim, snaps


def sheet(snaps, path, seed, cutoff):
    frames = list(snaps.items())
    fw, fh = frames[0][1][0].size
    pad, cap = 12, 22
    out = Image.new("RGB", (fw + pad * 2, pad + len(frames) * (fh + cap + pad)), PAL["sump"])
    d = ImageDraw.Draw(out)
    try:
        f = ImageFont.load_default(size=13)
    except TypeError:
        f = ImageFont.load_default()
    y = pad
    for t, (im, s) in frames:
        out.paste(im, (pad, y))
        feed = "feeding" if t <= cutoff else f"feed cut at t={cutoff}"
        txt = (f"t={t:>4}   tips {s['tips']:>3}   cords {s['cords']:>4}   nodes {s['nodes']:>3}   "
               f"necrotic {s['necro']:>3}   reach {s['reach']:>3} tiles   |   {feed}   |   seed {seed}")
        d.text((pad, y + fh + 4), txt, fill=PAL["chalk"], font=f)
        y += fh + cap + pad
    out.save(path)


if __name__ == "__main__":
    seed = int(sys.argv[1]) if len(sys.argv) > 1 else 7
    ticks = int(sys.argv[2]) if len(sys.argv) > 2 else 600
    cutoff = 350
    sim, snaps = run(seed, ticks, cutoff)
    sheet(snaps, f"{OUT}/mat_proto_seed{seed}.png", seed, cutoff)
    for t, (_, s) in snaps.items():
        print(s)
