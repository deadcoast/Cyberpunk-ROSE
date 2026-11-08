extends ScreenBase
## The Stash: a view of the city's one grid (Sim.stash, inventory crate through RoseStash), item card with the four
## verbs. Add loot, place, pin, upgrade, sell/feed numbers live; Sell/Feed/Craft actions land with PAC-026.

const CELL := 32
var stash: RoseStash
var items: RoseItems
var grid_frame: Panel
var cells: Control
var card: Control
var selected := -1
var status: Label
var haul_note := ""

func _ready() -> void:
	screen_title = "STASH"
	super._ready()

func _build_content(c: Control) -> void:
	stash = Sim.ensure_stash()
	items = RoseItems.new()
	grid_frame = make_frame(c, Vector2(0, 0), Vector2(12 * CELL + 8, 10 * CELL + 8))
	cells = Control.new()
	cells.position = Vector2(4, 4)
	grid_frame.add_child(cells)
	card = make_frame(c, Vector2(410, 0), Vector2(222, 230))
	status = make_label(c, "", Vector2(0, 0), UiTheme.DUST)
	status.position = Vector2(410, 236)
	make_button(c, "ADD LOOT", Vector2(410, 256), _add_loot)
	make_button(c, "UPGRADE STASH", Vector2(490, 256), func():
		if stash.upgrade():
			_redraw())
	make_button(c, "SAVE TEST", Vector2(410, 280), func():
		status.text = "save round trip: " + ("ok" if stash.save_round_trip() else "FAILED"))
	# a Runner's rig handed over by the Run on extraction
	if not Sim.pending_haul.is_empty():
		var before: int = stash.count()
		var left: int = stash.import_rig(Sim.pending_haul)
		Sim.pending_haul = PackedByteArray()
		haul_note = "   haul from %s: %d in%s" % [Sim.pending_haul_from, stash.count() - before, ("   %d left at the Mat edge: no room" % left) if left > 0 else ""]
	# what the Mat hauled and what regrew at the Heart
	if not Sim.pending_items.is_empty():
		var before2: int = stash.count()
		var left2 := 0
		for b in Sim.pending_items:
			left2 += maxi(0, stash.import_items(b))
		Sim.pending_items.clear()
		haul_note += "   the Mat brought %d home%s" % [stash.count() - before2, ("   %d would not fit" % left2) if left2 > 0 else ""]
	_redraw()

func _add_loot() -> void:
	var n: int = items.count()
	var rng := RandomNumberGenerator.new()
	for k in 3:
		var idx := rng.randi_range(0, n - 1)
		stash.add(idx, rng.randi_range(20, 100), true)
	_redraw()

func _redraw() -> void:
	for ch in cells.get_children():
		ch.queue_free()
	var w: int = stash.width()
	var h: int = stash.height()
	grid_frame.size = Vector2(w * CELL + 8, h * CELL + 8)
	# cell grid lines
	for y in h:
		for x in w:
			var cell := ColorRect.new()
			cell.color = Color8(23, 26, 38)
			cell.position = Vector2(x * CELL, y * CELL)
			cell.size = Vector2(CELL - 1, CELL - 1)
			cells.add_child(cell)
	for it in stash.items():
		var btn := Button.new()
		btn.flat = true
		btn.position = Vector2(it.x * CELL, it.y * CELL)
		btn.size = Vector2(it.w * CELL, it.h * CELL)
		btn.focus_mode = Control.FOCUS_NONE
		var hnd: int = it.handle
		btn.pressed.connect(func(): _select(hnd))
		cells.add_child(btn)
		var icon := make_slot(btn, icon_for(it.id), Vector2(0, 0))
		icon.stretch_mode = TextureRect.STRETCH_SCALE
		icon.size = btn.size
		icon.custom_minimum_size = btn.size
		if not it.identified:
			icon.modulate = Color(0.45, 0.45, 0.5)
		if it.pinned:
			var pin := ColorRect.new()
			pin.color = UiTheme.ACID
			pin.position = Vector2(btn.size.x - 4, 0)
			pin.size = Vector2(4, 4)
			btn.add_child(pin)
	status.text = "tier %d   %d items   %d free cells%s" % [stash.tier(), stash.count(), stash.free_cells(), haul_note]
	if selected >= 0:
		_select(selected)

func _select(handle: int) -> void:
	selected = handle
	for ch in card.get_children():
		ch.queue_free()
	var found: Dictionary = {}
	for it in stash.items():
		if it.handle == handle:
			found = it
	if found.is_empty():
		return
	make_slot(card, icon_for(found.id), Vector2(6, 6), 0, 0, 0.0, "", 2)
	make_label(card, (found.name if found.identified else "UNIDENTIFIED").to_upper(), Vector2(76, 6), UiTheme.CYAN)
	make_label(card, "%s  -  condition %d  -  %dx%d" % [found.tier, found.condition, found.w, found.h], Vector2(76, 18), UiTheme.DUST)
	make_label(card, "found in run" if found.identified == false else "appraised", Vector2(76, 30), UiTheme.DUST)
	var y := 80
	make_label(card, "SELL   %d eddies" % found.sell, Vector2(6, y), UiTheme.ACID)
	make_label(card, "FEED   %.2f nutrient" % found.feed, Vector2(6, y + 12), UiTheme.OOZE_GLOW)
	make_label(card, "CRAFT  by tag (Phase B)", Vector2(6, y + 24), UiTheme.DUST)
	make_button(card, "SELL", Vector2(6, 130))
	make_button(card, "FEED", Vector2(60, 130))
	make_button(card, "CRAFT", Vector2(114, 130))
	make_button(card, "UNPIN" if found.pinned else "KEEP", Vector2(168, 130), func():
		stash.pin(handle, not found.pinned)
		_redraw())
	make_button(card, "DISCARD", Vector2(6, 160), func():
		stash.remove(handle)
		selected = -1
		_redraw()
		for ch in card.get_children():
			ch.queue_free())
