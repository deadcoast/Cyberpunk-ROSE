"""Evidence sheet for the GDD: four runs of the Mat CA, same seed, different genomes / feed schedules."""
import time
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import src.mat_sim_proto as m

SEED = 11
HEART = (96, 25)
PIT2 = (150, 12)


def run(genome, ticks, cutoff, label):
    sim = m.MatSim(seed=SEED, genome=genome)
    sim.seed_heart(*HEART)
    t0 = time.perf_counter()
    for t in range(1, ticks + 1):
        plan = []
        if t <= cutoff:
            plan.append((*HEART, 0.60))
        if 120 <= t <= cutoff:
            plan.append((*PIT2, 0.30))
        s = sim.step(plan)
    dt = (time.perf_counter() - t0) / ticks * 1000
    pits = [HEART] + ([PIT2] if ticks <= cutoff else [])
    return sim.render(pits=pits), s, dt, label


G0 = [0.5] * 10
G_runner = list(G0); G_runner[0] = 0.9; G_runner[8] = 0.9          # Vigor, Conductance high
G_bramble = list(G0); G_bramble[1] = 0.9                           # Branching high

frames = [
    run(G0, 350, 350, "default genome (all genes 0.5) · fed to t=350"),
    run(G0, 600, 350, "default genome · feed cut at t=350, shown at t=600"),
    run(G_runner, 350, 350, "'Runner' strain: Vigor 0.9, Conductance 0.9 · fed to t=350"),
    run(G_bramble, 350, 350, "'Bramble' strain: Branching 0.9 · fed to t=350"),
]

# determinism check
a = run(G0, 200, 200, "")[1]
b = run(G0, 200, 200, "")[1]
assert a == b, "same seed must give identical state"

fw, fh = frames[0][0].size
pad, cap = 12, 24
out = Image.new("RGB", (fw + pad * 2, pad + len(frames) * (fh + cap + pad)), m.PAL["sump"])
d = ImageDraw.Draw(out)
try:
    f = ImageFont.load_default(size=13)
except TypeError:
    f = ImageFont.load_default()
y = pad
for im, s, dt, label in frames:
    out.paste(im, (pad, y))
    txt = (f"{label}   |   cords {s['cords']:>3}  nodes {s['nodes']:>3}  necrotic {s['necro']:>3}  "
           f"reach {s['reach']:>3} tiles   |   {dt:.1f} ms/tick (numpy)")
    d.text((pad, y + fh + 5), txt, fill=m.PAL["chalk"], font=f)
    y += fh + cap + pad
out.save(f"{m.OUT}/mat_evidence.png")
for im, s, dt, label in frames:
    print(label, s, f"{dt:.1f} ms/tick")
print("determinism: ok")
