"""Authoring script for data/chunks.toml — the run chunks, seven a stratum (Full Design, Content scope: 42): S1 Sump,
S2 Culverts, S3 Storm Drains (Phase B), S4 Service Tunnels, S5 the Vault, S6 Undercroft (PAC-081, the stack).
Run once; the TOML is the source of truth. Each chunk is 32 x 24 glyphs.

Glyphs
  #  rock            .  open floor/air    B  brick lining    P  pipe (wall)       ~  water (passable, slow)
  G  grate (extraction up)                D  scrap deposit (dig)                  X  collapse (dig to pass)
  *  light fixture (pool r4)              !  gas pocket                           L  locker (container, chunk tier)
  K  crate (container, tier +1)           R  corpse / body spawn                  S  sewerat spawn
  O  Oathkeeper patrol point              @  entry: the Mat edge (start chunks)   <  >  ^  v  exits
Exit windows are canonical so any east meets any west: horizontal exits occupy rows 11-13, vertical exits cols 14-17.
"""
import sys

W, H = 32, 24
EXIT_ROWS = (11, 12, 13)
EXIT_COLS = (14, 15, 16, 17)


class Canvas:
    def __init__(self):
        self.g = [["#"] * W for _ in range(H)]

    def rect(self, x0, y0, x1, y1, ch="."):
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                if 0 <= x < W and 0 <= y < H:
                    self.g[y][x] = ch

    def put(self, x, y, ch):
        self.g[y][x] = ch

    def hline(self, y, x0, x1, ch="."):
        self.rect(x0, y, x1, y, ch)

    def vline(self, x, y0, y1, ch="."):
        self.rect(x, y0, x, y1, ch)

    def line_brick(self):
        """Brick lining: every rock cell touching open becomes B."""
        for y in range(H):
            for x in range(W):
                if self.g[y][x] != "#":
                    continue
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < W and 0 <= ny < H and self.g[ny][nx] not in "#BP":
                        self.g[y][x] = "B"
                        break

    def exit_e(self):
        for r in EXIT_ROWS:
            self.g[r][W - 1] = ">"
            if self.g[r][W - 2] in "#B":
                self.g[r][W - 2] = "."

    def exit_w(self, entry=False):
        for r in EXIT_ROWS:
            self.g[r][0] = "@" if entry else "<"
            if self.g[r][1] in "#B":
                self.g[r][1] = "."

    def exit_n(self):
        for c in EXIT_COLS:
            self.g[0][c] = "^"
            if self.g[1][c] in "#B":
                self.g[1][c] = "."

    def exit_s(self):
        for c in EXIT_COLS:
            self.g[H - 1][c] = "v"
            if self.g[H - 2][c] in "#B":
                self.g[H - 2][c] = "."

    def rows(self):
        return ["".join(r) for r in self.g]


chunks = []


def add(id, stratum, kind, weight, canvas, note):
    chunks.append(dict(id=id, stratum=stratum, kind=kind, weight=weight, rows=canvas.rows(), note=note))


# ================================================================= S1 SUMP
c = Canvas()
c.rect(1, 10, 30, 14)                 # the culvert the Mat edge opens into
c.rect(4, 6, 9, 10)                   # a low alcove above the entry
c.hline(14, 1, 30, "~")               # standing water on the floor line
c.put(6, 13, "L"); c.put(8, 13, "D")
c.rect(24, 6, 29, 10); c.put(27, 9, "L"); c.put(26, 7, "*")
c.line_brick(); c.exit_w(entry=True); c.exit_e()
add("sump_entry", 1, "entry", 0, c, "where the Mat ends; shallow water, two lockers, the first deposit")

c = Canvas()
c.rect(1, 10, 30, 14); c.hline(14, 1, 30, "~")
c.rect(10, 4, 13, 10); c.rect(19, 4, 22, 10)          # two drops from the ceiling
c.put(11, 5, "D"); c.put(21, 5, "*"); c.put(5, 13, "S")
c.line_brick(); c.exit_w(); c.exit_e()
add("sump_culvert", 1, "culvert", 3, c, "a straight culvert with two ceiling drops; a sewerat in the dark end")

c = Canvas()
c.rect(1, 4, 30, 20); c.rect(1, 15, 30, 20, "~")       # a flooded cistern
c.rect(3, 10, 8, 14); c.rect(23, 10, 28, 14)           # ledges above the water
c.rect(12, 6, 19, 8)                                   # a catwalk
c.rect(14, 1, 17, 4)                                   # the throat up to the north exit
c.put(5, 13, "L"); c.put(25, 13, "L"); c.put(15, 7, "K"); c.put(15, 5, "*"); c.put(10, 19, "D"); c.put(20, 19, "R")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_n()
add("sump_cistern", 1, "room", 2, c, "a drowned cistern; lockers on the ledges, a crate on the catwalk, a body in the water")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(14, 10, 17, 14, "X")     # a collapse across the culvert
c.rect(14, 6, 17, 9)                                   # a crawlspace over it
c.put(13, 9, "."); c.put(18, 9, ".")                   # ...reached from either side
c.put(15, 7, "D"); c.put(16, 8, "S"); c.put(7, 13, "D"); c.put(24, 13, "!")
c.line_brick(); c.exit_w(); c.exit_e()
add("sump_collapse", 1, "collapse", 2, c, "rubble across the way: dig through or crawl over it with the sewerat")

c = Canvas()
c.rect(14, 1, 17, 22)                                  # the shaft
c.rect(1, 10, 30, 14)                                  # the culvert it crosses
c.rect(12, 18, 19, 22); c.put(13, 21, "L"); c.put(18, 21, "D"); c.put(15, 3, "*"); c.put(6, 13, "R")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_n(); c.exit_s()
add("sump_shaft", 1, "shaft", 2, c, "a vertical shaft crossing the culvert; a locker in the sump below, a light above")

c = Canvas()
c.rect(1, 10, 30, 14); c.hline(14, 1, 30, "~")
c.rect(14, 14, 17, 22, "~")                            # the culvert drains south through a flooded throat
c.rect(8, 17, 23, 21); c.hline(21, 8, 23, "~")         # into a low sump
c.put(10, 20, "D"); c.put(21, 20, "L"); c.put(12, 18, "S"); c.put(4, 11, "*")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_s()
add("sump_bend", 1, "bend", 2, c, "the culvert drains south through a flooded throat into a low sump")

# ================================================================= S2 CULVERTS
c = Canvas()
c.rect(1, 9, 30, 15)
c.hline(9, 1, 30, "P")                                 # pipe run under the ceiling
c.put(15, 11, "*")
c.rect(4, 4, 9, 9); c.rect(22, 4, 27, 9)               # brick arches
c.put(6, 8, "L"); c.put(24, 8, "L"); c.put(12, 14, "D"); c.put(19, 14, "K")
c.line_brick(); c.exit_w(entry=True); c.exit_e()
add("culvert_entry", 2, "entry", 0, c, "the Mat edge at the Culvert tier: brick arches, a pipe run, a crate")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(14, 1, 17, 22)
c.rect(4, 4, 11, 8); c.rect(20, 4, 27, 8)
c.hline(6, 12, 13); c.hline(6, 18, 19)                 # doorways from the shaft
c.put(15, 11, "O"); c.put(7, 7, "*"); c.put(23, 7, "L"); c.put(5, 13, "D")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_n(); c.exit_s()
add("culvert_junction", 2, "junction", 3, c, "a four-way under a light; a patrol point in the middle of it")

c = Canvas()
c.rect(1, 10, 30, 14)
c.rect(6, 3, 25, 9)                                    # the pump room above the culvert
c.hline(3, 6, 25, "P"); c.vline(10, 4, 8, "P"); c.vline(21, 4, 8, "P")
c.rect(12, 15, 19, 21)                                 # a sump below, open to the culvert
c.put(8, 8, "K"); c.put(23, 8, "K"); c.put(15, 5, "*"); c.put(14, 20, "D"); c.put(17, 20, "~"); c.put(28, 13, "S")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_s()
add("culvert_pumproom", 2, "room", 2, c, "a pump room of pipes; two crates, a flooded sump below")

c = Canvas()
c.rect(1, 10, 30, 14)
c.rect(13, 1, 18, 9)                                   # the climb to a grate
c.hline(1, 13, 18, "G"); c.put(15, 4, "*"); c.put(12, 13, "O"); c.put(20, 13, "O"); c.put(7, 13, "L"); c.put(25, 13, "D")
c.line_brick(); c.exit_w(); c.exit_e()
add("culvert_grate", 2, "grate", 2, c, "a street grate: neon leaks down, two scouts under it; extract here or go on")

c = Canvas()
c.rect(1, 10, 30, 14)
c.rect(8, 4, 23, 9)                                    # the nest
c.put(10, 8, "S"); c.put(15, 8, "S"); c.put(20, 8, "S"); c.put(12, 6, "R"); c.put(18, 6, "R"); c.put(15, 5, "D"); c.put(4, 13, "!")
c.line_brick(); c.exit_w(); c.exit_e()
add("culvert_nest", 2, "nest", 1, c, "a sewerat nest with two bodies in it and gas at the door")

c = Canvas()
c.rect(1, 10, 9, 14); c.rect(22, 10, 30, 14)           # two dry ends
c.rect(9, 10, 22, 18, "~"); c.rect(9, 10, 22, 11, "B") # a siphon: the way through is under water
c.hline(9, 10, 21, "P")
c.put(12, 17, "D"); c.put(19, 17, "K"); c.put(4, 13, "L"); c.put(26, 13, "S"); c.put(26, 11, "*")
c.line_brick(); c.exit_w(); c.exit_e()
add("culvert_siphon", 2, "siphon", 2, c, "a siphon: the only way on is under the water, where a crate sank")

# ================================================================= S3 STORM DRAINS
c = Canvas()
c.rect(1, 9, 30, 15); c.hline(15, 1, 30, "~")
c.rect(4, 3, 10, 9); c.put(7, 8, "L"); c.put(7, 5, "*")
c.rect(21, 3, 27, 9); c.put(24, 8, "K")
c.put(15, 14, "D")
c.line_brick(); c.exit_w(entry=True); c.exit_e()
add("drain_entry", 3, "entry", 0, c, "the Mat edge at the Storm Drains: wide, wet, two alcoves")

c = Canvas()
c.rect(1, 8, 30, 16); c.rect(1, 13, 30, 16, "~")       # a deep channel
c.rect(5, 4, 8, 8); c.rect(24, 4, 27, 8); c.put(6, 7, "D"); c.put(25, 7, "L"); c.put(15, 9, "*")
c.put(12, 12, "R"); c.put(20, 12, "S")
c.line_brick(); c.exit_w(); c.exit_e()
add("drain_fall", 3, "culvert", 3, c, "a drain channel: deep water on the floor, a body at the edge; Flood fills it fast")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(14, 1, 17, 10)
c.rect(5, 16, 12, 21); c.rect(19, 16, 26, 21)
c.put(8, 15, "."); c.put(23, 15, ".")                  # drop-ins
c.put(7, 20, "!"); c.put(10, 20, "D"); c.put(21, 20, "!"); c.put(24, 20, "D"); c.put(15, 3, "*")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_n()
add("drain_gas", 3, "hazard", 2, c, "gas pockets in two pits below the way, each with a deposit in it")

c = Canvas()
c.rect(1, 10, 30, 14)
c.rect(22, 4, 28, 8); c.put(23, 9, "X"); c.put(24, 9, "X")   # a sealed side room; rubble in its floor
c.put(23, 8, "K"); c.put(27, 8, "K"); c.put(25, 5, "*"); c.put(5, 13, "L"); c.put(12, 13, "O")
c.line_brick(); c.exit_w(); c.exit_e()
add("drain_vaultdoor", 3, "vault", 1, c, "a side room sealed with rubble; two crates inside, a scout outside")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(10, 15, 21, 21)
c.put(12, 20, "R"); c.put(15, 20, "R"); c.put(19, 20, "R"); c.put(15, 17, "S"); c.put(25, 13, "D"); c.put(6, 11, "*")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_s()
add("drain_bodies", 3, "nest", 1, c, "a pit of newcomer bodies with something feeding on them")

c = Canvas()
c.rect(1, 6, 30, 18); c.rect(1, 15, 30, 18, "~")       # an outfall chamber
c.rect(14, 1, 17, 6)                                   # a shaft up
c.rect(4, 9, 9, 14); c.rect(22, 9, 27, 14)             # two ledges
c.put(6, 13, "L"); c.put(24, 13, "K"); c.put(15, 3, "*"); c.put(15, 8, "*"); c.put(12, 17, "R"); c.put(19, 17, "S"); c.put(8, 17, "D")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_n()
add("drain_outfall", 3, "room", 2, c, "an outfall chamber under a shaft; ledges over deep water")

c = Canvas()
c.rect(1, 9, 30, 15); c.hline(9, 1, 30, "P"); c.hline(15, 1, 30, "~")
c.rect(11, 4, 14, 9); c.rect(19, 4, 22, 9)             # two ceiling pockets
c.put(12, 5, "D"); c.put(20, 5, "L"); c.put(15, 10, "*"); c.put(6, 14, "O"); c.put(25, 14, "O")
c.line_brick(); c.exit_w(); c.exit_e()
add("drain_tunnel", 3, "culvert", 3, c, "a long storm tunnel: pipes overhead, a patrol walking its length")

# ================================================================= S1, S2: the seventh of each (PAC-081)
c = Canvas()
c.rect(1, 10, 30, 14); c.rect(4, 15, 27, 19)            # silt flats under the culvert
c.hline(19, 4, 27, "~")
c.put(6, 18, "D"); c.put(11, 18, "D"); c.put(20, 18, "D"); c.put(25, 17, "R"); c.put(15, 11, "*"); c.put(28, 13, "S")
c.line_brick(); c.exit_w(); c.exit_e()
add("sump_silt", 1, "flats", 2, c, "silt flats below the culvert: three deposits in the mud, a body half under it")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(10, 4, 21, 9); c.hline(9, 10, 21, "~")
c.rect(14, 15, 17, 22)                                  # the overflow drops south
c.put(12, 6, "K"); c.put(19, 6, "L"); c.put(15, 5, "*"); c.put(15, 20, "D"); c.put(4, 13, "O")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_s()
add("culvert_weir", 2, "room", 2, c, "an overflow weir: a holding tank above the culvert, a drop below it, a scout at the door")

# ================================================================= S4 SERVICE TUNNELS
c = Canvas()
c.rect(1, 10, 30, 14); c.hline(9, 1, 30, "P"); c.hline(15, 1, 30, "P")
c.rect(22, 4, 28, 8); c.put(25, 9, ".")                # a fuse room over the conduit, a gap in it
c.put(25, 7, "K"); c.put(23, 5, "*"); c.put(8, 13, "D"); c.put(14, 13, "L")
c.line_brick(); c.exit_w(entry=True); c.exit_e()
add("service_entry", 4, "entry", 0, c, "the Mat edge in the Service Tunnels: conduits above and below, a fuse locker")

c = Canvas()
c.rect(1, 10, 30, 14); c.hline(9, 1, 30, "P"); c.hline(15, 1, 30, "P")
c.put(6, 11, "*"); c.put(24, 11, "*"); c.put(10, 13, "O"); c.put(20, 13, "O"); c.put(15, 13, "D")
c.line_brick(); c.exit_w(); c.exit_e()
add("service_run", 4, "culvert", 3, c, "a conduit run the Badges walk: lit at both ends, two scouts on its length")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(14, 1, 17, 22)
c.rect(3, 3, 11, 8); c.rect(20, 16, 28, 21)
c.hline(6, 12, 13); c.hline(18, 18, 19)
c.put(5, 7, "K"); c.put(9, 7, "K"); c.put(24, 20, "L"); c.put(15, 11, "*"); c.put(26, 17, "S")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_n(); c.exit_s()
add("service_junction", 4, "junction", 3, c, "a junction box: a supply room above, a cable vault below, four ways out")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(8, 3, 23, 9)
c.vline(11, 4, 8, "P"); c.vline(15, 4, 8, "P"); c.vline(19, 4, 8, "P")
c.put(9, 8, "!"); c.put(22, 8, "!"); c.put(13, 5, "K"); c.put(17, 5, "D"); c.put(15, 4, "*")
c.line_brick(); c.exit_w(); c.exit_e()
add("service_transformer", 4, "hazard", 2, c, "a transformer hall: leaking coolant pockets by the coils, a crate between them")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(13, 1, 18, 9); c.hline(1, 13, 18, "G")
c.put(15, 4, "*"); c.put(11, 13, "O"); c.put(21, 13, "O"); c.put(6, 13, "L"); c.put(26, 13, "K")
c.line_brick(); c.exit_w(); c.exit_e()
add("service_grate", 4, "grate", 2, c, "a maintenance grate to the street: Badge patrols at the foot of it")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(14, 10, 17, 14, "X"); c.rect(8, 18, 23, 21)
c.rect(8, 15, 9, 17); c.rect(22, 15, 23, 17)             # drops either side of the rubble to the crawl
c.put(12, 20, "D"); c.put(19, 20, "R"); c.put(15, 19, "S"); c.put(5, 11, "*")
c.line_brick(); c.exit_w(); c.exit_e()
add("service_cavein", 4, "collapse", 2, c, "a cave-in across the tunnel: dig through, or drop to the crawl below it")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(6, 3, 25, 8)
c.put(8, 7, "R"); c.put(12, 7, "R"); c.put(16, 7, "S"); c.put(20, 7, "S"); c.put(23, 4, "K"); c.put(15, 9, "."); c.put(4, 13, "!")
c.line_brick(); c.exit_w(); c.exit_e()
add("service_nest", 4, "nest", 1, c, "a crew room the sewerats took: bodies of the last crew, a crate they never opened")

# ================================================================= S5 THE VAULT
c = Canvas()
c.rect(1, 9, 30, 15); c.hline(15, 1, 30, "~")
c.rect(20, 3, 28, 8); c.put(22, 7, "K"); c.put(26, 7, "K"); c.put(24, 4, "*"); c.put(7, 14, "D")
c.line_brick(); c.exit_w(entry=True); c.exit_e()
add("vault_entry", 5, "entry", 0, c, "the Mat edge at the Vault: a drowned lobby, two shipping crates behind the desk")

c = Canvas()
c.rect(1, 10, 30, 14)
for x in (4, 9, 14, 19, 24):
    c.rect(x, 3, x + 2, 9); c.put(x + 1, 8, "L")
c.put(15, 11, "*"); c.put(27, 13, "O")
c.line_brick(); c.exit_w(); c.exit_e()
add("vault_stacks", 5, "room", 3, c, "the archive stacks: five aisles of lockers and a security scout at the end")

c = Canvas()
c.rect(1, 6, 30, 18); c.rect(1, 13, 30, 18, "~")
c.rect(6, 7, 9, 12); c.rect(22, 7, 25, 12)
c.put(7, 11, "K"); c.put(23, 11, "K"); c.put(15, 17, "R"); c.put(11, 17, "D"); c.put(15, 8, "*"); c.put(19, 17, "S")
c.line_brick(); c.exit_w(); c.exit_e()
add("vault_servers", 5, "culvert", 3, c, "a flooded server hall: racks on two islands, a body among the cables")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(20, 3, 29, 9); c.put(19, 9, "X"); c.put(20, 9, "X")
c.put(22, 8, "K"); c.put(25, 8, "K"); c.put(28, 8, "K"); c.put(24, 4, "*"); c.put(8, 13, "O"); c.put(12, 13, "O")
c.line_brick(); c.exit_w(); c.exit_e()
add("vault_door", 5, "vault", 2, c, "the strongroom: its door caved in, three crates inside, two guards outside")

c = Canvas()
c.rect(14, 1, 17, 22); c.rect(1, 10, 30, 14)
c.rect(18, 3, 24, 7); c.hline(5, 18, 18)
c.put(21, 6, "K"); c.put(15, 5, "*"); c.put(15, 19, "!"); c.put(6, 13, "L")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_n(); c.exit_s()
add("vault_lift", 5, "shaft", 2, c, "a dead elevator shaft: gas in the pit, an executive's crate on the landing")

c = Canvas()
c.rect(1, 10, 30, 14); c.hline(9, 1, 30, "P")
c.put(5, 11, "*"); c.put(26, 11, "*"); c.put(10, 13, "O"); c.put(16, 13, "O"); c.put(22, 13, "O"); c.put(14, 13, "D")
c.line_brick(); c.exit_w(); c.exit_e()
add("vault_security", 5, "culvert", 2, c, "the security corridor: lit, three posts, nowhere to hide but the dark between")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(8, 15, 23, 21)
c.put(10, 20, "S"); c.put(15, 20, "S"); c.put(20, 20, "S"); c.put(12, 17, "R"); c.put(18, 17, "K"); c.put(4, 13, "!")
c.put(15, 14, ".")
c.line_brick(); c.exit_w(); c.exit_e()
add("vault_nest", 5, "nest", 1, c, "the cafeteria the sewerats live in now; one crate under the counter")

# ================================================================= S6 UNDERCROFT
c = Canvas()
c.rect(1, 9, 30, 15); c.hline(15, 1, 30, "B")
c.rect(13, 1, 18, 8); c.hline(1, 13, 18, "G")
c.put(15, 4, "*"); c.put(6, 14, "L"); c.put(24, 14, "K")
c.line_brick(); c.exit_w(entry=True); c.exit_e()
add("under_entry", 6, "entry", 0, c, "the Mat edge in the Undercroft: a grate overhead leaks the street's neon")

c = Canvas()
c.rect(1, 8, 30, 16); c.rect(1, 14, 30, 16, "B")       # a metro platform over the track bed
c.rect(1, 17, 30, 19); c.rect(4, 14, 5, 16); c.rect(26, 14, 27, 16)   # the track bed, steps down at each end
c.put(5, 9, "*"); c.put(26, 9, "*"); c.put(10, 13, "L"); c.put(20, 13, "K"); c.put(15, 13, "O"); c.put(8, 18, "R"); c.put(22, 18, "D")
c.line_brick(); c.exit_w(); c.exit_e()
add("under_platform", 6, "room", 3, c, "a dead metro platform: lockers on it, a body on the track bed below")

c = Canvas()
c.rect(1, 10, 30, 14)
c.put(4, 11, "*"); c.put(28, 11, "*"); c.put(9, 13, "O"); c.put(21, 13, "O"); c.put(15, 13, "D"); c.put(15, 11, "S")
c.line_brick(); c.exit_w(); c.exit_e()
add("under_tunnel", 6, "culvert", 3, c, "a metro tunnel: a patrol at each end and something in the middle")

c = Canvas()
c.rect(1, 10, 30, 14)
c.rect(5, 2, 9, 9); c.hline(2, 5, 9, "G"); c.rect(22, 2, 26, 9); c.hline(2, 22, 26, "G")
c.put(7, 5, "*"); c.put(24, 5, "*"); c.put(15, 13, "O"); c.put(12, 13, "L"); c.put(18, 13, "K")
c.line_brick(); c.exit_w(); c.exit_e()
add("under_grates", 6, "grate", 2, c, "two grates to the street and the Oathkeepers who watch them")

c = Canvas()
c.rect(1, 10, 30, 14); c.hline(9, 1, 30, "P"); c.rect(10, 15, 21, 20); c.hline(20, 10, 21, "~")
c.put(15, 14, "."); c.put(12, 19, "D"); c.put(19, 19, "K"); c.put(15, 16, "!"); c.put(6, 11, "*")
c.line_brick(); c.exit_w(); c.exit_e()
add("under_gallery", 6, "hazard", 2, c, "a utility gallery over a gassed sump; a crate where the gas is thickest")

c = Canvas()
c.rect(14, 1, 17, 22); c.rect(1, 10, 30, 14)
c.rect(3, 15, 11, 20); c.rect(20, 3, 28, 8)
c.hline(17, 12, 13); c.hline(6, 18, 19)
c.put(6, 19, "L"); c.put(24, 7, "K"); c.put(15, 11, "*"); c.put(8, 16, "S")
c.line_brick(); c.exit_w(); c.exit_e(); c.exit_n(); c.exit_s()
add("under_station", 6, "junction", 2, c, "a station interchange: a ticket hall above, a staff room below")

c = Canvas()
c.rect(1, 10, 30, 14); c.rect(14, 10, 17, 14, "X"); c.rect(13, 4, 18, 9)
c.put(13, 9, "."); c.put(18, 9, "."); c.put(15, 6, "K"); c.put(16, 7, "S"); c.put(6, 13, "D"); c.put(25, 13, "O")
c.line_brick(); c.exit_w(); c.exit_e()
add("under_collapse", 6, "collapse", 2, c, "a collapsed crossover: dig through or climb over, with a scout waiting beyond")


def toml_str(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


out = ["# Cyberpunk: ROSE — run chunks. 32 x 24 glyphs each; see tools/gen/gen_chunks.py for the legend. Source of truth for the `run` crate.", ""]
for ch in chunks:
    assert all(len(r) == W for r in ch["rows"]) and len(ch["rows"]) == H
    out.append("[[chunk]]")
    out.append(f'id = {toml_str(ch["id"])}')
    out.append(f'stratum = {ch["stratum"]}')
    out.append(f'kind = {toml_str(ch["kind"])}')
    out.append(f'weight = {ch["weight"]}')
    out.append(f'note = {toml_str(ch["note"])}')
    out.append("rows = [")
    for r in ch["rows"]:
        out.append(f'  "{r}",')
    out.append("]")
    out.append("")
path = sys.argv[1] if len(sys.argv) > 1 else "data/chunks.toml"
open(path, "w").write("\n".join(out))
print(len(chunks), "chunks ->", path)
