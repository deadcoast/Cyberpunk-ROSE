extends ScreenBase
## Tweaker Bench: recipe tiers, craft-tag slots, outcome preview. Phase A: layout and the tag list.

const TAGS := ["blade", "housing", "conductor", "optic", "servo", "cell", "board", "filter", "pump", "weave", "enzyme", "salt"]

func _ready() -> void:
	screen_title = "BENCH"
	super._ready()

func _build_content(c: Control) -> void:
	var tiers := make_frame(c, Vector2(0, 0), Vector2(120, 290))
	make_label(tiers, "RECIPES", Vector2(6, 4), UiTheme.CYAN)
	var y := 20
	for t in ["TIER 1  scrap sword - zip gun - tarp coat - lantern - salt bag - repair kit", "TIER 2  (blueprint)", "TIER 3  (blueprint)", "TIER 4  (blueprint)"]:
		make_label(tiers, t.left(14), Vector2(6, y), UiTheme.CHALK if y == 20 else UiTheme.DUST)
		y += 14
	var slots := make_frame(c, Vector2(126, 0), Vector2(300, 150))
	make_label(slots, "SLOTS BY CRAFT TAG", Vector2(6, 4), UiTheme.CYAN)
	var x := 6
	y = 20
	for tag in TAGS:
		var f := make_frame(slots, Vector2(x, y), Vector2(44, 56))
		make_label(f, tag.left(6).to_upper(), Vector2(3, 40), UiTheme.DUST)
		make_slot(f, "anchor.item", Vector2(6, 4)).modulate = Color(0.4, 0.4, 0.45)
		x += 48
		if x > 250:
			x = 6
			y += 60
	var out := make_frame(c, Vector2(432, 0), Vector2(200, 150))
	make_label(out, "OUTCOME", Vector2(6, 4), UiTheme.CYAN)
	make_slot(out, icon_for("ger_sword"), Vector2(6, 20), 0, 0, 0.0, "", 2)
	make_label(out, "Scrap Sword", Vector2(80, 20), UiTheme.CHALK)
	make_label(out, "blade + housing + conductor", Vector2(80, 32), UiTheme.DUST)
	make_label(out, "dmg 13  noise 2", Vector2(80, 44), UiTheme.DUST)
	make_label(out, "provenance: Culverts, day 3", Vector2(80, 56), UiTheme.DUST)
	make_button(out, "QUEUE", Vector2(6, 110))
	var q := make_frame(c, Vector2(126, 156), Vector2(506, 134))
	make_label(q, "QUEUE   1. Scrap Sword (Deacon, 1 shift)   2. Salt Bag x2   3. Repair: Nail Rifle +30", Vector2(6, 4), UiTheme.DUST)
