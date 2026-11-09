extends ScreenBase
## The Tweaker Bench (Full Design v3.5): recipes by craft tag from data/recipes.toml through RoseStash. Recipe list
## by tier left (tier 1 open, 2-4 locked until blueprints); the chosen recipe's slots centre with a picker of
## Stash items that carry the slot's tag; outcome preview right with every stat against the base, condition and
## provenance; the queue below with shifts left. CRAFT takes the parts out of the Stash; CANCEL returns them; jobs
## finish when the shift ends. Craft on a Stash card arrives with that item pre-slotted (Sim.bench_slot).

const SLOT_W := 44
const SLOT_H := 58

var stash: RoseStash
var recipes: Array = []
var recipe_list: Panel
var slots_frame: Panel
var outcome_frame: Panel
var queue_frame: Panel
var chosen := ""                 # recipe id
var chosen_slot := 0
var filled: Array[int] = []      # handle per slot, -1 empty
var list_buttons := {}

func _ready() -> void:
	screen_title = "BENCH"
	super._ready()

func _build_content(c: Control) -> void:
	stash = Sim.ensure_stash()
	recipes = stash.recipes()
	recipe_list = make_frame(c, Vector2(0, 0), Vector2(150, 306))
	slots_frame = make_frame(c, Vector2(156, 0), Vector2(290, 150))
	outcome_frame = make_frame(c, Vector2(452, 0), Vector2(180, 150))
	queue_frame = make_frame(c, Vector2(156, 156), Vector2(476, 150))
	_draw_list()
	# an item sent over by Craft on the Stash: the first open recipe with a slot that takes it
	var sent: Dictionary = Sim.bench_slot
	Sim.bench_slot = {}
	if not sent.is_empty():
		var tags: PackedStringArray = sent.get("tags", PackedStringArray())
		for r in recipes:
			if not bool(r.open):
				continue
			var hit := -1
			for i in r.slots.size():
				var tag := String(r.slots[i].tag)
				if tags.has(tag) or (tag == "gear" and String(sent.get("category", "")) == "gear" and bool(sent.get("identified", true))) or (tag == "unidentified" and not bool(sent.get("identified", true))):
					hit = i
					break
			if hit >= 0:
				_choose(String(r.id))
				filled[hit] = int(sent.get("handle", -1))
				chosen_slot = (hit + 1) % filled.size()
				break
	if chosen == "":
		_choose(String(recipes[0].id) if recipes.size() > 0 else "")
	_redraw()

func _recipe(id: String) -> Dictionary:
	for r in recipes:
		if String(r.id) == id:
			return r
	return {}

func _choose(id: String) -> void:
	chosen = id
	chosen_slot = 0
	filled.clear()
	var r := _recipe(id)
	for i in r.get("slots", []).size():
		filled.append(-1)

func _draw_list() -> void:
	for ch in recipe_list.get_children():
		ch.queue_free()
	make_label(recipe_list, "RECIPES   tier %d open" % int(stash.tier_open()), Vector2(6, 4), UiTheme.CYAN)
	var y := 18
	for r in recipes:
		if not bool(r.open):
			continue
		var id := String(r.id)
		var b := make_button(recipe_list, String(r.name).to_upper(), Vector2(4, y), func(): _pick(id))
		list_buttons[id] = b
		y += 13
	y += 6
	for tier in range(int(stash.tier_open()) + 1, 5):
		var n := 0
		for r in recipes:
			if int(r.tier) == tier:
				n += 1
		make_label(recipe_list, "TIER %d   %d recipes   blueprint" % [tier, n], Vector2(6, y), UiTheme.DUST)
		y += 12

func _pick(id: String) -> void:
	_choose(id)
	_redraw()

func _redraw() -> void:
	for id in list_buttons:
		list_buttons[id].modulate = Color.WHITE if id == chosen else Color(0.62, 0.62, 0.68)
	_draw_slots()
	_draw_outcome()
	_draw_queue()

func _item(handle: int) -> Dictionary:
	if handle < 0:
		return {}
	for it in stash.items():
		if int(it.handle) == handle:
			return it
	return {}

func _draw_slots() -> void:
	for ch in slots_frame.get_children():
		ch.queue_free()
	var r := _recipe(chosen)
	if r.is_empty():
		make_label(slots_frame, "pick a recipe", Vector2(6, 4), UiTheme.DUST)
		return
	var shifts := int(r.shifts)
	make_label(slots_frame, "%s   tier %d   %d shift%s" % [String(r.name).to_upper(), int(r.tier), shifts, "" if shifts == 1 else "s"], Vector2(6, 4), UiTheme.CYAN)
	make_label(slots_frame, "makes " + String(r.product_name) + (("  x%d" % int(r.count)) if int(r.count) > 1 else ""), Vector2(6, 16), UiTheme.DUST)
	var slots: Array = r.slots
	for i in slots.size():
		var f := make_frame(slots_frame, Vector2(6 + i * (SLOT_W + 8), 28), Vector2(SLOT_W, SLOT_H))
		f.modulate = Color.WHITE if i == chosen_slot else Color(0.7, 0.7, 0.76)
		var tag := String(slots[i].tag)
		var drives := String(slots[i].drives)
		var it := _item(filled[i])
		if it.is_empty():
			make_slot(f, "anchor.item", Vector2(6, 4)).modulate = Color(0.45, 0.45, 0.5)
		else:
			make_item_tile(f, Vector2(6, 4), 1, 1, String(it.tier), String(it.id), bool(it.identified), 32)
		make_label(f, tag.left(6).to_upper(), Vector2(3, 38), UiTheme.CHALK)
		make_label(f, drives.to_upper() if drives != "" else "binder", Vector2(3, 48), UiTheme.CYAN if drives != "" else UiTheme.DUST)
		var k := i
		var b := Button.new()
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.size = f.size
		b.pressed.connect(func():
			if filled[k] >= 0:
				filled[k] = -1
			chosen_slot = k
			_redraw())
		f.add_child(b)
	# the picker: Stash items that fill the chosen slot
	if chosen_slot < slots.size():
		var tag := String(slots[chosen_slot].tag)
		var cands: PackedInt32Array = stash.candidates(tag)
		make_label(slots_frame, "FITS %s   %d in the Stash" % [tag.to_upper(), cands.size()], Vector2(6, 92), UiTheme.DUST)
		var shown := 0
		for h in cands:
			if filled.has(h):
				continue
			if shown >= 7:
				break
			var it := _item(h)
			if it.is_empty():
				continue
			var tile := make_item_tile(slots_frame, Vector2(6 + shown * 38, 104), 1, 1, String(it.tier), String(it.id), bool(it.identified), 32)
			var hh := h
			var b := Button.new()
			b.flat = true
			b.focus_mode = Control.FOCUS_NONE
			b.size = tile.size
			b.pressed.connect(func(): _fill(hh))
			tile.add_child(b)
			shown += 1
		if shown == 0:
			make_label(slots_frame, "all of them are in a slot already" if cands.size() > 0 else "nothing in the Stash carries this tag", Vector2(6, 112), UiTheme.DUST)

func _fill(handle: int) -> void:
	if chosen_slot < filled.size():
		filled[chosen_slot] = handle
		# step to the next empty slot
		for k in filled.size():
			var j := (chosen_slot + 1 + k) % filled.size()
			if filled[j] < 0:
				chosen_slot = j
				break
	_redraw()

func _complete() -> bool:
	return filled.size() > 0 and not filled.has(-1)

func _draw_outcome() -> void:
	for ch in outcome_frame.get_children():
		ch.queue_free()
	make_label(outcome_frame, "OUTCOME", Vector2(6, 4), UiTheme.CYAN)
	var r := _recipe(chosen)
	if r.is_empty():
		return
	if not _complete():
		var missing := filled.count(-1)
		make_label(outcome_frame, "%d slot%s to fill" % [missing, "" if missing == 1 else "s"], Vector2(6, 18), UiTheme.DUST)
		make_label(outcome_frame, "click a slot, then an item", Vector2(6, 30), UiTheme.DUST)
		return
	var p: Dictionary = stash.preview(chosen, PackedInt32Array(filled), "the Bench", Sim.day)
	if not bool(p.get("ok", false)):
		make_label(outcome_frame, "will not craft:", Vector2(6, 18), UiTheme.HOT_PINK)
		make_label(outcome_frame, String(p.get("error", "")).left(40), Vector2(6, 30), UiTheme.DUST)
		return
	make_item_tile(outcome_frame, Vector2(6, 16), 1, 1, String(p.tier), String(p.product_id), true, 32)
	make_label(outcome_frame, String(p.product_name).to_upper(), Vector2(44, 18), UiTheme.CHALK)
	make_label(outcome_frame, "%s   cond %d%s" % [String(p.tier), int(p.condition), ("   x%d" % int(p.count)) if int(p.count) > 1 else ""], Vector2(44, 30), UiTheme.DUST)
	var y := 52
	if p.has("base") and p.has("stats"):
		var base: Dictionary = p.base
		var st: Dictionary = p.stats
		for key in ["dmg", "armour", "noise", "light", "mag"]:
			var b := int(base[key])
			var v := int(st[key])
			if b == 0 and v == 0:
				continue
			var better := (v > b) if key != "noise" else (v < b)
			var col: Color = UiTheme.CHALK if v == b else (UiTheme.CYAN if better else UiTheme.HOT_PINK)
			make_label(outcome_frame, ("%s  %d" % [key.to_upper(), v]) if v == b else ("%s  %d to %d" % [key.to_upper(), b, v]), Vector2(6, y), col)
			y += 11
	if bool(p.get("spored", false)) and String(r.product) == "@spored":
		make_label(outcome_frame, "SPORED: regrows once", Vector2(6, y), UiTheme.OOZE_GLOW)
		y += 11
	var parts: PackedStringArray = p.get("parts", PackedStringArray())
	if parts.size() > 0:
		make_label(outcome_frame, "by %s, day %d" % [String(p.get("by", "the Bench")), Sim.day], Vector2(6, y), UiTheme.DUST)
		y += 11
		for part in parts:
			make_label(outcome_frame, "  " + String(part).left(28), Vector2(6, y), UiTheme.DUST)
			y += 10
	make_button(outcome_frame, "CRAFT", Vector2(120, 128), func():
		if stash.craft(chosen, PackedInt32Array(filled), "the Bench", Sim.day):
			_choose(chosen)
			_redraw())

func _draw_queue() -> void:
	for ch in queue_frame.get_children():
		ch.queue_free()
	var jobs: Array = stash.bench_jobs()
	make_label(queue_frame, "QUEUE   %d job%s" % [jobs.size(), "" if jobs.size() == 1 else "s"], Vector2(6, 4), UiTheme.CYAN)
	var y := 18
	for j in jobs:
		if y > 118:
			make_label(queue_frame, "+%d more" % (jobs.size() - 8), Vector2(6, y), UiTheme.DUST)
			break
		var idx := int(j.index)
		var left := int(j.shifts_left)
		var parts: PackedStringArray = j.parts
		make_label(queue_frame, "%s   %d shift%s left" % [String(j.name).to_upper(), left, "" if left == 1 else "s"], Vector2(6, y), UiTheme.CHALK)
		make_label(queue_frame, "from " + ", ".join(parts), Vector2(160, y), UiTheme.DUST)
		make_button(queue_frame, "CANCEL", Vector2(416, y - 3), func():
			stash.cancel_job(idx)
			_redraw())
		y += 13
	var waiting := int(stash.bench_waiting())
	if waiting > 0:
		make_label(queue_frame, "%d finished, waiting for room in the Stash" % waiting, Vector2(6, y), UiTheme.ACID)
		y += 12
	if jobs.is_empty():
		make_label(queue_frame, "nothing on the Bench", Vector2(6, y), UiTheme.DUST)
	make_label(queue_frame, "jobs finish when the shift ends, after a run", Vector2(6, 134), UiTheme.DUST)
