extends ScreenBase
## Colony (Full Design v3.12, Colony mode; UI and screens: Colony HUD): the world at Street / District / Stack zoom
## and the HUD over it — the clock and the city line top-left, the alert queue down the right edge, the tool bar at
## the bottom (Pit, Bait, Salt, Sever, Relay, Build, Zone, Overlay), the minimap of the Stack view bottom-right, the
## selected thing's card above the tool bar and the same card small under the cursor. Every rule is the crates'
## (RoseSim, RoseStash, RosePeople); this screen passes the clicks and draws the answers.

## Street 32 px a tile, District 8, Stack 3: the whole stratum (192 tiles) across the screen, the core's circuit
## board at three pixels a texel (the minimap is the same picture at one)
const ZOOMS := [1.0, 0.25, 3.0 / 32.0]
const VIEW_NAMES := ["STREET", "DISTRICT", "STACK"]
const TOOLS := ["pit", "bait", "salt", "sever", "relay", "build", "zone", "overlay"]
const OVERLAYS := ["", "nutrient", "signal", "light", "zones", "traffic"]
const ZONE_KINDS := ["stockpile", "hab", "quarantine", "kill", "erase"]
const SPEEDS := [2.0, 4.0, 8.0]
const CATS := [["chrome", "CHR"], ["part", "PRT"], ["organic", "ORG"], ["tech", "TEC"], ["gear", "GER"], ["contraband", "CON"], ["key", "KEY"], ["curio", "CUR"], ["relic", "REL"]]
const TIERS := ["COMMON", "UNCOMMON", "RARE", "BLACKLINE", "RELIC"]
const ALERT_ROWS := 12
const ALERT_EVERY := 1.0   # one new line a second at most

var world: WorldView
var cam: Camera2D
var view_index := 0
var clock_label: Label
var city_label: Label
var hint_label: Label
var tool := ""             # "" selects; "build" places Sim.build_tool; "bait", "salt", "sever", "zone"
var tool_buttons := {}
var rotated := false
var zone_kind := "stockpile"
var zone_bar: Panel
var drag_from := Vector2i(-1, -1)
var stroke_last := Vector2i(-1, -1)
var cursor: Node2D
var cursor_tile := Vector2i(-1, -1)
var cursor_reason := ""    # whether the armed def may stand under the cursor
var answer := ""           # the last verb's answer ("" when it went through)
var answer_ok := true
var answer_age := 0.0
const ANSWER_FOR := 4.0    # seconds a verb's answer stands on the hint line
var selected := {}         # {kind "slimo"|"structure"|"tile", id | x, y}
var following := false
var card: Panel
var card_sig := ""
var hover: Panel
var hover_label: Label
var hover_sig := ""
var alert_frame: Panel
var alert_rows: VBoxContainer
var alert_queue: Array = []
var alert_clock := 0.0
var alert_sig := ""
var alerts_now: Array = []
var minimap: TextureRect
var minimap_frame: Control
var minimap_img: Image
var minimap_label: Label
## the stratum the minimap shows: the one the camera is in (Full Design v3.19: the stack)
var mm_stratum := {}
var stack_return := Vector2.ZERO   # where the camera was before the Stack view framed the stratum

func _ready() -> void:
	screen_title = "COLONY"
	super._ready()
	world = WorldView.new()
	add_child(world)
	cam = Camera2D.new()
	add_child(cam)
	cam.make_current()
	if Sim.colony_cam.is_empty():
		var pits: PackedInt32Array = Sim.pits()
		var heart: int = pits[0] if pits.size() > 0 else (world.h / 2) * world.w + world.w / 2
		cam.position = Vector2((heart % world.w) * WorldView.TILE + 16, (heart / world.w) * WorldView.TILE + 16)
	else:
		cam.position = Sim.colony_cam.pos
		view_index = int(Sim.colony_cam.view)
		stack_return = Sim.colony_cam.get("back", Vector2.ZERO)
	cursor = Node2D.new()
	cursor.draw.connect(_draw_cursor)
	add_child(cursor)
	if Sim.build_tool != "":
		tool = "build"
	world.set_overlay(Sim.overlay)
	set_view(view_index)
	Sim.ticked.connect(_on_tick)
	_on_tick()

func _exit_tree() -> void:
	if cam != null:
		Sim.colony_cam = {"pos": cam.position, "view": view_index, "back": stack_return}

func _build_content(c: Control) -> void:
	var top := make_frame(c, Vector2(0, -2), Vector2(632, 26))
	clock_label = make_label(top, "", Vector2(4, 2), UiTheme.CYAN)
	city_label = make_label(top, "", Vector2(4, 13), UiTheme.DUST)
	# the alert queue: right edge, newest on top
	alert_frame = make_frame(c, Vector2(436, 28), Vector2(196, 14))
	alert_rows = VBoxContainer.new()
	alert_rows.position = Vector2(2, 2)
	alert_rows.add_theme_constant_override("separation", 0)
	alert_frame.add_child(alert_rows)
	alert_frame.visible = false
	# the selected thing's card, above the tool bar
	card = make_frame(c, Vector2(0, 168), Vector2(300, 86))
	card.visible = false
	# the zone kinds, while the Zone tool is in hand
	zone_bar = make_frame(c, Vector2(0, 256), Vector2(300, 15))
	var zb := HBoxContainer.new()
	zb.position = Vector2(2, 1)
	zb.add_theme_constant_override("separation", 3)
	zone_bar.add_child(zb)
	for k in ZONE_KINDS:
		var b := Button.new()
		b.text = k.to_upper()
		b.focus_mode = Control.FOCUS_NONE
		b.pressed.connect(func():
			zone_kind = k
			_refresh_tools())
		b.set_meta("kind", k)
		zb.add_child(b)
	zone_bar.visible = false
	# the hint line and the tool bar
	var hint := make_frame(c, Vector2(0, 272), Vector2(432, 15))
	hint_label = make_label(hint, "", Vector2(4, 2), UiTheme.DUST)
	var bar := make_frame(c, Vector2(0, 288), Vector2(432, 18))
	var tools := HBoxContainer.new()
	tools.position = Vector2(3, 1)
	tools.add_theme_constant_override("separation", 3)
	bar.add_child(tools)
	for t in TOOLS:
		var b := Button.new()
		b.text = t.to_upper()
		b.focus_mode = Control.FOCUS_NONE
		b.pressed.connect(func(): pick_tool(t))
		tools.add_child(b)
		tool_buttons[t] = b
	# the minimap: the Stack view at a texel a tile with the camera's frame; click to go there
	var mm := make_frame(c, Vector2(436, 252), Vector2(196, 54))
	minimap = TextureRect.new()
	minimap.position = Vector2(2, 3)
	minimap.size = Vector2(192, 48)
	minimap.stretch_mode = TextureRect.STRETCH_SCALE
	minimap.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	minimap.mouse_filter = Control.MOUSE_FILTER_STOP
	minimap.gui_input.connect(_on_minimap_input)
	mm.add_child(minimap)
	minimap_label = make_label(mm, "", Vector2(4, 4), UiTheme.DUST)
	minimap_label.z_index = 2
	minimap_frame = Control.new()
	minimap_frame.mouse_filter = Control.MOUSE_FILTER_IGNORE
	minimap_frame.size = minimap.size
	minimap_frame.draw.connect(_draw_minimap_frame)
	minimap.add_child(minimap_frame)
	# the small card under the cursor
	hover = make_frame(c, Vector2.ZERO, Vector2(150, 26))
	hover.mouse_filter = Control.MOUSE_FILTER_IGNORE
	hover_label = make_label(hover, "", Vector2(4, 2), UiTheme.CHALK)
	hover.visible = false

# ---------------------------------------------------------------- the clock
func _on_tick() -> void:
	var sim: RoseSim = Sim.ensure()
	if following and selected.get("kind", "") == "slimo":
		var p := _slimo_tile(int(selected.id))
		if p.x >= 0:
			cam.position = Vector2(p.x * WorldView.TILE + 16, p.y * WorldView.TILE + 16)
	_refresh_lines(sim)
	_poll_alerts(sim)
	_paint_minimap(sim)
	_draw_card()
	_refresh_hover()
	world.redraw_city()
	cursor.queue_redraw()

func _process(delta: float) -> void:
	var v := Vector2.ZERO
	if Input.is_key_pressed(KEY_W): v.y -= 1
	if Input.is_key_pressed(KEY_S): v.y += 1
	if Input.is_key_pressed(KEY_A): v.x -= 1
	if Input.is_key_pressed(KEY_D): v.x += 1
	if v != Vector2.ZERO:
		following = false
		cam.position += v.normalized() * 360.0 * delta / cam.zoom.x
		cam.position = cam.position.round()
		minimap_frame.queue_redraw()
	if answer != "":
		answer_age += delta
		if answer_age > ANSWER_FOR:
			answer = ""
			_refresh_hint()
	alert_clock += delta
	if alert_clock >= ALERT_EVERY and not alert_queue.is_empty():
		alert_clock = 0.0
		Sim.alerts_shown.push_front(alert_queue.pop_front())
		_draw_alerts()

func _speed_index() -> int:
	return maxi(0, SPEEDS.find(Sim.tick_hz))

func _refresh_lines(sim: RoseSim) -> void:
	var s: Dictionary = sim.stats()
	var secs := int(s.get("tick", 0)) / 2
	var ov := String(Sim.overlay).to_upper() if Sim.overlay != "" else "none"
	clock_label.text = "DAY %d %s  %d:%02d:%02d   %s %dx   %s   overlay %s   cargo %d riding %d waiting   jam %d   salt bag %d" % [
		Sim.day, ["DIM", "DARK", "FLOOD"][(Sim.day - 1) % 3], secs / 3600, (secs / 60) % 60, secs % 60,
		"PAUSED" if Sim.paused else "RUN", 1 << _speed_index(), VIEW_NAMES[view_index], ov,
		s.get("riding", 0), s.get("waiting", 0), s.get("jammed", 0), int(sim.salt_left())]
	var pop: int = Sim.ensure_people().count()
	var built := 0
	var ghosts := 0
	for st in sim.structures():
		if bool(st.built):
			built += 1
		else:
			ghosts += 1
	var beds: PackedInt32Array = sim.beds()
	var ds: Array = sim.districts(Sim.ensure_people())
	var cuts: int = sim.cuts().size()
	city_label.text = "%s of %d   %d built  %d ghost%s  %d cut%s   beds %d/%d   %d district%s   AUTHORITY %d" % [
		String(sim.city_tier_name(pop)).to_upper(), pop, built, ghosts, "" if ghosts == 1 else "s", cuts, "" if cuts == 1 else "s",
		beds[1], beds[0], ds.size(), "" if ds.size() == 1 else "s", int(sim.authority())]
	_refresh_hint()

# ---------------------------------------------------------------- the tools
func pick_tool(t: String) -> void:
	answer = ""
	match t:
		"pit", "relay":
			Sim.build_tool = t
			tool = "build"
			rotated = false
		"build":
			Screens.show_screen("build")
			return
		"overlay":
			var i := OVERLAYS.find(Sim.overlay)
			Sim.overlay = OVERLAYS[(i + 1) % OVERLAYS.size()]
			world.set_overlay(Sim.overlay)
		_:
			tool = "" if tool == t else t
			Sim.build_tool = ""
	_refresh_tools()

## Put the tool down (right-click with one armed).
func drop_tool() -> void:
	tool = ""
	Sim.build_tool = ""
	drag_from = Vector2i(-1, -1)
	_refresh_tools()

func _refresh_tools() -> void:
	for t in tool_buttons:
		var b: Button = tool_buttons[t]
		var armed: bool = (t == tool) or (tool == "build" and t == Sim.build_tool)
		if t == "overlay":
			b.text = "OVERLAY" if Sim.overlay == "" else Sim.overlay.to_upper()
			armed = Sim.overlay != ""
		b.add_theme_color_override("font_color", UiTheme.ACID if armed else UiTheme.CHALK)
	zone_bar.visible = tool == "zone"
	for b in zone_bar.get_child(0).get_children():
		(b as Button).add_theme_color_override("font_color", UiTheme.ACID if String(b.get_meta("kind")) == zone_kind else UiTheme.CHALK)
	_refresh_hint()
	cursor.queue_redraw()

func _refresh_hint() -> void:
	var line := ""
	match tool:
		"":
			line = "LMB select   RMB order / cancel   SPACE pause  +/- speed   1-9 views (CTRL sets)   F follow   TAB view"
		"build":
			var d := _def(Sim.build_tool)
			line = "%s%s   %s   R turn   RMB put down" % [String(d.get("name", Sim.build_tool)).to_upper(), " (turned)" if rotated else "", cursor_reason if cursor_reason != "" else "costs %s, %d build points" % [String(d.get("cost", {}).get("text", "")), int(d.get("build", 0))]]
		"bait":
			line = "BAIT   the cheapest Common Chrome in the Stash, smeared on the 3x3   RMB put down"
		"salt":
			line = "SALT   %d left in the open bag (a bag of road salt lays 8)   drag for a line   RMB put down" % int(Sim.ensure().salt_left())
		"sever":
			line = "SEVER   mark Mat to cut: 8 Build points at shift end   drag for a line   RMB a mark unmarks"
		"zone":
			line = "ZONE %s   drag a rectangle%s   RMB put down" % [zone_kind.to_upper(), "   (against the selected Rack or Cold Store, else the nearest)" if zone_kind == "stockpile" else ""]
			if zone_kind == "erase":
				line = "ZONE ERASE   click a zone to erase it   RMB put down"
	if answer != "":
		hint_label.text = answer.left(106)
		hint_label.add_theme_color_override("font_color", UiTheme.ACID if answer_ok else UiTheme.HOT_PINK)
	else:
		hint_label.text = line.left(106)
		hint_label.add_theme_color_override("font_color", UiTheme.DUST)

func _say(why: String, ok_text: String = "") -> void:
	answer_age = 0.0
	answer_ok = why == ""
	answer = ok_text if why == "" else why
	_refresh_hint()

func _def(id: String) -> Dictionary:
	for d in Sim.ensure().structure_defs():
		if String(d.id) == id:
			return d
	return {}

func _footprint(d: Dictionary) -> Vector2i:
	var fw := int(d.get("w", 1))
	var fh := int(d.get("h", 1))
	return Vector2i(fh, fw) if rotated else Vector2i(fw, fh)

## The tool at a tile (left-click, or a drag for Salt and Sever).
func apply_tool(tx: int, ty: int) -> void:
	var sim: RoseSim = Sim.ensure()
	match tool:
		"":
			select_at(tx, ty)
		"build":
			place_ghost(tx, ty)
		"bait":
			_say(String(sim.bait_from_stash(tx, ty)), "baited %d,%d" % [tx, ty])
		"salt":
			var why := String(sim.lay_salt(tx, ty))
			_say(why, "salted %d,%d   %d left in the bag" % [tx, ty, int(sim.salt_left())])
			if why == "":
				world.paint_salt(tx, ty)
		"sever":
			_say(String(sim.mark_cut(tx, ty)), "marked %d,%d to be cut" % [tx, ty])
		"zone":
			if zone_kind == "erase":
				var z := int(sim.zone_at(tx, ty))
				_say("" if z >= 0 and sim.erase_zone(z) else "no zone there", "zone erased")
	world.redraw_city()
	cursor.queue_redraw()

## Place the armed def as a ghost; why not goes on the hint line.
func place_ghost(tx: int, ty: int) -> int:
	if Sim.build_tool == "":
		return -1
	var sim: RoseSim = Sim.ensure()
	cursor_tile = Vector2i(tx, ty)
	cursor_reason = String(sim.can_place_turned(Sim.build_tool, tx, ty, rotated))
	if cursor_reason != "":
		_say(cursor_reason)
		return -1
	var id := int(sim.place_turned(Sim.build_tool, tx, ty, rotated))
	_say("" if id >= 0 else "it cannot stand there", "%s placed: Build takes it at shift end once the Stash pays" % String(_def(Sim.build_tool).get("name", "")))
	return id

## A rectangle of zone: a stockpile against the selected store (else the nearest built Rack or Cold Store), the
## other kinds as they are.
func paint_rect(a: Vector2i, b: Vector2i) -> void:
	var sim: RoseSim = Sim.ensure()
	var cells := PackedInt32Array()
	for y in range(mini(a.y, b.y), maxi(a.y, b.y) + 1):
		for x in range(mini(a.x, b.x), maxi(a.x, b.x) + 1):
			if x >= 0 and y >= 0 and x < world.w and y < world.h:
				cells.append(y * world.w + x)
	if cells.is_empty():
		return
	if zone_kind == "stockpile":
		var anchor := _store_for((a + b) / 2)
		if anchor < 0:
			_say("a stockpile hangs on a built Rack or Cold Store: none stands")
			return
		_say(String(sim.paint_stockpile(anchor, cells)), "stockpile painted: it fills at shift end")
		selected = {"kind": "structure", "id": anchor}
	else:
		var z := int(sim.paint_zone(zone_kind, cells, PackedStringArray(), 0, 0, -1))
		_say("" if z >= 0 else "no cell there takes a zone: signalled Mat, or floor beside it", "%s zone painted" % zone_kind)
	world.redraw_city()

func _store_for(at: Vector2i) -> int:
	var sim: RoseSim = Sim.ensure()
	var best := -1
	var best_d := 1 << 30
	for st in sim.structures():
		if not bool(st.built) or not (String(st.def) in ["rack", "cold_store"]):
			continue
		if selected.get("kind", "") == "structure" and int(selected.id) == int(st.id):
			return int(st.id)
		var dd: int = absi(int(st.x) - at.x) + absi(int(st.y) - at.y)
		if dd < best_d:
			best_d = dd
			best = int(st.id)
	return best

# ---------------------------------------------------------------- select, hover, order
func _slimo_tile(id: int) -> Vector2i:
	var sim: RoseSim = Sim.ensure()
	var ids: PackedInt32Array = sim.agent_ids()
	var pos: PackedInt32Array = sim.agent_positions()
	for k in mini(ids.size(), pos.size()):
		if ids[k] == id:
			return Vector2i(pos[k] % world.w, pos[k] / world.w)
	return Vector2i(-1, -1)

func _slimo_at(tx: int, ty: int) -> int:
	if view_index == 2:
		return -1
	var sim: RoseSim = Sim.ensure()
	var ids: PackedInt32Array = sim.agent_ids()
	var pos: PackedInt32Array = sim.agent_positions()
	var i := ty * world.w + tx
	for k in mini(ids.size(), pos.size()):
		if pos[k] == i:
			return ids[k]
	return -1

## What is under a tile: a slimo, else a structure, else a tile of Mat (or of zone, salt or a cut); {} for none.
func thing_at(tx: int, ty: int) -> Dictionary:
	var s := _slimo_at(tx, ty)
	if s >= 0:
		return {"kind": "slimo", "id": s}
	var t: Dictionary = Sim.ensure().tile_info(tx, ty)
	if t.is_empty():
		return {}
	if int(t.structure) >= 0:
		return {"kind": "structure", "id": int(t.structure)}
	if String(t.mat) != "" or int(t.zone) >= 0 or bool(t.salt) or bool(t.cut):
		return {"kind": "tile", "x": tx, "y": ty}
	return {}

func select_at(tx: int, ty: int) -> void:
	var was: Dictionary = selected
	selected = thing_at(tx, ty)
	if selected.get("kind", "") != "slimo" or was.get("id", -2) != selected.get("id", -1):
		following = false
	answer = ""
	card_sig = ""
	_draw_card()

## Right-click: put a tool down; order the selected slimo to the structure under the cursor; else cancel the ghost
## or the cut under it.
func right_click(tx: int, ty: int) -> void:
	var sim: RoseSim = Sim.ensure()
	if tool != "":
		if tool == "sever" and sim.unmark_cut(tx, ty):
			_say("", "unmarked %d,%d" % [tx, ty])
			world.redraw_city()
			return
		drop_tool()
		return
	var t: Dictionary = sim.tile_info(tx, ty)
	if t.is_empty():
		return
	if selected.get("kind", "") == "slimo" and int(t.structure) >= 0:
		var info: Dictionary = sim.structure_info(int(t.structure), Sim.ensure_people().count())
		_say(String(sim.order(int(selected.id), int(t.structure))), "%s works the %s next shift" % [Sim.slimo_name(int(selected.id)), String(info.get("name", ""))])
		card_sig = ""
		_draw_card()
		return
	if bool(t.cut):
		sim.unmark_cut(tx, ty)
		_say("", "unmarked %d,%d" % [tx, ty])
	elif int(t.structure) >= 0:
		var info: Dictionary = sim.structure_info(int(t.structure), 0)
		if not bool(info.get("built", true)):
			_say("" if sim.cancel_ghost(int(t.structure)) else "paid for: the parts are cut and welded, it stays", "ghost cancelled")
	world.redraw_city()

func _refresh_hover() -> void:
	var mp: Vector2 = get_viewport().get_mouse_position()
	var p: Vector2 = get_global_mouse_position()
	var tx := int(floor(p.x / WorldView.TILE))
	var ty := int(floor(p.y / WorldView.TILE))
	var th := thing_at(tx, ty) if tool == "" and _mouse_in_world() else {}
	if th.is_empty() or th == selected:
		hover.visible = false
		return
	var text := _short(th)
	if text == "":
		hover.visible = false
		return
	hover_label.text = text
	var lines := text.count("\n") + 1
	var width := 0
	for l in text.split("\n"):
		width = maxi(width, l.length() * 4)
	hover.size = Vector2(width + 8, lines * 10 + 4)
	var local := mp - Vector2(4, 26) + Vector2(10, 10)
	local.x = minf(local.x, 632 - hover.size.x)
	local.y = minf(local.y, 306 - hover.size.y)
	hover.position = local
	hover.visible = true

func _mouse_in_world() -> bool:
	var mp: Vector2 = get_viewport().get_mouse_position()
	return get_viewport().gui_get_hovered_control() == null and mp.y > 26 and mp.y < 338

## The small card: a line or two of what the full card says.
func _short(th: Dictionary) -> String:
	var sim: RoseSim = Sim.ensure()
	match String(th.kind):
		"slimo":
			var rec: Dictionary = Sim.ensure_people().record(int(th.id))
			if rec.is_empty():
				return ""
			return "%s  %s\n%s" % [String(rec.full_name).to_upper(), String(rec.depth), String(rec.status)]
		"structure":
			var info: Dictionary = sim.structure_info(int(th.id), Sim.ensure_people().count())
			if info.is_empty():
				return ""
			var state := "TIER %d/%d" % [int(info.tier), int(info.tiers)] if bool(info.built) else "GHOST %d/%d" % [int(info.progress), int(info.need)]
			return "%s  %s\n%d at work" % [String(info.name).to_upper(), state, (info.workers as Array).size()]
		"tile":
			var t: Dictionary = sim.tile_info(int(th.x), int(th.y))
			var what := String(t.mat).to_upper() if String(t.mat) != "" else String(t.substrate).to_upper()
			return "%s  signal %.2f  traffic %d%s" % [what, float(t.sigma), int(t.traffic), "  JAM" if bool(t.jam) else ""]
	return ""

# ---------------------------------------------------------------- the card
func _card_info() -> Dictionary:
	var sim: RoseSim = Sim.ensure()
	var people: RosePeople = Sim.ensure_people()
	var kind := String(selected.get("kind", ""))
	var info := {"kind": kind}
	match kind:
		"slimo":
			var id := int(selected.id)
			info.rec = people.record(id)
			info.job = ""
			info.at = ""
			for row in sim.job_grid(people):
				if int(row.id) == id:
					info.job = String(row.job)
					info.at = String(row.structure_name)
			var orders: Dictionary = sim.orders()
			info.ordered = ""
			if orders.has(id):
				info.ordered = String(sim.structure_info(int(orders[id]), 0).get("name", ""))
			info.following = following
		"structure":
			info.s = sim.structure_info(int(selected.id), people.count())
			if info.s.is_empty():
				return {"kind": ""}
			if bool(info.s.quota):
				info.q = sim.quota(int(selected.id))
			if bool(info.s.store) and int(info.s.zone) >= 0:
				for z in sim.zones():
					if int(z.id) == int(info.s.zone):
						info.zone = z
				info.room = {}
				for r in Sim.ensure_stash().rooms():
					if int(r.zone) == int(info.s.zone):
						info.room = r
		"tile":
			info.t = sim.tile_info(int(selected.x), int(selected.y))
	return info

func _draw_card() -> void:
	var info := _card_info()
	var sig := str(info)
	if sig == card_sig:
		return
	card_sig = sig
	for ch in card.get_children():
		ch.queue_free()
	card.visible = String(info.kind) != ""
	if not card.visible:
		return
	make_button(card, "X", Vector2(286, 1), func():
		selected = {}
		following = false
		card_sig = ""
		_draw_card())
	match String(info.kind):
		"slimo": _card_slimo(info)
		"structure": _card_structure(info)
		"tile": _card_tile(info)

func _card_slimo(info: Dictionary) -> void:
	var rec: Dictionary = info.rec
	if rec.is_empty():
		make_label(card, "not on the map: on a run, or gone", Vector2(6, 4), UiTheme.DUST)
		return
	var id := int(rec.id)
	make_label(card, "%s   %s %s" % [String(rec.full_name).to_upper(), String(rec.depth), String(rec.depth_name)], Vector2(6, 4), UiTheme.CYAN)
	make_label(card, String(rec.status), Vector2(6, 15), UiTheme.CHALK)
	var job := "this shift: %s at the %s" % [String(info.job).to_upper(), String(info.at)] if String(info.job) != "" else "this shift: no job"
	make_label(card, job, Vector2(6, 26), UiTheme.CHALK)
	var n: Dictionary = rec.needs
	make_label(card, "FED %d   REST %d   LIT %d   DRY %d   MOOD %d (toward %d)" % [int(n.get("fed", 0)), int(n.get("rest", 0)), int(n.get("lit", 0)), int(n.get("dry", 0)), int(n.get("mood", 0)), int(rec.mood_target)], Vector2(6, 37), UiTheme.DUST)
	if String(info.ordered) != "":
		make_label(card, "next shift: the %s (ordered)" % String(info.ordered), Vector2(6, 50), UiTheme.ACID)
	else:
		make_label(card, "RMB a structure: it works there next shift", Vector2(6, 50), UiTheme.DUST)
	make_button(card, "UNFOLLOW" if bool(info.following) else "FOLLOW (F)", Vector2(6, 68), func():
		following = not following
		card_sig = "")
	make_button(card, "SLIMO CARD", Vector2(90, 68), func():
		Sim.focus_slimo = id
		Screens.show_screen("slimo"))

func _card_structure(info: Dictionary) -> void:
	var s: Dictionary = info.s
	var sim: RoseSim = Sim.ensure()
	var id := int(s.id)
	var state := ""
	if not bool(s.built):
		state = "GHOST   %d/%d build points   %s" % [int(s.progress), int(s.need), "paid" if bool(s.paid) else "waits to be paid"]
	elif int(s.upgrading_to) > 0:
		state = "TIER %d/%d   to tier %d: %d/%d" % [int(s.tier), int(s.tiers), int(s.upgrading_to), int(s.progress), int(s.need)]
	else:
		state = "TIER %d/%d   HP %d/%d" % [int(s.tier), int(s.tiers), int(s.hp), int(s.hp_max)]
	# mending (Full Design v3.17, Building): the builders on a damaged structure this shift
	var menders := 0
	for wk in s.workers:
		if String(wk.job) == "Build":
			menders += 1
	var damaged := bool(s.built) and int(s.upgrading_to) == 0 and int(s.hp) < int(s.hp_max)
	if damaged:
		state += ("   %d MENDING IT" % menders) if menders > 0 else "   NO BUILDER FREE"
	make_label(card, String(s.name).to_upper(), Vector2(6, 4), UiTheme.CYAN)
	make_label(card, state, Vector2(110, 4), (UiTheme.ACID if menders > 0 else UiTheme.HOT_PINK) if damaged else UiTheme.CHALK)
	var sig_line := ""
	if float(s.sigma_need) > 0.0:
		sig_line = "signal %.2f of %.2f" % [float(s.sigma), float(s.sigma_need)]
	if bool(s.get("dark", false)):
		sig_line = "DARK: NO POWER, NO WORK   " + sig_line
	elif bool(s.get("drowned", false)):
		sig_line = "DROWNED: NO WORK THIS SHIFT   " + sig_line
	var slots := ""
	if String(s.hosts) != "":
		var names := []
		for wk in s.workers:
			if String(wk.job).to_lower() == String(s.hosts).to_lower():
				names.append(Sim.slimo_name(int(wk.slimo)))
		slots = "%s %d slot%s: %s" % [String(s.hosts).to_upper(), int(s.slots), "" if int(s.slots) == 1 else "s", ", ".join(names) if not names.is_empty() else "no one this shift"]
	make_label(card, ("%s   %s" % [slots, sig_line]).strip_edges().left(72), Vector2(6, 15), UiTheme.HOT_PINK if sig_line != "" and float(s.sigma) < float(s.sigma_need) else UiTheme.DUST)
	if info.has("q") and not (info.q as Dictionary).is_empty():
		_card_quota(id, info.q)
	elif bool(s.store):
		_card_stockpile(info)
	elif String(s.def) == "shrine" and bool(s.built):
		_card_nerves(id)
	# the upgrade, the ghost's cancel
	if not bool(s.built):
		if not bool(s.paid):
			make_button(card, "CANCEL GHOST", Vector2(6, 68), func():
				_say("" if sim.cancel_ghost(id) else "paid for: it stays", "ghost cancelled")
				selected = {}
				card_sig = ""
				world.redraw_city())
	elif int(s.tier) < int(s.tiers) and int(s.upgrading_to) == 0:
		if String(s.upgrade) == "":
			make_button(card, "UPGRADE  %s" % String(s.upgrade_cost), Vector2(6, 68), func():
				_say(String(sim.upgrade(id, Sim.ensure_people().count())), "the next tier's ghost stands over it")
				card_sig = ""
				world.redraw_city())
		else:
			make_label(card, "upgrade: %s" % String(s.upgrade), Vector2(6, 72), UiTheme.DUST)
	# the organ under it: the Mat's own, or a housing's (Full Design v3.18, Organs)
	var t: Dictionary = sim.tile_info(int(s.x), int(s.y))
	if String(t.get("organ", "")) != "" and not info.has("q") and not bool(s.store):
		_organ_block(t, 26)

## A Pit's or Still's quota (Full Design v3.11, Quotas), on its card.
func _card_quota(id: int, q: Dictionary) -> void:
	var sim: RoseSim = Sim.ensure()
	var pit := String(q.kind) == "pit"
	var keep := "KEEP %.1f nutrient" % float(q.target) if pit else "KEEP %d Enzyme Parts" % int(q.target)
	var have := "stock %.2f riding %.2f" % [float(q.stock), float(q.riding)] if pit else "store %d riding %d" % [int(q.stock), int(q.riding)]
	make_label(card, "%s   %s   %s%s" % [keep, String(q.from).to_upper(), have, "" if bool(q.reached) else "   NOT ON THE MAT"], Vector2(6, 28), UiTheme.ACID if float(q.target) > 0.0 else UiTheme.DUST)
	var step := 0.5 if pit else 1.0
	var cap: float = float(q.cap) if pit else 99.0
	make_button(card, "-", Vector2(6, 42), func():
		sim.set_quota(id, maxf(0.0, float(q.target) - step), int(q.ceiling))
		card_sig = "")
	make_button(card, "+", Vector2(22, 42), func():
		sim.set_quota(id, minf(cap, float(q.target) + step), int(q.ceiling))
		card_sig = "")
	if pit:
		make_button(card, "CEILING %s" % String(q.ceiling_name).to_upper(), Vector2(40, 42), func():
			sim.set_quota(id, float(q.target), (int(q.ceiling) + 1) % 5)
			card_sig = "")
	make_button(card, "DEFAULT", Vector2(140 if pit else 40, 42), func():
		sim.clear_quota(id)
		card_sig = "")
	make_label(card, "pulled from the Stash at shift end; cap %.0f" % cap if pit else "pulled from the Stash at shift end", Vector2(6, 57), UiTheme.DUST)

## A stockpile's priority and filter (Stockpiles), and what it holds.
func _card_stockpile(info: Dictionary) -> void:
	var sim: RoseSim = Sim.ensure()
	if not info.has("zone"):
		make_label(card, "no stockpile yet: ZONE > STOCKPILE, drag beside it", Vector2(6, 28), UiTheme.DUST)
		return
	var z: Dictionary = info.zone
	var r: Dictionary = info.room
	var zid := int(z.id)
	var pri := int(z.priority)
	var held := "%d on its cells   %d waiting for a hauler   cap %d" % [int(r.count) - int(r.waiting), int(r.waiting), int(r.cap)] if not r.is_empty() else "fills at shift end"
	make_label(card, "STOCKPILE   %d cells   %s" % [(z.cells as PackedInt32Array).size(), held], Vector2(6, 28), UiTheme.CHALK)
	make_label(card, "PRIORITY %d   %s" % [pri, "draws from the Heart" if pri >= 3 else "takes the Heart's overflow"], Vector2(6, 39), UiTheme.ACID)
	make_button(card, "-", Vector2(180, 36), func():
		sim.set_zone_priority(zid, maxi(1, pri - 1))
		card_sig = "")
	make_button(card, "+", Vector2(196, 36), func():
		sim.set_zone_priority(zid, mini(4, pri + 1))
		card_sig = "")
	var cats: PackedStringArray = z.categories
	var x := 6
	for c in CATS:
		var on := cats.has(c[0])
		var b := make_button(card, c[1], Vector2(x, 52), func():
			var next := PackedStringArray(cats)
			if next.has(c[0]):
				next.remove_at(next.find(c[0]))
			else:
				next.append(c[0])
			sim.set_zone_filter(zid, next, int(z.min_tier), int(z.min_condition))
			card_sig = "")
		b.add_theme_color_override("font_color", UiTheme.ACID if on else UiTheme.DUST)
		x += 22
	make_label(card, "any" if cats.is_empty() else "", Vector2(x + 2, 55), UiTheme.ACID)
	make_button(card, "TIER %s+" % TIERS[int(z.min_tier)], Vector2(x + 18, 52), func():
		sim.set_zone_filter(zid, cats, (int(z.min_tier) + 1) % 5, int(z.min_condition))
		card_sig = "")

## The organs that keep a rule and its settings by number (Pass 2's defaults first).
const ORGAN_RULES := {
	"bulb": {"names": ["DARK OVER ATTENTION 50", "DARK OVER 25", "DARK OVER 75", "NEVER DARK"]},
	"sphincter": {"names": ["SHUT AT LIGHT IN 8", "SHUT AT LIGHT IN 4", "SHUT AT LIGHT IN 12", "ALWAYS SHUT", "ALWAYS OPEN"]},
	"sac": {"names": ["RELEASE", "HOLD"]},
}

## A clickable word on a card: a label with a flat button over it.
func _word(text: String, pos: Vector2, color: Color, on_press: Callable) -> void:
	var l := make_label(card, text, pos, color)
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.position = pos
	b.size = Vector2(text.length() * 4 + 4, 10)
	b.pressed.connect(on_press)
	card.add_child(b)
	l.z_index = 1

## A Relay Shrine's four Nerve rules (Full Design v3.18, Nerve logic): when [reading] [< or >] [value] then
## [action]; each word cycles on a click, X clears a rule, + RULE adds one.
func _card_nerves(id: int) -> void:
	var sim: RoseSim = Sim.ensure()
	var n: Dictionary = sim.nerve_rules(id)
	if not bool(n.researched):
		make_label(card, "NERVE RULES: research Nerve logic at the Bench", Vector2(6, 28), UiTheme.DUST)
		return
	var rules: Array = n.rules
	var steps: Array = n.steps
	var y := 26
	for k in rules.size():
		var r: Dictionary = rules[k]
		var kk := k
		var rd := int(r.reading)
		var vals: PackedFloat32Array = steps[rd]
		var set_rule := func(reading: int, above: bool, value: float, action: int):
			var why: String = sim.set_nerve_rule(id, kk, reading, above, value, action)
			_say(why, "")
			card_sig = ""
		make_label(card, "%d WHEN" % (k + 1), Vector2(6, y), UiTheme.DUST)
		_word(String(r.reading_name).to_upper(), Vector2(44, y), UiTheme.CYAN, func():
			var nr := (rd + 1) % 7
			var nv: PackedFloat32Array = steps[nr]
			set_rule.call(nr, bool(r.above), nv[nv.size() / 2], int(r.action)))
		_word(">" if bool(r.above) else "<", Vector2(112, y), UiTheme.CYAN, func(): set_rule.call(rd, not bool(r.above), float(r.value), int(r.action)))
		_word(("%.1f" % float(r.value)) if float(r.value) < 1.01 and rd <= 1 else str(int(r.value)), Vector2(124, y), UiTheme.CYAN, func():
			var at := 0
			for j in vals.size():
				if absf(vals[j] - float(r.value)) < 0.001:
					at = j
			set_rule.call(rd, bool(r.above), vals[(at + 1) % vals.size()], int(r.action)))
		make_label(card, "THEN", Vector2(152, y), UiTheme.DUST)
		_word(String(r.action_name).to_upper(), Vector2(176, y), UiTheme.ACID, func(): set_rule.call(rd, bool(r.above), float(r.value), (int(r.action) + 1) % 7))
		make_label(card, "HOLDS" if bool(r.held) else "", Vector2(236, y), UiTheme.OOZE_GLOW)
		_word("X", Vector2(286, y), UiTheme.HOT_PINK, func(): set_rule.call(-1, false, 0.0, 0))
		y += 10
	if rules.size() < 4:
		_word("+ RULE", Vector2(6, y), UiTheme.ACID, func():
			sim.set_nerve_rule(id, rules.size(), 0, false, 0.3, 1)
			card_sig = "")

## An organ on a card: what it does by itself, and its rule once Organ rules stands (Full Design v3.18, Organs).
func _organ_block(t: Dictionary, top: int) -> void:
	var sim: RoseSim = Sim.ensure()
	var x := int(t.x)
	var y := int(t.y)
	make_label(card, String(t.organ).to_upper() + ("   DARK" if bool(t.get("organ_dark", false)) else ""), Vector2(6, top), UiTheme.OOZE_GLOW)
	# what it does, in two lines broken at a space
	var text := String(t.organ_text)
	var cut := text.length()
	if cut > 64:
		cut = text.rfind(" ", 64)
	make_label(card, text.left(cut), Vector2(6, top + 10), UiTheme.DUST)
	if cut < text.length():
		make_label(card, text.substr(cut + 1).left(64), Vector2(6, top + 19), UiTheme.DUST)
	top += 8
	var rules: Dictionary = ORGAN_RULES.get(String(t.organ_id), {})
	if rules.is_empty():
		return
	var r := int(t.organ_rule)
	var names: Array = rules.names
	var line := "RULE: %s" % names[mini(r, names.size() - 1)]
	if bool(t.organ_rules):
		make_button(card, line, Vector2(6, top + 21), func():
			sim.organ_rule(y * sim.width() + x, (r + 1) % names.size())
			card_sig = ""
			world.redraw_city())
	else:
		make_label(card, line + "   (Organ rules sets it)", Vector2(6, top + 22), UiTheme.DUST)

func _card_tile(info: Dictionary) -> void:
	var t: Dictionary = info.t
	if t.is_empty():
		return
	var sim: RoseSim = Sim.ensure()
	var x := int(t.x)
	var y := int(t.y)
	var what := String(t.mat).to_upper() if String(t.mat) != "" else "NO MAT"
	make_label(card, "%s at %d,%d   on %s" % [what, x, y, String(t.substrate)], Vector2(6, 4), UiTheme.CYAN)
	make_label(card, "signal %.2f   nutrient %.2f   traffic %d%s" % [float(t.sigma), float(t.nutrient), int(t.traffic), "   JAMMED" if bool(t.jam) else ""], Vector2(6, 15), UiTheme.HOT_PINK if bool(t.jam) else UiTheme.CHALK)
	var marks := []
	if int(t.zone) >= 0:
		marks.append("%s zone %d" % [String(t.zone_kind), int(t.zone)])
	if bool(t.salt):
		marks.append("salted")
	if bool(t.cut):
		marks.append("marked to be cut")
	make_label(card, "   ".join(marks), Vector2(6, 26), UiTheme.DUST)
	# a crossing between strata: its slab, and the breach (Full Design v3.19, The stack and the breach)
	for cr in sim.crossings():
		if int(cr.y) == y and (int(cr.x) == x or int(cr.x) + 1 == x):
			_crossing_block(cr)
			return
	if String(t.get("organ", "")) != "":
		_organ_block(t, 37)
	# the handoff (Full Design v3.18): the Mat's proposed bypass, confirmed or severed here
	if bool(t.get("proposal", false)):
		if bool(t.proposal_confirmed):
			make_label(card, "THE MAT LAYS ITS BYPASS: %d TILES TO GO" % int(t.proposal_tiles), Vector2(6, 37), UiTheme.OOZE_GLOW)
		else:
			var n := int(t.proposal_tiles)
			make_label(card, "THE MAT PROPOSES A CORD OF %d TILE%s ROUND THE JAM" % [n, "" if n == 1 else "S"], Vector2(6, 37), UiTheme.OOZE_GLOW)
			make_button(card, "CONFIRM", Vector2(6, 50), func():
				sim.answer_proposal(true)
				card_sig = ""
				world.redraw_city())
			make_button(card, "SEVER", Vector2(70, 50), func():
				sim.answer_proposal(false)
				card_sig = ""
				world.redraw_city())
	if bool(t.cut):
		make_button(card, "UNMARK", Vector2(6, 68), func():
			sim.unmark_cut(x, y)
			card_sig = ""
			world.redraw_city())
	if int(t.zone) >= 0:
		make_button(card, "ERASE ZONE", Vector2(60, 68), func():
			sim.erase_zone(int(t.zone))
			card_sig = ""
			world.redraw_city())

## A slab's card: where it leads, how it opens; OPEN spends the stratum's grate key.
func _crossing_block(cr: Dictionary) -> void:
	var sim: RoseSim = Sim.ensure()
	var above := int(cr.below) + 1
	var name := ""
	for st in sim.strata():
		if int(st.n) == above:
			name = String(st.name).to_upper()
	if not bool(cr.sealed):
		make_label(card, "THE WAY INTO S%d %s IS OPEN" % [above, name], Vector2(6, 37), UiTheme.OOZE_GLOW)
		return
	make_label(card, "SLAB: THE WAY INTO S%d %s" % [above, name], Vector2(6, 37), UiTheme.ACID)
	make_label(card, "a Relay within 4 tiles breaks it at the shift's end", Vector2(6, 48), UiTheme.CHALK)
	if String(cr.key) == "":
		return
	if bool(cr.has_key):
		make_button(card, "OPEN WITH THE %s" % String(cr.key_name).to_upper(), Vector2(6, 62), func():
			var why: String = sim.open_crossing(int(cr.tile))
			if why != "":
				push_warning(why)
			card_sig = ""
			world.redraw_city())
	else:
		make_label(card, "or the %s, which the Stash has not" % String(cr.key_name), Vector2(6, 59), UiTheme.DUST)

# ---------------------------------------------------------------- the alerts
func _level_color(level: String) -> Color:
	match level:
		"red": return UiTheme.HOT_PINK
		"yellow": return UiTheme.ACID
	return UiTheme.VESTMENT

## The crates' alerts: a new one joins the queue (one line a second), one whose cause has gone clears.
func _poll_alerts(sim: RoseSim) -> void:
	alerts_now = sim.alerts(Sim.ensure_people())
	var keys := {}
	for a in alerts_now:
		keys[String(a.key)] = a
	var shown: Array = Sim.alerts_shown.filter(func(k): return keys.has(k))
	Sim.alerts_shown = shown
	alert_queue = alert_queue.filter(func(k): return keys.has(k))
	for a in alerts_now:
		var k := String(a.key)
		if not shown.has(k) and not alert_queue.has(k):
			alert_queue.append(k)
	_draw_alerts()

func _draw_alerts() -> void:
	var by_key := {}
	for a in alerts_now:
		by_key[String(a.key)] = a
	var rows: Array = []
	for k in Sim.alerts_shown:
		if by_key.has(k) and rows.size() < ALERT_ROWS:
			rows.append(by_key[k])
	var sig := str(rows)
	if sig == alert_sig:
		return
	alert_sig = sig
	for ch in alert_rows.get_children():
		ch.queue_free()
	alert_frame.visible = not rows.is_empty()
	alert_frame.size = Vector2(196, rows.size() * 11 + 4)
	for a in rows:
		var b := Button.new()
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.clip_text = true
		b.custom_minimum_size = Vector2(192, 11)
		b.text = String(a.text)
		b.tooltip_text = String(a.text)
		var col := _level_color(String(a.level))
		b.add_theme_color_override("font_color", col)
		b.add_theme_color_override("font_hover_color", UiTheme.VESTMENT)
		var x := int(a.x)
		var y := int(a.y)
		b.pressed.connect(func():
			if x >= 0:
				following = false
				if view_index == 2:
					set_view(0)
				cam.position = Vector2(x * WorldView.TILE + 16, y * WorldView.TILE + 16)
				minimap_frame.queue_redraw())
		alert_rows.add_child(b)

# ---------------------------------------------------------------- the minimap
## The stratum the camera is in, held to the strata reached.
func _camera_stratum() -> Dictionary:
	var row := int(cam.position.y / WorldView.TILE)
	var best := {}
	for st in world.strata_info:
		if not bool(st.reached):
			continue
		best = st if best.is_empty() else best
		if row >= int(st.ceiling) and row < int(st.ceiling) + int(st.rows):
			return st
		if int(st.ceiling) < int(best.ceiling):
			best = st
	return best

func _paint_minimap(sim: RoseSim) -> void:
	var rgb: PackedByteArray = sim.stack_image()
	if rgb.size() != world.w * world.h * 3:
		return
	mm_stratum = _camera_stratum()
	var y0 := int(mm_stratum.get("ceiling", 0))
	var rows := int(mm_stratum.get("rows", world.h))
	var whole := Image.create_from_data(world.w, world.h, false, Image.FORMAT_RGB8, rgb)
	minimap_img = whole.get_region(Rect2i(0, y0, world.w, mini(rows, world.h - y0)))
	if minimap.texture == null or (minimap.texture as ImageTexture).get_height() != minimap_img.get_height():
		minimap.texture = ImageTexture.create_from_image(minimap_img)
	else:
		(minimap.texture as ImageTexture).update(minimap_img)
	minimap_label.text = "S%d %s" % [int(mm_stratum.get("n", 1)), String(mm_stratum.get("short", ""))]
	minimap_frame.queue_redraw()

func _draw_minimap_frame() -> void:
	var half: Vector2 = get_viewport_rect().size * 0.5 / cam.zoom.x / WorldView.TILE
	var y0 := float(mm_stratum.get("ceiling", 0))
	var rows := float(mm_stratum.get("rows", world.h))
	var c: Vector2 = cam.position / WorldView.TILE - Vector2(0, y0)
	var k: Vector2 = minimap.size / Vector2(maxi(1, world.w), maxf(1.0, rows))
	var r := Rect2((c - half) * k, half * 2.0 * k)
	r = r.intersection(Rect2(Vector2.ZERO, minimap.size))
	minimap_frame.draw_rect(r, UiTheme.CYAN, false, 1.0)

func _on_minimap_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var rows := float(mm_stratum.get("rows", world.h))
		var k: Vector2 = Vector2(world.w, rows) / minimap.size
		var t: Vector2 = (event.position * k).floor() + Vector2(0, float(mm_stratum.get("ceiling", 0)))
		following = false
		if view_index == 2:
			set_view(0)
		cam.position = t * WorldView.TILE + Vector2(16, 16)
		minimap_frame.queue_redraw()

# ---------------------------------------------------------------- the world under the cursor
func _draw_cursor() -> void:
	var T := WorldView.TILE
	# the selection
	match String(selected.get("kind", "")):
		"structure":
			for st in Sim.ensure().structures():
				if int(st.id) == int(selected.id):
					cursor.draw_rect(Rect2(int(st.x) * T - 2, int(st.y) * T - 2, int(st.w) * T + 4, int(st.h) * T + 4), UiTheme.CYAN, false, 2.0)
		"tile":
			cursor.draw_rect(Rect2(int(selected.x) * T, int(selected.y) * T, T, T), UiTheme.CYAN, false, 2.0)
		"slimo":
			var p := _slimo_tile(int(selected.id))
			if p.x >= 0:
				cursor.draw_rect(Rect2(p.x * T - 2, p.y * T + T - 4, T + 4, 4), UiTheme.CYAN)
			var orders: Dictionary = Sim.ensure().orders()
			if orders.has(int(selected.id)):
				for st in Sim.ensure().structures():
					if int(st.id) == int(orders[int(selected.id)]):
						cursor.draw_rect(Rect2(int(st.x) * T - 2, int(st.y) * T - 2, int(st.w) * T + 4, int(st.h) * T + 4), UiTheme.ACID, false, 2.0)
	# the ghost under the cursor, Cyan where it may stand, Hot Pink where it may not
	if tool == "build" and Sim.build_tool != "" and cursor_tile.x >= 0:
		var f := _footprint(_def(Sim.build_tool))
		var ok := cursor_reason == ""
		var r := Rect2(cursor_tile.x * T, cursor_tile.y * T, f.x * T, f.y * T)
		cursor.draw_rect(r, Color(UiTheme.CYAN if ok else UiTheme.HOT_PINK, 0.25))
		cursor.draw_rect(r, UiTheme.CYAN if ok else UiTheme.HOT_PINK, false, 2.0)
	# the zone being dragged
	if tool == "zone" and drag_from.x >= 0 and cursor_tile.x >= 0:
		var a := Vector2i(mini(drag_from.x, cursor_tile.x), mini(drag_from.y, cursor_tile.y))
		var b := Vector2i(maxi(drag_from.x, cursor_tile.x), maxi(drag_from.y, cursor_tile.y))
		var col: Color = WorldView.ZONE_COLORS.get(zone_kind, UiTheme.CYAN)
		var r := Rect2(a.x * T, a.y * T, (b.x - a.x + 1) * T, (b.y - a.y + 1) * T)
		cursor.draw_rect(r, Color(col, 0.2))
		cursor.draw_rect(r, col, false, 2.0)

# ---------------------------------------------------------------- views, keys, the mouse
## Street, District or Stack. The Stack view frames the strata the city has reached; leaving it returns to where the camera was.
func set_view(i: int) -> void:
	var to := clampi(i, 0, ZOOMS.size() - 1)
	if to == 2 and view_index != 2:
		stack_return = cam.position
		# the strata the city has reached, framed (Full Design v3.19: the Stack view is the stack)
		cam.position = Vector2(world.w / 2.0, (world.top_row + world.h) / 2.0) * WorldView.TILE
	elif to != 2 and view_index == 2 and stack_return != Vector2.ZERO:
		cam.position = stack_return
	view_index = to
	cam.zoom = Vector2.ONE * ZOOMS[view_index]
	world.draw_crowd = view_index < 2
	world.show_districts = view_index >= 1
	world.set_view_level(view_index)
	minimap_frame.queue_redraw()

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		var k: int = event.keycode
		if k >= KEY_1 and k <= KEY_9:
			var n := k - KEY_0
			if event.ctrl_pressed:
				Sim.bookmarks[n] = {"pos": cam.position, "view": view_index}
				_say("", "view %d set" % n)
			elif Sim.bookmarks.has(n):
				following = false
				cam.position = Sim.bookmarks[n].pos
				set_view(int(Sim.bookmarks[n].view))
		match k:
			KEY_TAB: set_view((view_index + 1) % ZOOMS.size())
			KEY_SPACE: Sim.paused = not Sim.paused
			KEY_EQUAL, KEY_PLUS, KEY_KP_ADD: Sim.set_speed(SPEEDS[mini(SPEEDS.size() - 1, _speed_index() + 1)])
			KEY_MINUS, KEY_KP_SUBTRACT: Sim.set_speed(SPEEDS[maxi(0, _speed_index() - 1)])
			KEY_F:
				if selected.get("kind", "") == "slimo":
					following = not following
					card_sig = ""
			KEY_R:
				if tool == "build":
					rotated = not rotated
					_hover_tile(cursor_tile.x, cursor_tile.y)
		_refresh_lines(Sim.ensure())
		minimap_frame.queue_redraw()
	if event is InputEventMouseMotion:
		var p: Vector2 = get_global_mouse_position()
		var t := Vector2i(int(floor(p.x / WorldView.TILE)), int(floor(p.y / WorldView.TILE)))
		if t != cursor_tile:
			_hover_tile(t.x, t.y)
			# Salt and Sever lay a line while the button is held
			if (event.button_mask & MOUSE_BUTTON_MASK_LEFT) and tool in ["salt", "sever"] and t != stroke_last:
				stroke_last = t
				apply_tool(t.x, t.y)
		_refresh_hover()
	if event is InputEventMouseButton:
		var p: Vector2 = get_global_mouse_position()
		var tx := int(floor(p.x / WorldView.TILE))
		var ty := int(floor(p.y / WorldView.TILE))
		if event.button_index == MOUSE_BUTTON_LEFT:
			if event.pressed:
				if tool == "zone" and zone_kind != "erase":
					drag_from = Vector2i(tx, ty)
				else:
					stroke_last = Vector2i(tx, ty)
					apply_tool(tx, ty)
			elif tool == "zone" and drag_from.x >= 0:
				paint_rect(drag_from, Vector2i(tx, ty))
				drag_from = Vector2i(-1, -1)
			cursor.queue_redraw()
		elif event.pressed and event.button_index == MOUSE_BUTTON_RIGHT:
			right_click(tx, ty)
		elif event.pressed and event.button_index == MOUSE_BUTTON_WHEEL_UP:
			set_view(view_index - 1)
		elif event.pressed and event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			set_view(view_index + 1)
		_draw_card()

## The cursor's tile: with Build armed, whether the def may stand there (turned with R).
func _hover_tile(tx: int, ty: int) -> void:
	cursor_tile = Vector2i(tx, ty)
	if tool == "build":
		cursor_reason = String(Sim.ensure().can_place_turned(Sim.build_tool, tx, ty, rotated)) if Sim.build_tool != "" else "nothing to place"
		if answer != "" and answer_ok:
			answer = ""
		_refresh_hint()
	cursor.queue_redraw()
