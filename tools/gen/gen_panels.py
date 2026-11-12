"""Authoring script for data/chapters.toml and game/assets/manifest/panels.toml (Full Design v3.19, Narrative and
Ivan: Panels). Run once; the two TOML files are the source of truth after it: chapters.toml carries the captions the
game shows, panels.toml the slots and PixelLab prompts the art pipeline reads. Captions are Ivan's, two sentences at
most, and name the world only through the string table's tokens ({order}, {city_above}, {fallen_corp}, {people},
{ferals}, {currency}, {city_below}); {name} and {teller} are filled by the campaign.
"""
import os

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
STYLE = "graphic-novel still, hard ink outlines, high contrast chiaroscuro, SEEPAGE palette, purple ooze light, neon only as light, text box space along the bottom"
NEG = "soft shading, photoreal, bright daylight, green tint, readable text, logos"

# (n, id, name, opens, goal, card, beat, open[4], close[4], close_refused[4] or None)
# each panel: (caption, scene)
CHAPTERS = [
    (1, "seepage", "Seepage", "the founding", "8 slimos, every one with a bed", "8 SLIMOS WITH BEDS", "Ivan's welcome",
     [("Welcome to the bottom of {city_above}. Whatever you were up there, down here you're one of the {people}.", "Ivan Ooze in the black water of the Sump, arms open, chrome flecks catching purple light, a grate far above"),
      ("The {order} came down from the Moon with a cure for cyberpsychosis. It cured the chrome right out of us.", "a white-robed lunar priest with a syringe gun over a crowd, vaccine posters, a chromed arm melting into flesh"),
      ("The ones the cure ruined, they flushed. We grew where the flush landed.", "bodies sliding down a storm drain chute into darkness, one hand still reaching"),
      ("That's the Heart. Feed it, and the Mat goes where you point it.", "the Heart: a pulsing purple ooze mass with a scrap Pit beside it, cords spreading into culverts")],
     [("Eight of us under a roof. In the Sump that's a census.", "eight slimos of different depths asleep in scrap Hab-Pods, ooze light on their faces"),
      ("Every one that wakes down here gets a name and a bed, or the drain takes them back.", "a newcomer waking on wet concrete, a slimo offering a hand"),
      ("Write it on the wall, tweaker: the Heart is beating.", "a slimo painting a tag on brick by the Heart's glow"),
      ("Now let's see what this place can make.", "Ivan grinning in close-up, scrap piled behind him")],
     None),
    (2, "tweakers", "Tweakers", "chapter 1 closed", "a Scrap Forge standing and a sale at a Seep Market", "A FORGE AND A SALE", "the first line about eddies",
     [("Give a tech-tweaker a pile of scrap and a Forge, and stand back.", "a D3 tweaker with welding goggles at a scrap Forge, sparks in Muzzle orange"),
      ("Up the grates there are people who'll pay {currency} for what we pull out of the dark.", "a hand passing a chrome part up through a grate to a gloved hand, neon street above"),
      ("Every piece of chrome is a choice: sell it up, or feed it down.", "a slimo holding chrome over a Pit with a market stall lit behind"),
      ("Sell, and we eat. Feed, and the Mat grows into something that might eat them.", "the Mat's cords thickening around a Pit, a shadow of a city above")],
     [("First {currency} in the till. Smells like rust and other people's money.", "a tin cash box with a few chips, ooze fingerprints on it"),
      ("The Forge is hot, the stall is open, and nobody up top knows our name.", "the Forge glowing beside a Seep Market stall under a grate"),
      ("Keep it that way as long as you can.", "Ivan in shadow with a finger to his lips"),
      ("It won't be long.", "a white light glint at the far end of a culvert")],
     None),
    (3, "the_hand", "The Hand", "chapter 2 closed and the Culverts reached", "the envoy answered and the accord's Cleansing gone", "ANSWER THE ENVOY", "the betrayal",
     [("White light at the grate, and nobody's shooting. That's new.", "an Oathkeeper envoy in vestment white with a lamp, standing still at a grate, slimos hiding"),
      ("An envoy of the {order}. Says they can keep the Gangs and the Badges off us.", "the envoy's helmeted face, a cross of lamps on the visor"),
      ("All it costs is a tithe: a tenth of our junk up the grate, every shift.", "a crate of junk chrome being hauled up through a grate on a rope"),
      ("Accept, and we get two quiet shifts. Refuse, and we find out how they say no.", "Ivan between the envoy's lamp and the Heart's glow, choosing")],
     [("They took the tithe, drew the map, and came back with bleach. Write it down somewhere.", "lamp carriers and flamers pouring down a grate, bleach in the water"),
      ("Now they know the way to the Heart, and they'll never forget it.", "a map on an Oathkeeper data slate with the Heart circled"),
      ("We buried the burned and counted the living. More of the second than they planned.", "slimos carrying a burned body past a smoking Pod"),
      ("No more envoys. Next time they come, they come for all of it.", "the grate empty, a broken lamp on the floor")],
     [("They asked once, nicely. Then they came with bleach.", "lamp carriers and flamers pouring down a grate, bleach in the water"),
      ("We said no, and the white light said it back louder.", "a wall of lamps advancing through a culvert, cords smoking"),
      ("We buried the burned and counted the living. More of the second than they planned.", "slimos carrying a burned body past a smoking Pod"),
      ("No more envoys. Next time they come, they come for all of it.", "the grate empty, a broken lamp on the floor")]),
    (4, "sewerats", "Sewerats", "chapter 3 closed and the Storm Drains reached", "the chapter's Bloom ended, or two of its crawlers tamed", "END THE BLOOM", "a chapter-1 newcomer gone feral",
     [("Something's growing in the Storm Drains that isn't ours.", "a sickly green-purple growth spreading across a wide storm drain, water rushing"),
      ("A strain cut off from any Heart: feral, hungry, and coming this way.", "Pond Scum tissue choking a drain pipe, eyes in the dark"),
      ("{ferals}: the lost ones. They still have faces in there somewhere.", "sewerats crawling, one with a human face half-dissolved"),
      ("One of them is {name}. We never reached the drain where they fell.", "a slimo's tag on a wall, half eaten by feral growth")],
     [("The Bloom is done. What it ate, it doesn't give back.", "dead feral tissue grey and collapsing in the drain"),
      ("The ones we tamed sleep in the Pens now. Don't ask them about before.", "tamed sewerats curled in a wire Pen, a Tend slimo watching"),
      ("{name} is on the wall. Spell it right.", "a memorial wall of scratched names lit by ooze"),
      ("Up above, the {order} are counting us. Let them count.", "an Oathkeeper tally board with marks, seen through a grate")],
     None),
    (5, "warchest", "Warchest", "chapter 4 closed and the Service Tunnels reached", "a Runner home from the Vault with a Rare or better, and a Downer Still standing", "THE VAULT AND A STILL", "what Arasaka left behind",
     [("Under the Service Tunnels there's a vault {fallen_corp} forgot to flood all the way.", "a drowned corporate archive, server racks half under black water, emergency lights"),
      ("Corpo chrome, still in the crates. Corpo files, still on the drives.", "a sealed corporate crate cracked open, chrome gleaming inside"),
      ("Everything the Moon wants is down there, for the first one who walks out with it.", "a Runner with a lantern wading into the archive"),
      ("Bring me a Runner home from the Vault, and a Still that turns slime into something they'll beg for.", "a Downer Still bubbling, copper coils and ooze")],
     [("{teller} worked down there once. Says the elevators still say good morning.", "a slimo in a torn corporate jacket at a dead elevator panel"),
      ("{fallen_corp} built a war chest for a war it never got to fight.", "rows of corporate weapons crates in the dark"),
      ("Now it's ours. Every crate, every drive, every bad idea.", "slimos hauling crates along a Mat cord"),
      ("The Still is running. Downer by the drum.", "drums of Downer stacked by the Still, steam rising")],
     None),
    (6, "common_ground", "Common Ground", "chapter 5 closed and the Vault reached", "an alliance signed and the Undercroft reached", "SIGN AN ALLIANCE", "the factions that hunted slimos asking for Downer",
     [("The {order} took {city_above} last night. Curfew lamps on every corner.", "an occupied neon street, white lamps on every corner, people hiding"),
      ("Food paste is half what it was. The streets are scared and hungry.", "an empty food stall shutter, a queue of hungry faces"),
      ("And now the people who hunted us come down the grates, asking for Downer.", "gang members, nomads and suits climbing down a grate, cautious"),
      ("Pick one to stand with. Their enemies become ours.", "Ivan at the head of a scrap table, envoys on both sides")],
     [("We shook hands in the Undercroft, under their feet.", "a handshake between a slimo and a human under a metro platform"),
      ("Gangs, Nomads, suits, Badges: everybody's one of the {people} when the white light comes.", "humans and slimos side by side in the dark"),
      ("This is where {city_above} and {city_below} find common ground.", "a grate with neon above and ooze below, light meeting light"),
      ("Now we get ready.", "slimos and allies loading weapons by the Heart")],
     None),
    (7, "rise", "Rise", "chapter 6 closed", "the Great Cleansing gone", "THE GREAT CLEANSING", "the ending the genome earned",
     [("They're coming, all of them, for the Great Cleansing.", "the night sky over the city full of descending lunar ships with white lamps"),
      ("Every lamp the Moon owns, down every grate the Undercroft has.", "white light pouring down three grates at once"),
      ("Six shifts. Everything we built, everything we bred, everything we kept.", "the whole city on the Stack view, cords glowing, a countdown"),
      ("Let them come. We've been waiting under their feet for ten years.", "Ivan at the Heart, arms crossed, the Mat rising behind him")],
     [("The lamps are out. The squads are down, or gone back up the grates.", "broken lamps in bleached water, silence"),
      ("Listen. That's the Heart.", "the Heart pulsing in the dark, slimos gathering"),
      ("Whatever we are now, we earned it.", "Ivan's face in close-up, tired, grinning"),
      ("Here's what we became.", "the camera pulling back from the Heart into darkness")],
     None),
]

ENDINGS = [
    ("garden", "Garden", ["lume", "thrift"], 0.05,
     [("The Mat lit up under {city_above} like a second skyline.", "the cross-section city glowing purple and cyan under neon streets"),
      ("Clean water, cold light, a cure that doesn't take your chrome. All ours, and all for sale.", "slimos handing up bottled water and glowing bulbs through grates"),
      ("The grates stay open. They need us now.", "an open grate with humans and slimos trading in the light"),
      ("Welcome to {city_below}. Mind the glow.", "a neon sign made of ooze reading nothing, glowing in a culvert")]),
    ("hive", "Hive", ["chrome", "conductance"], 0.0,
     [("Every cord's a wire, every wire's a wall.", "chrome-armoured cords crossing a tunnel like rebar"),
      ("There's an armoured Mat under the whole of {city_above}, and it remembers every lamp.", "a cross-section of the city with a metal-grey Mat under every street"),
      ("The war chest isn't a stockpile anymore. It's the ground they stand on.", "Oathkeeper boots on a street, the Mat's armour beneath"),
      ("Knock first.", "a chrome-plated grate, sealed, with a single purple light behind it")]),
    ("world_eater", "World-Eater", ["appetite", "feral"], -0.05,
     [("We were hungry. We were always going to be hungry.", "the Mat bursting through a street, cars sinking into ooze"),
      ("The Mat came up through the streets and ate the white light, and then it ate the streets.", "lamps swallowed by tissue, neon signs sinking"),
      ("The {order} lost. So did everybody else.", "an empty city half-dissolved under a purple sky"),
      ("I'm still talking. Somebody has to.", "Ivan alone on a heap of ooze and wreckage")]),
    ("accord", "Accord", ["thrift", "toxicity"], 0.0,
     [("They can't cleanse it and they can't get near it. Every lamp that tried came back sick.", "Oathkeepers retreating from a toxic glowing Mat, masks cracked"),
      ("So the {order} did something they've never done: they signed.", "an Oathkeeper hand signing a treaty on a scrap table"),
      ("{city_above} gets its water and its light. {city_below} gets its border.", "a painted line across a tunnel, slimos on one side, humans on the other"),
      ("First treaty in the history of the Moon. Signed in slime.", "the treaty sheet stained with purple ooze")]),
    ("scattered", "Scattered", [], 0.0,
     [("The Heart's gone, and the signal with it.", "the burned Heart, black and cracked, flames dying"),
      ("What's left of the Mat went feral in the drains, or went white in the light.", "feral tissue and bleached cords in the dark"),
      ("But the strain's in my veins, and I'm still breathing in a sump nobody's found yet.", "Ivan crawling into a new, darker sump"),
      ("Feed me something. We start again.", "a tiny new Heart pulsing in black water")]),
]


def q(s):
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'


def panels_toml():
    out = ["# Cyberpunk: ROSE — chapter and ending panels (Full Design v3.19, Narrative and Ivan: Panels). Written by",
           "# tools/gen/gen_panels.py once; this file is the art's source of truth. 1280 x 720, hero art at twice native.", ""]
    def slot(sid, tag, scene):
        out.extend(["[[slot]]", f"id = {q(sid)}", 'kind = "panel"', "size = [1280, 720]", "frames = 1", 'states = [""]',
                    'ramp = "noir"', 'silhouette = "frame"', f"tag = {q(tag)}", 'anchor = "anchor.tile"',
                    f"prompt = {q(scene + ', ' + STYLE)}", f"negative = {q(NEG)}", 'status = "placeholder"', ""])
    for (n, cid, name, opens, goal, card, beat, op, cl, clr) in CHAPTERS:
        for k, (_, scene) in enumerate(op):
            slot(f"panel.ch{n}.open.{k+1}", f"{n}O{k+1}", f"chapter {n} {name}, opening panel {k+1}: {scene}")
        for k, (_, scene) in enumerate(cl):
            slot(f"panel.ch{n}.close.{k+1}", f"{n}C{k+1}", f"chapter {n} {name}, closing panel {k+1}: {scene}")
    for (eid, name, genes, ally, ps) in ENDINGS:
        for k, (_, scene) in enumerate(ps):
            slot(f"panel.end.{eid}.{k+1}", f"E{eid[0].upper()}{k+1}", f"the {name} ending, panel {k+1}: {scene}")
    return "\n".join(out)


def chapters_toml():
    out = ["# Cyberpunk: ROSE — the campaign's chapters and endings (Full Design v3.19, Narrative and Ivan). Written by",
           "# tools/gen/gen_panels.py once; this file is the source of truth for captions. A panel's art is its slot in",
           "# game/assets/manifest/panels.toml. Captions name the world through the string table's tokens; {name} is the",
           "# chapter-4 newcomer, {teller} the chapter-5 Corpo-Past slimo (or \"a Runner\").", ""]
    for (n, cid, name, opens, goal, card, beat, op, cl, clr) in CHAPTERS:
        out.extend(["[[chapter]]", f"n = {n}", f"id = {q(cid)}", f"name = {q(name)}", f"opens = {q(opens)}",
                    f"goal = {q(goal)}", f"card = {q(card)}", f"beat = {q(beat)}"])
        out.append("open = [")
        for k, (cap, _) in enumerate(op):
            out.append(f"  {{ slot = {q(f'panel.ch{n}.open.{k+1}')}, caption = {q(cap)} }},")
        out.append("]")
        out.append("close = [")
        for k, (cap, _) in enumerate(cl):
            out.append(f"  {{ slot = {q(f'panel.ch{n}.close.{k+1}')}, caption = {q(cap)} }},")
        out.append("]")
        if clr:
            out.append("close_refused = [")
            for k, (cap, _) in enumerate(clr):
                out.append(f"  {{ slot = {q(f'panel.ch{n}.close.{k+1}')}, caption = {q(cap)} }},")
            out.append("]")
        out.append("")
    for (eid, name, genes, ally, ps) in ENDINGS:
        out.extend(["[[ending]]", f"id = {q(eid)}", f"name = {q(name)}", "genes = [" + ", ".join(q(g) for g in genes) + "]",
                    f"ally = {ally}"])
        out.append("panels = [")
        for k, (cap, _) in enumerate(ps):
            out.append(f"  {{ slot = {q(f'panel.end.{eid}.{k+1}')}, caption = {q(cap)} }},")
        out.append("]")
        out.append("")
    return "\n".join(out)


if __name__ == "__main__":
    with open(os.path.join(ROOT, "data", "chapters.toml"), "w") as f:
        f.write(chapters_toml())
    with open(os.path.join(ROOT, "game", "assets", "manifest", "panels.toml"), "w") as f:
        f.write(panels_toml())
    print("wrote data/chapters.toml and game/assets/manifest/panels.toml")
