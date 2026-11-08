extends ScreenBase
## Tweaker Bench: recipe tiers, craft-tag slots, outcome preview. Craft on the Stash arrives here with the item
## in the first slot that takes one of its tags (Full Design v3.4); the recipe table lands with PAC-027, so the
## outcome panel says what the slotted item offers and that no recipe is loaded yet.

const TAGS := ["blade", "housing", "conductor", "optic", "servo", "cell", "board", "filter", "pump", "weave", "enzyme", "salt"]

var slotted := {}
var slot_frames := {}

func _ready() -> void:
	screen_title = "BENCH"
	super._ready()

func _build_content(c: Control) -> void:
	slotted = Sim.bench_slot
	Sim.bench_slot = {}
	var tiers := make_frame(c, Vector2(0, 0), Vector2(120, 290))
	make_label(tiers, "RECIPES", Vector2(6, 4), UiTheme.CYAN)
	var y := 20
	for t in ["TIER 1  no recipes yet", "TIER 2  blueprint", "TIER 3  blueprint", "TIER 4  blueprint"]:
		make_label(tiers, t, Vector2(6, y), UiTheme.CHALK if y == 20 else UiTheme.DUST)
		y += 14
	var slots := make_frame(c, Vector2(126, 0), Vector2(300, 150))
	make_label(slots, "SLOTS BY CRAFT TAG", Vector2(6, 4), UiTheme.CYAN)
	var x := 6
	y = 20
	var tags: PackedStringArray = slotted.get("tags", PackedStringArray()) if not slotted.is_empty() else PackedStringArray()
	var placed := false
	for tag in TAGS:
		var f := make_frame(slots, Vector2(x, y), Vector2(44, 56))
		slot_frames[tag] = f
		var takes := tags.has(tag)
		make_label(f, tag.left(6).to_upper(), Vector2(3, 44), UiTheme.CHALK if takes else UiTheme.DUST)
		if takes and not placed:
			make_item_tile(f, Vector2(6, 6), 1, 1, String(slotted.tier), String(slotted.id), bool(slotted.identified), 32)
			placed = true
		else:
			make_slot(f, "anchor.item", Vector2(6, 6)).modulate = Color(0.4, 0.4, 0.45) if not takes else Color(0.7, 0.7, 0.75)
		x += 48
		if x > 250:
			x = 6
			y += 60
	var out := make_frame(c, Vector2(432, 0), Vector2(200, 150))
	make_label(out, "OUTCOME", Vector2(6, 4), UiTheme.CYAN)
	if slotted.is_empty():
		make_label(out, "slot a part from the Stash", Vector2(6, 20), UiTheme.DUST)
		make_label(out, "(Craft on an item's card)", Vector2(6, 32), UiTheme.DUST)
	else:
		make_label(out, (String(slotted.name) if bool(slotted.identified) else "UNIDENTIFIED").to_upper(), Vector2(6, 20), UiTheme.CHALK)
		make_label(out, "%s %s   condition %d" % [String(slotted.tier), String(slotted.category), int(slotted.condition)], Vector2(6, 32), UiTheme.DUST)
		make_label(out, "offers: " + ", ".join(tags), Vector2(6, 44), UiTheme.CHALK)
		make_label(out, "no recipe table loaded yet", Vector2(6, 64), UiTheme.DUST)
		make_label(out, "the Bench takes recipes next", Vector2(6, 76), UiTheme.DUST)
	var q := make_frame(c, Vector2(126, 156), Vector2(506, 134))
	make_label(q, "QUEUE", Vector2(6, 4), UiTheme.CYAN)
	make_label(q, "empty", Vector2(6, 18), UiTheme.DUST)
