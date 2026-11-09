extends ScreenBase
## The Build menu (Full Design v3.9, The 24 structures): every def from data/structures.toml through
## RoseSim.structure_defs, with its footprint, cost and build points, the job it hosts and its slots, its per-tier
## numbers and what the city has of it. PLACE arms the Colony's build tool with the def; the Colony screen puts the
## ghost on the Mat. The list scrolls in pages of eight.

const PER_PAGE := 8

var defs: Array = []
var page := 0
var chosen := ""
var list_frame: Panel
var card_frame: Panel
var sim: RoseSim

func _ready() -> void:
	screen_title = "BUILD"
	super._ready()

func _build_content(c: Control) -> void:
	sim = Sim.ensure()
	defs = sim.structure_defs()
	list_frame = make_frame(c, Vector2(0, 0), Vector2(300, 306))
	card_frame = make_frame(c, Vector2(306, 0), Vector2(326, 306))
	if Sim.build_tool != "":
		chosen = Sim.build_tool
	elif defs.size() > 0:
		chosen = String(defs[0].id)
	for i in defs.size():
		if String(defs[i].id) == chosen:
			page = i / PER_PAGE
	_redraw()

## Lines of at most `width` characters, broken at spaces.
func _wrap(text: String, width: int) -> Array:
	var lines := []
	var line := ""
	for word in text.split(" ", false):
		if line != "" and line.length() + 1 + word.length() > width:
			lines.append(line)
			line = word
		else:
			line = word if line == "" else line + " " + word
	if line != "":
		lines.append(line)
	return lines

## A whole sprite fitted into a box, aspect kept.
func _thumb(parent: Control, art: String, state: String, pos: Vector2, box: Vector2) -> AssetSlot:
	var slot := make_slot(parent, art, pos, 0, 0, 0.0, state)
	slot.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	slot.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	slot.custom_minimum_size = box
	slot.size = box
	return slot

func _count(def_id: String) -> Array:
	var built := 0
	var ghosts := 0
	for s in sim.structures():
		if String(s.def) == def_id:
			if bool(s.built):
				built += 1
			else:
				ghosts += 1
	return [built, ghosts]

func _redraw() -> void:
	_draw_list()
	_draw_card()

func _draw_list() -> void:
	for ch in list_frame.get_children():
		ch.queue_free()
	var pages := int(ceil(float(defs.size()) / PER_PAGE))
	make_label(list_frame, "STRUCTURES   %d   page %d/%d" % [defs.size(), page + 1, pages], Vector2(6, 4), UiTheme.CYAN)
	var pop: int = Sim.ensure_people().count()
	make_label(list_frame, "%s of %d   tier %d" % [String(sim.city_tier_name(pop)).to_upper(), pop, int(sim.city_tier(pop))], Vector2(190, 4), UiTheme.DUST)
	var y := 18
	for i in range(page * PER_PAGE, mini(defs.size(), (page + 1) * PER_PAGE)):
		var d: Dictionary = defs[i]
		var id := String(d.id)
		var row := Control.new()
		row.position = Vector2(0, y)
		list_frame.add_child(row)
		_thumb(row, String(d.art), "t1", Vector2(6, 0), Vector2(24, 30))
		var n := _count(id)
		var col: Color = UiTheme.CYAN if id == chosen else UiTheme.CHALK
		make_label(row, String(d.name).to_upper(), Vector2(36, 1), col)
		var line := "%dx%d   %s   %d pts" % [int(d.w), int(d.h), String(d.cost.text), int(d.build)]
		if bool(d.prebuilt):
			line = "%dx%d   prebuilt" % [int(d.w), int(d.h)]
		make_label(row, line, Vector2(36, 11), UiTheme.DUST)
		var have := []
		if n[0] > 0:
			have.append("%d built" % n[0])
		if n[1] > 0:
			have.append("%d ghost%s" % [n[1], "" if n[1] == 1 else "s"])
		if not have.is_empty():
			make_label(row, ", ".join(have), Vector2(36, 21), UiTheme.OOZE_GLOW)
		var b := Button.new()
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.size = Vector2(296, 32)
		b.pressed.connect(func():
			chosen = id
			_redraw())
		row.add_child(b)
		y += 34
	if page > 0:
		make_button(list_frame, "PREV", Vector2(6, 288), func():
			page -= 1
			_redraw())
	if page + 1 < pages:
		make_button(list_frame, "NEXT", Vector2(250, 288), func():
			page += 1
			_redraw())

func _def(id: String) -> Dictionary:
	for d in defs:
		if String(d.id) == id:
			return d
	return {}

func _draw_card() -> void:
	for ch in card_frame.get_children():
		ch.queue_free()
	var d := _def(chosen)
	if d.is_empty():
		make_label(card_frame, "pick a structure", Vector2(6, 4), UiTheme.DUST)
		return
	# the tiers side by side (the Heart has four)
	var tx := 8
	var tiers := []
	for k in int(d.tiers):
		tiers.append("t%d" % (k + 1))
	for t in tiers:
		var f := make_frame(card_frame, Vector2(tx, 20), Vector2(70, 70))
		_thumb(f, String(d.art), t, Vector2(3, 3), Vector2(64, 64))
		make_label(card_frame, t.to_upper(), Vector2(tx + 28, 92), UiTheme.DUST)
		tx += 76
	make_label(card_frame, String(d.name).to_upper(), Vector2(8, 4), UiTheme.CYAN)
	make_label(card_frame, "%d tiers" % int(d.tiers), Vector2(250, 4), UiTheme.DUST)
	var y := 106
	for line in _wrap(String(d.text), 62):
		make_label(card_frame, line, Vector2(8, y), UiTheme.CHALK)
		y += 10
	y += 4
	var stands := "stands on %dx%d: anchor on Cord or Node" % [int(d.w), int(d.h)]
	if bool(d.node):
		stands = "stands on %dx%d: anchor on a Node" % [int(d.w), int(d.h)]
	if bool(d.relay):
		stands = "stands on a Relay"
	if bool(d.grate):
		stands += ", a grate within 2"
	if float(d.sigma) > 0.0:
		stands += "   signal %.1f" % float(d.sigma)
	make_label(card_frame, stands, Vector2(8, y), UiTheme.DUST)
	y += 11
	if bool(d.prebuilt):
		make_label(card_frame, "prebuilt at the founding", Vector2(8, y), UiTheme.DUST)
	else:
		make_label(card_frame, "costs %s   %d build points (a builder adds Hands a shift)" % [String(d.cost.text), int(d.build)], Vector2(8, y), UiTheme.ACID)
	y += 11
	if int(d.heart_tier) > 0:
		make_label(card_frame, "needs the Heart at tier %d" % int(d.heart_tier), Vector2(8, y), UiTheme.HOT_PINK)
		y += 11
	if String(d.hosts) != "":
		var slots: PackedInt32Array = d.slots
		var sl := []
		for v in slots:
			sl.append(str(v))
		var gate := "   (%s %d)" % [String(d.hosts).capitalize(), int(d.gate)] if int(d.gate) > 0 else ""
		make_label(card_frame, "hosts %s   slots %s%s" % [String(d.hosts).to_upper(), " / ".join(sl), gate], Vector2(8, y), UiTheme.CYAN)
		y += 11
	for eff in d.effects:
		var vals: PackedInt32Array = eff.values
		var vs := []
		for v in vals:
			vs.append(str(v))
		make_label(card_frame, "%s  %s" % [String(eff.name), " / ".join(vs)], Vector2(8, y), UiTheme.DUST)
		y += 10
	var n := _count(chosen)
	make_label(card_frame, "in the city: %d built, %d ghost%s" % [n[0], n[1], "" if n[1] == 1 else "s"], Vector2(8, 260), UiTheme.OOZE_GLOW)
	if not bool(d.prebuilt):
		make_button(card_frame, "PLACE ON THE COLONY", Vector2(170, 282), func():
			Sim.build_tool = chosen
			Screens.show_screen("colony"))
	make_label(card_frame, "HP %d per tier" % int(d.hp), Vector2(8, 286), UiTheme.DUST)
