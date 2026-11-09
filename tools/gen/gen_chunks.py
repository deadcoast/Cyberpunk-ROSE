"""Authoring script for data/chunks.toml — the first 14 run chunks (S1 Sump, S2 Culverts, S3 Storm Drains).
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
