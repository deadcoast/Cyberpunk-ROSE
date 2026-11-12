extends ScreenBase
## Codex (Full Design v3.18, UI and screens: Codex; Progression: Collections): the Scrapdex (the 150 entries as tiles,
## silhouettes for the unseen), the Strainbook (the 38 cards, dark until earned), the curio sets (each with its three
## and its passive), the Memorial (the dead) and the Trophies (the twelve). Every rule is the progress crate's
## (RoseSim.codex); the screen reads it. Lore (Full Design v3.19) keeps every panel set seen and replays it.

const TABS := ["SCRAPDEX", "STRAINBOOK", "SETS", "MEMORIAL", "TROPHIES", "LORE"]

var tab := "SCRAPDEX"
var view: Control
var buttons := {}
var data: Dictionary
var detail := ""

func _ready() -> void:
	screen_title = "CODEX"
	super._ready()

func _build_content(c: Control) -> void:
	data = Sim.ensure().codex(Sim.ensure_people())
	# what outlives a run (Full Design v3.19, Meta): an entry seen or earned in any run counts
	var dex_ever: PackedStringArray = Sim.ensure_profile().scrapdex()
	for e in data.get("scrapdex", []):
		if not bool(e.seen) and dex_ever.has(String(e.id)):
			e.seen = true
	var book_ever: PackedStringArray = Sim.ensure_profile().strainbook()
	for e in data.get("strainbook", []):
		if int(e.earned) < 0 and book_ever.has(String(e.id)):
			e.earned = 0
	var x := 0
	for t in TABS:
		var tt: String = t
		buttons[t] = make_button(c, t, Vector2(x, 0), func(): _set_tab(tt))
		x += 72
	view = Control.new()
	view.position = Vector2(0, 16)
	c.add_child(view)
	_set_tab(Sim.codex_tab if Sim.codex_tab != "" else "SCRAPDEX")

func _set_tab(t: String) -> void:
	tab = t
	Sim.codex_tab = t
	detail = ""
	for k in buttons:
		buttons[k].modulate = Color.WHITE if k == t else Color(0.55, 0.55, 0.62)
	_draw()

func _draw() -> void:
	for ch in view.get_children():
		ch.queue_free()
	match tab:
		"SCRAPDEX":
			_scrapdex()
		"STRAINBOOK":
			_strainbook()
		"SETS":
			_sets()
		"MEMORIAL":
			_memorial()
		"TROPHIES":
			_trophies()
		"LORE":
			_lore()

func _click(parent: Control, size_: Vector2, text: String) -> void:
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.size = size_
	b.pressed.connect(func():
		detail = text
		_draw())
	parent.add_child(b)

func _scrapdex() -> void:
	var dex: Array = data.get("scrapdex", [])
	var seen := 0
	for e in dex:
		if bool(e.seen):
			seen += 1
	make_label(view, "SCRAPDEX   %d / %d seen   an entry is seen when an item of it is identified" % [seen, dex.size()], Vector2(0, 2), UiTheme.CYAN)
	var cols := 19
	for i in dex.size():
		var e: Dictionary = dex[i]
		var pos := Vector2((i % cols) * 33, 16 + (i / cols) * 33)
		var s := make_slot(view, icon_for(String(e.id)), pos)
		if not bool(e.seen):
			s.modulate = Color(0.16, 0.16, 0.2)
		_click(s, Vector2(32, 32), ("%s: %s" % [String(e.name).to_upper(), String(e.codex)]) if bool(e.seen) else "unseen: a %s item" % String(e.tier))
	if detail != "":
		make_label(view, detail.left(100), Vector2(0, 280), UiTheme.CHALK)

func _strainbook() -> void:
	var book: Array = data.get("strainbook", [])
	var earned := 0
	for e in book:
		if int(e.earned) >= 0:
			earned += 1
	make_label(view, "STRAINBOOK   %d / %d   organs, classes, the Relic state, notable mutations" % [earned, book.size()], Vector2(0, 2), UiTheme.CYAN)
	for i in book.size():
		var e: Dictionary = book[i]
		var pos := Vector2((i % 8) * 79, 16 + (i / 8) * 52)
		var f := make_frame(view, pos, Vector2(76, 50))
		var got := int(e.earned) >= 0
		var art := String(e.art)
		if art != "":
			var s := make_slot(f, art, Vector2(40, 2), 32, 32, 0.0, Assets.organ_state(art))
			if not got:
				s.modulate = Color(0.25, 0.25, 0.3)
		var words := String(e.name).to_upper().split(" ")
		var l1 := ""
		var l2 := ""
		for w in words:
			if l2 == "" and (l1 + " " + w).strip_edges().length() <= (6 if art != "" else 11):
				l1 = (l1 + " " + w).strip_edges()
			else:
				l2 = (l2 + " " + w).strip_edges()
		var col: Color = UiTheme.OOZE_GLOW if got else UiTheme.DUST
		make_label(f, l1 if got else "?", Vector2(3, 3), col)
		if got:
			make_label(f, l2.left(11), Vector2(3, 13), col)
		make_label(f, "SHIFT %d" % int(e.earned) if got else "", Vector2(3, 36), UiTheme.DUST)
		_click(f, f.size, ("%s: %s" % [String(e.name).to_upper(), String(e.text)]) if got else "not yet")
	if detail != "":
		make_label(view, detail.left(100), Vector2(0, 280), UiTheme.CHALK)

func _sets() -> void:
	var sets: Array = data.get("sets", [])
	make_label(view, "CURIO SETS   three of a set in the Stash: its passive holds", Vector2(0, 2), UiTheme.CYAN)
	var y := 16
	for e in sets:
		var f := make_frame(view, Vector2(0, y), Vector2(632, 38))
		var done := bool(e.complete)
		make_label(f, String(e.name).to_upper(), Vector2(6, 4), UiTheme.ACID if done else UiTheme.CHALK)
		make_label(f, "%d OF 3" % mini(3, int(e.owned)), Vector2(6, 16), UiTheme.CYAN if done else UiTheme.DUST)
		var items: PackedStringArray = e.items
		for k in items.size():
			var s := make_slot(f, icon_for(items[k]), Vector2(110 + k * 34, 3))
			if k >= int(e.owned):
				s.modulate = Color(0.3, 0.3, 0.36)
		make_label(f, String(e.text).left(64), Vector2(220, 4), UiTheme.CHALK if done else UiTheme.DUST)
		make_label(f, "HOLDS" if done else "", Vector2(220, 16), UiTheme.ACID)
		y += 40

func _memorial() -> void:
	var dead: Array = data.get("memorial", [])
	make_label(view, "MEMORIAL   %d name%s on the wall" % [dead.size(), "" if dead.size() == 1 else "s"], Vector2(0, 2), UiTheme.CYAN)
	var y := 18
	if dead.is_empty():
		make_label(view, "nobody yet", Vector2(0, y), UiTheme.DUST)
	for e in dead:
		if y > 280:
			make_label(view, "and more", Vector2(0, y), UiTheme.DUST)
			break
		make_label(view, String(e.name).to_upper(), Vector2(0, y), UiTheme.CHALK)
		make_label(view, "%s %s" % [String(e.depth), String(e.past).to_upper()], Vector2(220, y), UiTheme.DUST)
		make_label(view, String(e.line), Vector2(320, y), UiTheme.DUST)
		y += 12

func _trophies() -> void:
	var tr: Array = data.get("trophies", [])
	var won := 0
	for e in tr:
		if int(e.won) >= 0:
			won += 1
	make_label(view, "TROPHIES   %d / %d" % [won, tr.size()], Vector2(0, 2), UiTheme.CYAN)
	for i in tr.size():
		var e: Dictionary = tr[i]
		var f := make_frame(view, Vector2((i % 4) * 158, 16 + (i / 4) * 54), Vector2(154, 50))
		var got := int(e.won) >= 0
		make_label(f, String(e.name).to_upper(), Vector2(6, 4), UiTheme.ACID if got else UiTheme.DUST)
		make_label(f, String(e.text).left(24), Vector2(6, 16), UiTheme.CHALK if got else UiTheme.DUST)
		make_label(f, "SHIFT %d" % int(e.won) if got else "", Vector2(6, 34), UiTheme.DUST)

## The panel sets seen, in the order they played; a click replays one. Beside them, the endings reached in every run
## with their warchests and strain codes (the profile).
func _lore() -> void:
	var camp: Dictionary = Sim.ensure().campaign()
	var seen: PackedStringArray = camp.get("seen", PackedStringArray())
	make_label(view, "LORE   %d panel set%s seen   click one to see it again" % [seen.size(), "" if seen.size() == 1 else "s"], Vector2(0, 2), UiTheme.CYAN)
	var ends: Array = Sim.ensure_profile().summary().get("endings", [])
	make_label(view, "ENDINGS REACHED %d" % ends.size(), Vector2(330, 2), UiTheme.CYAN)
	var ey := 18
	for e in ends:
		make_label(view, "%s  WARCHEST %d  SHIFT %d  %s" % [String(e.name).to_upper(), int(e.warchest), int(e.shift), String(e.difficulty).to_upper()], Vector2(330, ey), UiTheme.OOZE_GLOW)
		make_label(view, String(e.code), Vector2(330, ey + 10), UiTheme.DUST)
		ey += 24
		if ey > 270:
			break
	var y := 18
	if seen.is_empty():
		make_label(view, "nothing yet: a chapter's panels land here once they have played", Vector2(0, y), UiTheme.DUST)
	for k in seen:
		var key := String(k)
		var title := String(Sim.ensure().panel_set(key).get("title", key))
		make_button(view, title, Vector2(0, y), func():
			Sim.panel_replay = key
			Sim.panel_return = "codex"
			Screens.show_screen("panels"))
		y += 16
		if y > 280:
			break
