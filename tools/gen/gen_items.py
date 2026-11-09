"""Authoring script for data/items.toml. Run once; the TOML is the source of truth afterwards.
Every item: id, name, category, tier, size (w,h), weight kg, base value eddies, feed nutrient + gene bias,
craft tags, optional set, optional gear block, one codex line. 164 items across 9 categories."""
import sys

GENES = ["vigor", "branching", "thrift", "appetite", "lume", "toxicity", "chrome", "feral", "conductance", "plasticity"]
TIER_FEED = {"common": 0.18, "uncommon": 0.30, "rare": 0.45, "blackline": 0.60, "relic": 1.00}
TIER_VALUE = {"common": 4, "uncommon": 15, "rare": 60, "blackline": 220, "relic": 0}

items = []


def item(id, name, cat, tier, size, weight, value=None, feed=None, bias=None, tags=(), set=None, gear=None, codex=""):
    items.append(dict(id=id, name=name, category=cat, tier=tier, size=size, weight=weight,
                      value=TIER_VALUE[tier] if value is None else value,
                      feed=(TIER_FEED[tier] if cat in ("chrome", "relic") else 0.0) if feed is None else feed,
                      bias=bias or {}, tags=list(tags), set=set, gear=gear, codex=codex))


# ---------------------------------------------------------------- CHROME (38): the Mat's food, Oathkeeper-liquefied implants and working cyberware
chrome_common = [
    ("chr_optic_slag", "Optic Slag", "a fused cheap eye; the lens survived, the brain did not", ["optic"], {"lume": 0.3}),
    ("chr_jaw_plate", "Jaw Plate", "street dental chrome, molars still in it", ["housing"], {"thrift": 0.2}),
    ("chr_finger_servos", "Finger Servos", "five of them, one still twitching", ["servo"], {"branching": 0.3}),
    ("chr_neural_tap", "Dead Neural Tap", "the port that took the vaccine first", ["conductor"], {"conductance": 0.2}),
    ("chr_spine_bolt", "Spine Bolt", "a single vertebral anchor, re-organified around the thread", ["housing"], {"thrift": 0.3}),
    ("chr_subdermal_mesh", "Subdermal Mesh", "a hand's worth of armour weave gone soft", ["weave"], {"chrome": 0.2}),
    ("chr_vox_box", "Vox Box", "a voice modulator stuck on one tone", ["conductor"], {"lume": 0.2}),
    ("chr_knee_actuator", "Knee Actuator", "runner's chrome; the gel inside has gone purple", ["servo"], {"vigor": 0.3}),
    ("chr_ear_coil", "Ear Coil", "audio implant, cheap, still warm", ["conductor"], {"branching": 0.2}),
    ("chr_hand_shunt", "Hand Shunt", "a power pass-through for cheap arm mods", ["conductor"], {"conductance": 0.3}),
    ("chr_skin_wire", "Skin Wire", "a metre of under-skin cable", ["conductor"], {"conductance": 0.2}),
    ("chr_pain_editor", "Burnt Pain Editor", "the chip that let a chromehead keep going", ["board"], {"appetite": 0.3}),
    ("chr_liver_sieve", "Liver Sieve", "a filter organ with a corporate logo on it", ["filter"], {"thrift": 0.4}),
    ("chr_toe_claws", "Toe Claws", "climbing chrome, blunted", ["blade"], {"vigor": 0.2}),
]
chrome_uncommon = [
    ("chr_street_eye", "Street Eye", "working low-light optic, scratched", ["optic"], {"lume": 0.5}),
    ("chr_arm_frame", "Arm Frame", "a forearm chassis, intact", ["housing", "servo"], {"vigor": 0.4, "chrome": 0.2}),
    ("chr_reflex_chip", "Reflex Chip", "clocked for bar fights", ["board"], {"vigor": 0.5}),
    ("chr_gill_plate", "Gill Plate", "a sewer-diver's lung mod", ["filter"], {"thrift": 0.5}),
    ("chr_sync_jack", "Sync Jack", "a clean interface port", ["conductor", "board"], {"conductance": 0.5}),
    ("chr_heat_sink", "Heat Sink Rib", "a rib replaced with a radiator", ["housing"], {"thrift": 0.3, "chrome": 0.3}),
    ("chr_pulse_cell", "Pulse Cell", "an implant battery, half charge", ["cell"], {"lume": 0.4, "conductance": 0.2}),
    ("chr_lamp_eye", "Lamp Eye", "an optic that throws light", ["optic", "cell"], {"lume": 0.6}),
    ("chr_grip_palm", "Grip Palm", "a loader's hand, magnetic", ["servo"], {"chrome": 0.4}),
    ("chr_balance_gyro", "Balance Gyro", "inner-ear chrome, still spinning", ["servo", "board"], {"branching": 0.5}),
    ("chr_vein_pump", "Vein Pump", "a secondary heart for stims", ["pump"], {"appetite": 0.5}),
    ("chr_shock_cuff", "Shock Cuff", "a defensive wrist discharge", ["conductor", "cell"], {"toxicity": 0.4}),
]
chrome_rare = [
    ("chr_corpo_eye", "Corpo Eye", "Arasaka-grade optic with the serial filed", ["optic", "board"], {"lume": 0.5, "conductance": 0.4}),
    ("chr_lattice_spine", "Lattice Spine", "full spinal replacement, titanium lattice", ["housing", "conductor"], {"chrome": 0.6, "conductance": 0.3}),
    ("chr_cortex_shunt", "Cortex Shunt", "the thing that made an executive fast", ["board", "conductor"], {"conductance": 0.7}),
    ("chr_black_lung", "Black Lung", "a filter organ built for toxic floors", ["filter", "pump"], {"thrift": 0.6, "toxicity": 0.3}),
    ("chr_mono_tendons", "Mono Tendons", "monowire tendon set, a full leg", ["servo", "blade"], {"vigor": 0.7}),
    ("chr_armour_dermis", "Armour Dermis", "a torso of woven plate under skin", ["weave", "housing"], {"chrome": 0.7}),
    ("chr_vault_jack", "Vault Jack", "an access implant keyed to a dead building", ["board"], {"conductance": 0.4, "plasticity": 0.3}),
]
chrome_black = [
    ("chr_lamp_core", "Lamp Core", "the white-light emitter from a carrier's pole", ["cell", "optic"], {"toxicity": 0.5, "lume": 0.4}),
    ("chr_sanitation_glove", "Sanitation Glove", "a flamer's trigger hand, mil-tech servos", ["servo", "housing"], {"appetite": 0.6, "chrome": 0.3}),
    ("chr_moon_board", "Moon Board", "lunar-fab control board; the traces are gold", ["board", "conductor"], {"conductance": 0.6, "feral": 0.3}),
    ("chr_bleach_pump", "Bleach Pump", "the sprayer's heart", ["pump"], {"toxicity": 0.7}),
    ("chr_vestment_weave", "Vestment Weave", "ballistic cloth with a prayer printed in the fibre", ["weave"], {"feral": 0.5, "chrome": 0.4}),
]
for id, name, codex, tags, bias in chrome_common:
    item(id, name, "chrome", "common", (1, 1), 0.6, tags=tags, bias=bias, codex=codex)
for id, name, codex, tags, bias in chrome_uncommon:
    item(id, name, "chrome", "uncommon", (1, 2) if "housing" in tags else (1, 1), 1.2, tags=tags, bias=bias, codex=codex)
for id, name, codex, tags, bias in chrome_rare:
    item(id, name, "chrome", "rare", (2, 2) if "housing" in tags else (1, 2), 2.4, tags=tags, bias=bias, codex=codex)
for id, name, codex, tags, bias in chrome_black:
    item(id, name, "chrome", "blackline", (2, 2), 3.0, tags=tags, bias=bias, codex=codex)

# ---------------------------------------------------------------- PARTS (24): low sell, high craft
parts = [
    ("prt_pipe", "Pipe Length", "common", (1, 3), 2.0, 2, ["housing"], "half a metre of galvanised nothing"),
    ("prt_wire", "Wire Spool", "common", (1, 1), 0.5, 2, ["conductor"], "copper, mostly"),
    ("prt_lens", "Cracked Lens", "common", (1, 1), 0.2, 3, ["optic"], "fogged but it bends light"),
    ("prt_bearing", "Bearing Set", "common", (1, 1), 0.4, 3, ["servo"], "six balls, one race"),
    ("prt_cloth", "Filter Cloth", "common", (1, 1), 0.2, 2, ["filter"], "sewer-grade"),
    ("prt_cell_dead", "Dead Cell", "common", (1, 1), 0.6, 2, ["cell"], "takes a charge, leaks it by morning"),
    ("prt_board_fried", "Fried Board", "common", (1, 1), 0.2, 3, ["board"], "three traces still live"),
    ("prt_blade_stock", "Blade Stock", "common", (1, 2), 0.8, 3, ["blade"], "a strip of spring steel"),
    ("prt_tarp", "Tarp", "common", (2, 2), 1.0, 2, ["weave"], "keeps the drip off"),
    ("prt_valve", "Valve", "common", (1, 1), 0.7, 3, ["pump"], "it turns"),
    ("prt_servo", "Servo Motor", "uncommon", (1, 1), 0.6, 8, ["servo"], "stepper, torque intact"),
    ("prt_lens_good", "Clear Lens", "uncommon", (1, 1), 0.2, 9, ["optic"], "from a dead camera"),
    ("prt_cell", "Live Cell", "uncommon", (1, 1), 0.6, 10, ["cell"], "holds a charge for a shift"),
    ("prt_board", "Logic Board", "uncommon", (1, 1), 0.2, 12, ["board"], "boots"),
    ("prt_pump", "Pump Head", "uncommon", (2, 1), 1.4, 10, ["pump"], "moves brine"),
    ("prt_plate", "Welded Plate", "uncommon", (2, 2), 3.0, 9, ["housing", "weave"], "someone's door"),
    ("prt_enzyme_jar", "Enzyme Jar", "uncommon", (1, 1), 0.5, 11, ["enzyme"], "Mat spit, sealed"),
    ("prt_salt", "Salt Bag", "uncommon", (1, 2), 2.5, 6, ["salt"], "road salt; eight tiles of no"),
    ("prt_gold_trace", "Gold Trace", "rare", (1, 1), 0.1, 40, ["conductor"], "Moon-grade conductor"),
    ("prt_mil_servo", "Mil Servo", "rare", (1, 1), 0.8, 45, ["servo"], "quiet and strong"),
    ("prt_optic_array", "Optic Array", "rare", (2, 1), 0.5, 50, ["optic", "board"], "sees in the dark"),
    ("prt_reactor_cell", "Reactor Cell", "rare", (1, 2), 1.5, 55, ["cell"], "warm to the touch, always"),
    ("prt_spore_tissue", "Spore Tissue", "rare", (1, 1), 0.3, 30, ["enzyme", "weave"], "cut from a Sac; it remembers shapes"),
    ("prt_relic_shard", "Relic Shard", "blackline", (1, 1), 0.4, 0, ["conductor", "enzyme"], "a splinter of lunar stone; the Mat wants it"),
]
for id, name, tier, size, w, v, tags, codex in parts:
    item(id, name, "part", tier, size, w, value=v, tags=tags, codex=codex)

# ---------------------------------------------------------------- ORGANICS (14)
organics = [
    ("org_paste", "Food Paste", "common", (1, 1), 0.4, 2, [], "a shift of not drawing on the Mat"),
    ("org_jerky", "Rat Jerky", "common", (1, 1), 0.2, 2, [], "sewerat, cured"),
    ("org_water", "Clean Water", "common", (1, 1), 1.0, 2, [], "a Lung's day of work"),
    ("org_mould", "Mould Culture", "common", (1, 1), 0.1, 1, ["enzyme"], "grows on anything"),
    ("org_antiseptic", "Antiseptic", "uncommon", (1, 1), 0.2, 8, [], "stops a bleed"),
    ("org_splint", "Splint Kit", "uncommon", (1, 2), 0.5, 7, [], "sets a fracture"),
    ("org_stim", "Stim", "uncommon", (1, 1), 0.1, 14, [], "one shift of stamina, one shift of debt"),
    ("org_downer", "Downer Dose", "rare", (1, 1), 0.1, 45, [], "the cure that works; the Oathkeepers' does not"),
    ("org_brine", "Lung Brine", "common", (1, 2), 1.2, 3, ["enzyme"], "what a Lung sweats"),
    ("org_fever_tonic", "Fever Tonic", "uncommon", (1, 1), 0.2, 12, [], "clears Fungal Fever"),
    ("org_ooze_sample", "Ooze Sample", "uncommon", (1, 1), 0.3, 9, ["enzyme"], "your own strain in a jar"),
    ("org_lume_jelly", "Lume Jelly", "uncommon", (1, 1), 0.2, 10, ["cell"], "glows for a day"),
    ("org_corpse_tag", "Corpse Tag", "common", (1, 1), 0.0, 0, [], "a name and a shift"),
    ("org_spore_cap", "Spore Cap", "rare", (1, 1), 0.2, 25, ["enzyme"], "tames a sewerat if it eats it"),
]
for id, name, tier, size, w, v, tags, codex in organics:
    item(id, name, "organic", tier, size, w, value=v, tags=tags, codex=codex)

# ---------------------------------------------------------------- TECH (16)
tech = [
    ("tec_shard_street", "Street Shard", "common", (1, 1), 0.1, 6, "gang gossip and a map of two blocks"),
    ("tec_shard_corpo", "Corpo Shard", "rare", (1, 1), 0.1, 70, "payroll, a floor plan, a password"),
    ("tec_shard_oath", "Oathkeeper Shard", "blackline", (1, 1), 0.1, 200, "patrol schedules in a dead language"),
    ("tec_blueprint_chrome1", "Blueprint: Chrome I", "uncommon", (2, 1), 0.2, 30, "scrap forge tolerances"),
    ("tec_blueprint_chrome2", "Blueprint: Chrome II", "rare", (2, 1), 0.2, 90, "vault doors and gate housings"),
    ("tec_blueprint_chrome3", "Blueprint: Chrome III", "blackline", (2, 1), 0.2, 260, "lamp capture"),
    ("tec_blueprint_flesh1", "Blueprint: Flesh I", "uncommon", (2, 1), 0.2, 30, "pod ventilation"),
    ("tec_blueprint_flesh2", "Blueprint: Flesh II", "rare", (2, 1), 0.2, 90, "the Still"),
    ("tec_blueprint_flesh3", "Blueprint: Flesh III", "blackline", (2, 1), 0.2, 260, "shepherd training"),
    ("tec_blueprint_myc1", "Blueprint: Mycelium I", "uncommon", (2, 1), 0.2, 30, "pits and bait"),
    ("tec_blueprint_myc2", "Blueprint: Mycelium II", "rare", (2, 1), 0.2, 90, "quotas and housings"),
    ("tec_blueprint_myc3", "Blueprint: Mycelium III", "blackline", (2, 1), 0.2, 260, "nerve logic"),
    ("tec_map_s2", "Culvert Survey", "uncommon", (2, 1), 0.1, 20, "reveals a Culvert route"),
    ("tec_map_s4", "Service Schematic", "rare", (2, 1), 0.1, 60, "reveals a Service Tunnel route"),
    ("tec_comms", "Night City Comms", "rare", (1, 2), 0.8, 80, "hears the street; Badges pay for it"),
    ("tec_recorder", "Chaplain's Recorder", "blackline", (1, 1), 0.3, 150, "sermons, and the frequency of the lamps"),
]
for id, name, tier, size, w, v, codex in tech:
    item(id, name, "tech", tier, size, w, value=v, tags=["board"] if "shard" in id or "comms" in id or "recorder" in id else [], codex=codex)

# ---------------------------------------------------------------- GEAR (31)
# gear = dict(slot, kind, dmg, armour, noise, light, tool)
gear = [
    # melee 6
    ("ger_pipe", "Lead Pipe", "common", (1, 3), 2.5, 6, dict(slot="hand", kind="melee", dmg=8, noise=3), "swings slow, lands heavy"),
    ("ger_shiv", "Chrome Shiv", "common", (1, 1), 0.3, 5, dict(slot="hand", kind="melee", dmg=5, noise=1), "quiet"),
    ("ger_fist", "Chrome Fist", "uncommon", (1, 1), 1.5, 22, dict(slot="hand", kind="melee", dmg=10, noise=2), "a loader's hand on a strap"),
    ("ger_sword", "Scrap Sword", "uncommon", (1, 3), 1.8, 28, dict(slot="hand", kind="melee", dmg=13, noise=2), "blade stock, housing, a conductor for the hum"),
    ("ger_monowire", "Monowire", "rare", (1, 1), 0.2, 110, dict(slot="hand", kind="melee", dmg=18, noise=1), "do not touch the line"),
    ("ger_lamp_pole", "Captured Lamp Pole", "blackline", (1, 4), 4.0, 0, dict(slot="hand", kind="melee", dmg=9, noise=2, light=8), "white light on a stick; ferals hate it, so does the Mat"),
    # ranged 7
    ("ger_zip", "Zip Gun", "common", (1, 2), 1.0, 10, dict(slot="hand", kind="ranged", dmg=12, noise=6, mag=1), "one shot, then a prayer"),
    ("ger_scrap_pistol", "Scrap Pistol", "uncommon", (1, 2), 1.2, 35, dict(slot="hand", kind="ranged", dmg=11, noise=5, mag=6), "jams below condition 20"),
    ("ger_nail_rifle", "Nail Rifle", "uncommon", (1, 4), 3.5, 48, dict(slot="hand", kind="ranged", dmg=16, noise=4, mag=8), "built from a framing gun"),
    ("ger_flechette", "Flechette Pipe", "rare", (1, 3), 3.0, 95, dict(slot="hand", kind="ranged", dmg=22, noise=6, mag=2), "spreads; shreds Bursters"),
    ("ger_shock_lance", "Shock Lance", "rare", (1, 4), 3.2, 120, dict(slot="hand", kind="ranged", dmg=14, noise=3, mag=4), "stuns a lamp carrier"),
    ("ger_sanitation_rifle", "Sanitation Rifle", "blackline", (2, 4), 4.5, 380, dict(slot="hand", kind="ranged", dmg=28, noise=5, mag=12), "theirs; it still burns cords"),
    ("ger_bleach_sprayer", "Bleach Sprayer", "blackline", (2, 3), 5.0, 300, dict(slot="hand", kind="ranged", dmg=6, noise=2, mag=20), "kills a Bloom; kills your own Mat too"),
    # armour 6
    ("ger_tarp_coat", "Tarp Coat", "common", (2, 2), 1.5, 8, dict(slot="body", kind="armour", armour=2, noise=0), "dry, at least"),
    ("ger_plate_vest", "Plate Vest", "uncommon", (2, 2), 6.0, 40, dict(slot="body", kind="armour", armour=6, noise=1), "a door, worn"),
    ("ger_dermis_wrap", "Dermis Wrap", "rare", (2, 2), 2.0, 130, dict(slot="body", kind="armour", armour=8, noise=0), "armour dermis sewn into a jacket"),
    ("ger_vestment", "Stolen Vestment", "blackline", (2, 2), 1.8, 0, dict(slot="body", kind="armour", armour=7, noise=0), "scouts hesitate for one second"),
    ("ger_helm", "Welding Helm", "uncommon", (1, 1), 1.2, 18, dict(slot="head", kind="armour", armour=3, light=-1), "shields the eyes from white light"),
    ("ger_waders", "Waders", "common", (1, 2), 1.5, 9, dict(slot="feet", kind="armour", armour=1, noise=-1), "water tiles do not soak you"),
    # tools 6
    ("ger_lantern", "Lantern", "common", (1, 1), 0.8, 7, dict(slot="rig", kind="tool", light=6, noise=0), "see six tiles, be seen from nine"),
    ("ger_dig_claw", "Dig Claw", "uncommon", (1, 2), 1.5, 24, dict(slot="hand", kind="tool", noise=4), "deposits in half the time"),
    ("ger_pry_bar", "Pry Bar", "common", (1, 3), 2.0, 6, dict(slot="hand", kind="tool", noise=3), "lockers open; so do skulls, slowly"),
    ("ger_lure", "Rat Lure", "uncommon", (1, 1), 0.3, 15, dict(slot="rig", kind="tool", noise=5), "throws noise elsewhere"),
    ("ger_repair_kit", "Repair Kit", "uncommon", (2, 1), 1.0, 20, dict(slot="rig", kind="tool"), "+30 condition, once"),
    ("ger_backpack_2", "Tweaker's Pack", "uncommon", (2, 2), 1.0, 35, dict(slot="back", kind="pack", grid="8x5"), "two more rows"),
    # implants 6
    ("imp_night_eye", "Night Eye", "rare", (1, 1), 0.1, 140, dict(slot="implant", kind="implant", stat="eye", bonus=2, weakness="white light blinds for 3 s"), "fog radius +2"),
    ("imp_tendon_set", "Tendon Set", "rare", (1, 1), 0.3, 150, dict(slot="implant", kind="implant", stat="nerve", bonus=2, weakness="fractures take twice as long"), "sprint longer"),
    ("imp_loader_arm", "Loader Arm", "rare", (1, 2), 2.0, 160, dict(slot="implant", kind="implant", stat="body", bonus=2, weakness="noise +1"), "carry 8 kg more"),
    ("imp_appraiser", "Appraiser's Lens", "rare", (1, 1), 0.1, 170, dict(slot="implant", kind="implant", stat="eye", bonus=1, weakness="Lit need rises faster"), "identifies Uncommon on sight"),
    ("imp_mat_ear", "Mat Ear", "rare", (1, 1), 0.1, 130, dict(slot="implant", kind="implant", stat="matsense", bonus=2, weakness="Mood falls off the Mat"), "hear the signal"),
    ("imp_lamp_core", "Lamp-Core Socket", "blackline", (1, 1), 0.4, 0, dict(slot="implant", kind="implant", stat="eye", bonus=3, weakness="the Mat will not carry you; Toxicity hurts you"), "white light in your own head"),
]
for id, name, tier, size, w, v, g, codex in gear:
    item(id, name, "gear", tier, size, w, value=v, gear=g, codex=codex)

# ---------------------------------------------------------------- CONTRABAND (6)
contraband = [
    ("con_downer_crate", "Downer Crate", "rare", (2, 2), 4.0, 400, ["enzyme"], "twelve doses; every faction wants it; the Moon knows"),
    ("con_cold_light", "Cold Light Cell", "uncommon", (1, 1), 0.3, 18, ["cell"], "a Bulb's week in a jar; sells topside as decor"),
    ("con_filtered_water", "Filtered Water Drum", "uncommon", (2, 2), 10.0, 30, [], "Nomads carry it up the drains"),
    ("con_enzyme_vial", "Ooze Enzyme Vial", "rare", (1, 1), 0.2, 60, ["enzyme"], "the Still's input; Corpo chemists bid"),
    ("con_vestment_cloth", "Vestment Cloth", "blackline", (2, 1), 0.5, 180, ["weave"], "gangs wear it as a joke; the joke raises Attention"),
    ("con_comms_rig", "Night City Comms Rig", "rare", (2, 2), 3.0, 140, ["board", "conductor"], "hears the whole street"),
]
for id, name, tier, size, w, v, tags, codex in contraband:
    item(id, name, "contraband", tier, size, w, value=v, tags=tags, codex=codex)

# ---------------------------------------------------------------- KEYS (8)
keys = [
    ("key_grate_sump", "Sump Grate Key", "uncommon", (1, 1), 0.2, 0, "opens the two Sump grates"),
    ("key_grate_culvert", "Culvert Grate Key", "uncommon", (1, 1), 0.2, 0, "opens the Culvert grates"),
    ("key_grate_drain", "Drain Grate Key", "rare", (1, 1), 0.2, 0, "opens the Storm Drain grates"),
    ("key_grate_service", "Service Grate Key", "rare", (1, 1), 0.2, 0, "opens the Service Tunnel grates"),
    ("key_vault_card", "Vault Card", "rare", (1, 1), 0.0, 0, "one Vault door, once"),
    ("key_vault_master", "Vault Master Card", "blackline", (1, 1), 0.0, 0, "every Vault door, until a Corpo contact notices"),
    ("key_lamp_badge", "Lamp-Carrier Badge", "blackline", (1, 1), 0.1, 0, "one Runner passes one Patrol, once"),
    ("key_undercroft", "Undercroft Pass", "blackline", (1, 1), 0.1, 0, "the metro gate under Night City"),
]
for id, name, tier, size, w, v, codex in keys:
    item(id, name, "key", tier, size, w, value=v, codex=codex)

# ---------------------------------------------------------------- CURIOS (22), seven sets of three plus one loose
curios = [
    ("cur_street_sign", "Pre-Vaccine Street Sign", "uncommon", (3, 1), 2.0, 12, "street", "a block that no longer exists"),
    ("cur_ooze_toy", "Child's Ooze Toy", "uncommon", (1, 1), 0.2, 15, "street", "a jar of purple; it was a toy once"),
    ("cur_arcade_token", "Arcade Token", "uncommon", (1, 1), 0.0, 8, "street", "a game nobody can play"),
    ("cur_nameplate", "Corpo Nameplate", "rare", (2, 1), 0.5, 40, "arasaka", "a desk, a floor, a tower, all gone"),
    ("cur_lanyard", "Executive Lanyard", "rare", (1, 1), 0.1, 35, "arasaka", "the photo has been scraped off"),
    ("cur_tower_model", "Tower Desk Model", "rare", (2, 2), 1.0, 70, "arasaka", "the building, at 1:2000, before the head came off"),
    ("cur_rosary", "Lunar Stone Rosary", "blackline", (1, 1), 0.2, 120, "reliquary", "fifty-nine beads of holy rock"),
    ("cur_hymnal", "Oathkeeper Hymnal", "blackline", (1, 2), 0.6, 90, "reliquary", "the words are kind; the margins are not"),
    ("cur_vial_moon", "Vial of Moon Dust", "blackline", (1, 1), 0.1, 150, "reliquary", "regolith from the holy rock"),
    ("cur_rat_skull", "Sewerat Skull", "uncommon", (1, 1), 0.3, 10, "feral", "the teeth kept growing"),
    ("cur_burster_husk", "Burster Husk", "uncommon", (2, 1), 0.4, 12, "feral", "dry, and still faintly purple"),
    ("cur_maw_tooth", "Maw Tooth", "rare", (1, 2), 0.8, 45, "feral", "the Maw that held a corridor for a week"),
    ("cur_nomad_patch", "Nomad Patch", "uncommon", (1, 1), 0.0, 9, "road", "a clan that moves goods down drains"),
    ("cur_badge", "Retired Badge", "uncommon", (1, 1), 0.1, 14, "road", "NCPD; the officer is a slimo now"),
    ("cur_gang_colours", "Gang Colours", "uncommon", (1, 1), 0.1, 9, "road", "a jacket back; the crew is mostly dead"),
    ("cur_first_cord", "First Cord", "rare", (1, 1), 0.1, 0, "heart", "a dried length of your own first strand"),
    ("cur_heart_shard", "Heart Shard", "rare", (1, 1), 0.2, 0, "heart", "a bulb that calcified when the Heart was cut"),
    ("cur_ivan_fragment", "Ivan's Fragment", "rare", (1, 1), 0.1, 0, "heart", "a chrome flake from the narrator's shoulder"),
    ("cur_neon_tube", "Night City Neon Tube", "rare", (3, 1), 1.0, 55, "leak", "still lights pink when it gets signal"),
    ("cur_billboard", "Billboard Fragment", "rare", (2, 2), 2.0, 40, "leak", "half a face selling something"),
    ("cur_metro_ticket", "Metro Ticket", "uncommon", (1, 1), 0.0, 6, "leak", "one trip, Undercroft to anywhere"),
    ("cur_vaccine_vial", "MAP-SSRI-II Vial", "rare", (1, 1), 0.1, 30, None, "empty; the label promises safety"),
]
for id, name, tier, size, w, v, setname, codex in curios:
    item(id, name, "curio", tier, size, w, value=v, set=setname, codex=codex)

# ---------------------------------------------------------------- RELICS (5)
relics = [
    ("rel_reactor_fragment", "Reliquary Reactor Fragment", {"plasticity": 1.0, "conductance": 0.6}, "helium-3 vessel wall; the Mat trembles near it"),
    ("rel_holy_rock", "Holy Rock Chip", {"plasticity": 1.0, "chrome": 0.6}, "lunar stone from under the basilica"),
    ("rel_saint_implant", "Saint's Implant", {"plasticity": 1.0, "lume": 0.7}, "a chaplain's chrome that never went soft"),
    ("rel_first_vaccine", "First-Batch Vaccine", {"plasticity": 1.0, "feral": 0.7}, "the ampoule that started Slimecity"),
    ("rel_hive_seed", "Hive Seed", {"plasticity": 1.0, "chrome": 1.0, "conductance": 0.8}, "a Relic that grows chrome; the Hive ending's key"),
]
for id, name, bias, codex in relics:
    item(id, name, "relic", "relic", (2, 2), 3.0, value=0, bias=bias, codex=codex)


def toml_str(s):
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'


out = ["# Cyberpunk: ROSE — item table. Source of truth for the `items` crate. Authored, not generated at runtime.",
       "# Fields: id name category tier size(w,h) weight(kg) value(eddies) feed(nutrient) bias{gene=weight} tags set gear codex",
       f"# {len(items)} items.", ""]
ids = set()
for it in items:
    assert it["id"] not in ids, it["id"]
    ids.add(it["id"])
    out.append("[[item]]")
    out.append(f'id = {toml_str(it["id"])}')
    out.append(f'name = {toml_str(it["name"])}')
    out.append(f'category = {toml_str(it["category"])}')
    out.append(f'tier = {toml_str(it["tier"])}')
    out.append(f'size = [{it["size"][0]}, {it["size"][1]}]')
    out.append(f'weight = {it["weight"]}')
    out.append(f'value = {it["value"]}')
    out.append(f'feed = {it["feed"]}')
    if it["bias"]:
        out.append("bias = { " + ", ".join(f"{k} = {v}" for k, v in it["bias"].items()) + " }")
    if it["tags"]:
        out.append("tags = [" + ", ".join(toml_str(t) for t in it["tags"]) + "]")
    if it["set"]:
        out.append(f'set = {toml_str(it["set"])}')
    if it["gear"]:
        g = it["gear"]
        parts_ = []
        for k, v in g.items():
            parts_.append(f"{k} = {toml_str(v) if isinstance(v, str) else v}")
        out.append("gear = { " + ", ".join(parts_) + " }")
    out.append(f'codex = {toml_str(it["codex"])}')
    out.append("")

path = sys.argv[1] if len(sys.argv) > 1 else "data/items.toml"
open(path, "w").write("\n".join(out))
from collections import Counter
print(len(items), dict(Counter(i["category"] for i in items)))
