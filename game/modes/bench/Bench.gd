extends ScreenBase
## The Tweaker Bench (Full Design v3.5): recipes by craft tag from data/recipes.toml through RoseStash. Recipe list
## by tier left (tier 1 open, 2-4 locked until blueprints); the chosen recipe's slots centre with a picker of
## Stash items that carry the slot's tag; outcome preview right with every stat against the base, condition and
## provenance; the queue below with the minutes left. CRAFT takes the parts out of the Stash; CANCEL returns them; a
## Tweaker in a Bench slot works a job a minute at a time (Full Design v3.21), a recipe's shift ten minutes. Craft on a Stash card arrives with that item pre-slotted (Sim.bench_slot).
## RESEARCH (Full Design v3.18, Research): the tree as three columns of four tiers, a node a tile — researched lit,
## open with its points and cost, locked under its blueprint; the project standing, the picked node's card with
## RESEARCH; the blueprints in the Stash with STUDY. Every rule is the progress crate's (RoseSim.tech).

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
var tab := "craft"
var craft_view: Control
var research_view: Control
var tab_buttons := {}
var picked_node := ""
var research_msg := ""

func _ready() -> void:
	screen_title = "BENCH"
	super._ready()

func _build_content(c: Control) -> void:
	stash = Sim.ensure_stash()
	recipes = stash.recipes()
	for t in ["craft", "research"]:
		var tt: String = t
		tab_buttons[t] = make_button(c, t.to_upper(), Vector2(0 if t == "craft" else 52, 0), func(): _set_tab(tt))
	craft_view = Control.new()
	craft_view.position = Vector2(0, 16)
	c.add_child(craft_view)
	research_view = Control.new()
	research_view.position = Vector2(0, 16)
	c.add_child(research_view)
	recipe_list = make_frame(craft_view, Vector2(0, 0), Vector2(150, 290))
	slots_frame = make_frame(craft_view, Vector2(156, 0), Vector2(290, 140))
	outcome_frame = make_frame(craft_view, Vector2(452, 0), Vector2(180, 140))
	queue_frame = make_frame(craft_view, Vector2(156, 146), Vector2(476, 144))
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
	_set_tab(Sim.bench_tab if Sim.bench_tab != "" else "craft")

func _set_tab(t: String) -> void:
	tab = t
	Sim.bench_tab = t
	craft_view.visible = t == "craft"
	research_view.visible = t == "research"
	for k in tab_buttons:
		tab_buttons[k].modulate = Color.WHITE if k == t else Color(0.55, 0.55, 0.62)
	if t == "research":
		_draw_research()

# ---------------------------------------------------------------- research

const NODE_W := 64
const NODE_H := 40

func _draw_research() -> void:
	for ch in research_view.get_children():
		ch.queue_free()
	var sim: RoseSim = Sim.ensure()
	var t: Dictionary = sim.tech()
	var open: PackedInt32Array = t.open
	var names: PackedStringArray = t.branches
	var nodes: Array = t.nodes
	for b in 3:
		var col := make_frame(research_view, Vector2(b * 212, 0), Vector2(206, 200))
		make_label(col, "%s   tier %d open" % [names[b].to_upper(), open[b]], Vector2(6, 4), UiTheme.CYAN)
		for n in nodes:
			if int(n.branch) != b:
				continue
			var k := 0
			for m in nodes:
				if int(m.branch) == b and int(m.tier) == int(n.tier) and String(m.id) < String(n.id):
					k += 1
			var pos := Vector2(4 + k * (NODE_W + 4), 18 + (int(n.tier) - 1) * (NODE_H + 4))
			_node_tile(col, pos, n)
	var proj: Dictionary = t.project
	var pf := make_frame(research_view, Vector2(0, 206), Vector2(418, 84))
	if proj.is_empty():
		make_label(pf, "PROJECT   none: pick a node or study a blueprint", Vector2(6, 4), UiTheme.DUST)
	else:
		make_label(pf, "PROJECT   %s" % String(proj.name).to_upper(), Vector2(6, 4), UiTheme.CYAN)
		var got := int(proj.got)
		var need := maxi(1, int(proj.need))
		make_gauge(pf, "ui.gauge.ooze", Vector2(6, 16), float(got) / float(need), "%d / %d points   %d on Research" % [got, need, int(proj.workers)])
		make_button(pf, "DROP", Vector2(370, 2), func():
			sim.research_drop()
			_draw_research())
	# the picked node's card
	var picked := {}
	for n in nodes:
		if String(n.id) == picked_node:
			picked = n
	if not picked.is_empty():
		var st := String(picked.state)
		make_label(pf, String(picked.name).to_upper(), Vector2(6, 32), UiTheme.ACID if st == "researched" else UiTheme.CHALK)
		make_label(pf, String(picked.text).left(68), Vector2(6, 44), UiTheme.DUST)
		if st == "researched":
			make_label(pf, "researched", Vector2(6, 58), UiTheme.ACID)
		else:
			var cost := "%d points   %d P %d C" % [int(picked.points), int(picked.parts), int(picked.chrome)]
			if bool(picked.paid):
				cost = "%d points   paid" % int(picked.points)
			make_label(pf, cost, Vector2(6, 58), UiTheme.CHALK)
			var why := String(picked.why)
			if why != "":
				make_label(pf, why, Vector2(200, 58), UiTheme.HOT_PINK)
			elif not bool(picked.current):
				var id := String(picked.id)
				make_button(pf, "RESEARCH", Vector2(340, 56), func():
					research_msg = sim.research_pick("node", id)
					_replan()
					_draw_research())
	if research_msg != "":
		make_label(pf, research_msg, Vector2(6, 70), UiTheme.HOT_PINK)
	# the blueprints the Stash holds, and what the Bench has finished
	var bf := make_frame(research_view, Vector2(424, 206), Vector2(208, 84))
	var bps: Array = t.blueprints
	make_label(bf, "BLUEPRINTS   %d held" % bps.size(), Vector2(6, 4), UiTheme.CYAN)
	var y := 16
	for bp in bps:
		if y > 52:
			break
		var why := String(bp.why)
		make_label(bf, String(bp.name).to_upper().left(22), Vector2(6, y), UiTheme.CHALK if why == "" else UiTheme.DUST)
		if why == "":
			var item := String(bp.item)
			make_button(bf, "STUDY", Vector2(160, y - 3), func():
				research_msg = sim.research_pick("study", item)
				_replan()
				_draw_research())
		else:
			y += 10
			make_label(bf, "  " + why, Vector2(6, y), UiTheme.DUST)
		y += 13
	var hist: Array = t.history
	if bps.is_empty():
		make_label(bf, "a blueprint opens its branch's tier", Vector2(6, y), UiTheme.DUST)
		y += 12
	for h in hist:
		if y > 72:
			break
		make_label(bf, String(h).left(32), Vector2(6, y), UiTheme.DUST)
		y += 10

## A project picked opens the Research slots: the shift's plan takes them now, as an order does.
func _replan() -> void:
	if research_msg == "":
		Sim.ensure().refresh_movers(Sim.ensure_people(), Sim.day)

func _node_tile(parent: Control, pos: Vector2, n: Dictionary) -> void:
	var st := String(n.state)
	var f := make_frame(parent, pos, Vector2(NODE_W, NODE_H))
	var col: Color = UiTheme.ACID if st == "researched" else (UiTheme.CHALK if st == "open" else UiTheme.DUST)
	if st == "locked":
		f.modulate = Color(0.62, 0.62, 0.7)
	# the name in lines of ten, the state under it
	var lines: Array = []
	for w in String(n.name).to_upper().split(" "):
		if lines.is_empty() or (String(lines[-1]) + " " + w).length() > 10:
			lines.append(w)
		else:
			lines[-1] = String(lines[-1]) + " " + w
	for k in mini(lines.size(), 3):
		make_label(f, String(lines[k]).left(10), Vector2(3, 1 + k * 9), col)
	var foot := "DONE" if st == "researched" else ("%d PTS" % int(n.points) if st == "open" else "BLUEPRINT")
	if bool(n.current):
		foot = "RESEARCHING"
	make_label(f, foot, Vector2(3, 29), UiTheme.CYAN if bool(n.current) else (UiTheme.ACID if st == "researched" else UiTheme.DUST))
	if String(n.id) == picked_node:
		var edge := ColorRect.new()
		edge.color = UiTheme.CYAN
		edge.position = Vector2(0, NODE_H - 2)
		edge.size = Vector2(NODE_W, 2)
		f.add_child(edge)
	var id := String(n.id)
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.size = f.size
	b.pressed.connect(func():
		picked_node = id
		research_msg = ""
		_draw_research())
	f.add_child(b)

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
		make_label(recipe_list, "TIER %d   %d   CHROME %d" % [tier, n, tier], Vector2(6, y), UiTheme.DUST)
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
	var minutes := int(r.shifts) * 10
	make_label(slots_frame, "%s   tier %d   %d min of a Tweaker" % [String(r.name).to_upper(), int(r.tier), minutes], Vector2(6, 4), UiTheme.CYAN)
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
	make_button(outcome_frame, "CRAFT", Vector2(120, 122), func():
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
		if y > 104:
			make_label(queue_frame, "+%d more" % (jobs.size() - 7), Vector2(6, y), UiTheme.DUST)
			break
		var idx := int(j.index)
		var left := int(j.get("minutes_left", int(j.shifts_left) * 10))
		var parts: PackedStringArray = j.parts
		make_label(queue_frame, "%s   %d min left" % [String(j.name).to_upper(), left], Vector2(6, y), UiTheme.CHALK)
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
	make_label(queue_frame, "a Tweaker in a Bench slot works the first jobs, a minute at a time", Vector2(6, 128), UiTheme.DUST)
