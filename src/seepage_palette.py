"""SEEPAGE palette for Cyberpunk: ROSE — swatch sheet (PNG) + GIMP/Aseprite palette (.gpl)."""
from PIL import Image, ImageDraw, ImageFont

# (group, name, hex, role)
PALETTE = [
    ("NOIR  · ambient, shadow", "Void",            "#07070C", "ink outline, deepest shadow"),
    ("NOIR  · ambient, shadow", "Sump",            "#0E1018", "CanvasModulate ambient"),
    ("NOIR  · ambient, shadow", "Culvert",         "#171A26", "unlit concrete"),
    ("NOIR  · ambient, shadow", "Concrete Night",  "#232737", "lit-edge concrete"),
    ("NOIR  · ambient, shadow", "Wet Concrete",    "#343A4F", "wet surfaces in light"),
    ("INDUSTRIAL · gray",       "Rust Gray",       "#4A4F5E", "steel, pipe shadow"),
    ("INDUSTRIAL · gray",       "Rebar",           "#6B6F7B", "steel in light, chrome inclusions"),
    ("INDUSTRIAL · gray",       "Dust",            "#8E9097", "worn edges"),
    ("INDUSTRIAL · gray",       "Chalk",           "#B5B4B0", "chrome highlight, bone"),
    ("MUD · brown",             "Silt",            "#2A1F19", "dead organic, dry necrosis"),
    ("MUD · brown",             "Sewer Brown",     "#4A3526", "brick shadow, muck"),
    ("MUD · brown",             "Rust",            "#6E4E34", "brick, corroded pipe"),
    ("MUD · brown",             "Oxide",           "#9A7049", "brick in light, leather"),
    ("BILE · vomit green",      "Bile",            "#2E3A1E", "standing water shadow"),
    ("BILE · vomit green",      "Vomit Green",     "#4E5E27", "standing water, mold"),
    ("BILE · vomit green",      "Pond Scum",       "#7A8A3A", "water surface in light"),
    ("DRAIN · deep blue",       "Drain Blue",      "#122238", "deep water"),
    ("DRAIN · deep blue",       "Cistern",         "#1F3D63", "water body"),
    ("DRAIN · deep blue",       "Flood Light",     "#35679C", "moving water highlight"),
    ("FLESH · symbiote skin",   "Bruise",          "#5B3A4A", "deep-mod flesh, Ivan, fresh necrosis"),
    ("FLESH · symbiote skin",   "Flesh Dim",       "#9A6070", "slimo skin shadow"),
    ("FLESH · symbiote skin",   "Flesh",           "#D99AA6", "slimo skin lit"),
    ("MAT · biolume (emits)",   "Mat Dark",        "#0F2E1C", "cord core"),
    ("MAT · biolume (emits)",   "Mat",             "#1F6E3A", "cord body"),
    ("MAT · biolume (emits)",   "Mat Lit",         "#3FD66E", "cord edge, tip"),
    ("MAT · biolume (emits)",   "Mat Glow",        "#B6FFC8", "node bulb, veins, signal pulse"),
    ("NEON · emitters only",    "Hot Pink",        "#FF2E97", "Night City leak, UI alert"),
    ("NEON · emitters only",    "Cyan",            "#19E6FF", "Night City leak, terminal text"),
    ("NEON · emitters only",    "Acid Yellow",     "#F6FF2E", "callouts, hazard, Night City leak"),
    ("NEON · emitters only",    "Deep Purple",     "#7A1FFF", "Ivan's lume, Relic events"),
    ("NEON · emitters only",    "Muzzle Orange",   "#FF6A1A", "gunfire, sparks, flamer"),
    ("WHITE · Oathkeepers",     "Vestment White",  "#F2EFE4", "Oathkeeper light and cloth; the only pure white"),
    ("WHITE · Oathkeepers",     "Reliquary Gold",  "#D4A52A", "Oathkeeper trim, Relic tier"),
]

def hex_rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i+2], 16) for i in (0, 2, 4))

def write_gpl(path):
    with open(path, "w") as f:
        f.write("GIMP Palette\nName: SEEPAGE (Cyberpunk ROSE)\nColumns: 8\n#\n")
        for _, name, hx, _ in PALETTE:
            r, g, b = hex_rgb(hx)
            f.write(f"{r:3d} {g:3d} {b:3d}\t{name}\n")

def render_sheet(path):
    sw, sh = 72, 56           # swatch size
    pad, gap = 16, 6
    label_w = 260
    groups = []
    for g, *_ in PALETTE:
        if g not in groups:
            groups.append(g)
    rows = []  # (group_label or None, [entries])
    for g in groups:
        rows.append((g, [p for p in PALETTE if p[0] == g]))
    W = pad * 2 + label_w + 5 * (sw + gap) + 200
    H = pad * 2 + 44 + len(rows) * (sh + 34)
    img = Image.new("RGB", (W, H), hex_rgb("#0E1018"))
    d = ImageDraw.Draw(img)
    try:
        font = ImageFont.load_default(size=13)
        small = ImageFont.load_default(size=11)
        big = ImageFont.load_default(size=18)
    except TypeError:
        font = small = big = ImageFont.load_default()
    d.text((pad, pad), "SEEPAGE  ·  Cyberpunk: ROSE pixel palette  ·  33 colours", fill=hex_rgb("#B5B4B0"), font=big)
    d.text((pad, pad + 24), "Rule: ramps NOIR–FLESH are surface colours. MAT, NEON, WHITE appear only as light: emitters, UI, leaks.",
           fill=hex_rgb("#6B6F7B"), font=small)
    y = pad + 56
    for g, entries in rows:
        d.text((pad, y + 4), g, fill=hex_rgb("#8E9097"), font=font)
        x = pad + label_w
        for _, name, hx, role in entries:
            d.rectangle([x, y, x + sw, y + sh], fill=hex_rgb(hx), outline=hex_rgb("#07070C"))
            d.text((x, y + sh + 3), name, fill=hex_rgb("#B5B4B0"), font=small)
            d.text((x, y + sh + 15), hx, fill=hex_rgb("#6B6F7B"), font=small)
            x += sw + gap
        # role notes to the right of the row
        notes = " / ".join(e[3] for e in entries)
        d.text((x + 10, y + 4), notes[:60], fill=hex_rgb("#4A4F5E"), font=small)
        if len(notes) > 60:
            d.text((x + 10, y + 18), notes[60:120], fill=hex_rgb("#4A4F5E"), font=small)
        if len(notes) > 120:
            d.text((x + 10, y + 32), notes[120:180], fill=hex_rgb("#4A4F5E"), font=small)
        y += sh + 34
    img.save(path)

if __name__ == "__main__":
    import os; out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "out"); os.makedirs(out, exist_ok=True)
    write_gpl(os.path.join(out, "SEEPAGE.gpl"))
    render_sheet(os.path.join(out, "seepage_palette.png"))
    print("ok", len(PALETTE))
