extends ScreenBase
## Colony: the world at Street / District / Stack zoom, the clock, the tools, and the city on it (Full Design v3.9,
## v3.11): structures and ghosts drawn on their footprints, the build tool placing the def the Build menu armed (the
## ghost cursor is Cyan where it may stand, Hot Pink with the reason where it may not), districts named in District
## view, the roster and the cargo on the Mat, the Stack view's traffic. Click a Pit or a Still for its quota.

const ZOOMS := [1.0, 0.25, 1.0 / 32.0]
const VIEW_NAMES := ["STREET", "DISTRICT", "STACK"]

var world: WorldView
var cam: Camera2D
var view_index := 0
var stats_label: Label
var city_label: Label
var tool := "select"
var quota_panel: Panel
var quota_id := -1
var cursor: Node2D
var cursor_tile := Vector2i(-1, -1)
var cursor_reason := ""

func _ready() -> void:
	screen_title = "COLONY"
	super._ready()
	world = WorldView.new()
	add_child(world)
	cam = Camera2D.new()
	cam.position = Vector2(world.w * WorldView.TILE / 2.0, world.h * WorldView.TILE / 2.0)
	cam.zoom = Vector2.ONE * ZOOMS[view_index]
	add_child(cam)
	cam.make_current()
	cursor = Node2D.new()
	cursor.draw.connect(_draw_cursor)
	add_child(cursor)
	if Sim.build_tool != "":
		tool = "build"
	Sim.ticked.connect(_refresh)
	_refresh()

func _build_content(c: Control) -> void:
	# the clock and the city lines in a frame, readable over any tile
	var top := make_frame(c, Vector2(0, -2), Vector2(632, 26))
	stats_label = make_label(top, "", Vector2(4, 2), UiTheme.CYAN)
	city_label = make_label(top, "", Vector2(4, 13), UiTheme.DUST)
	make_label(top, "ATTENTION", Vector2(520, 2), UiTheme.DUST)
	make_gauge(top, "ui.gauge.attention", Vector2(528, 14), 0.3, "")
	var bar := make_frame(c, Vector2(0, 286), Vector2(632, 20))
	var tools := HBoxContainer.new()
	tools.position = Vector2(4, 2)
	tools.add_theme_constant_override("separation", 4)
	bar.add_child(tools)
	for t in ["select", "bait", "salt", "sever", "relay", "build"]:
		var b := Button.new()
		b.text = t.to_upper()
		b.focus_mode = Control.FOCUS_NONE
		b.pressed.connect(func():
			tool = t
			cursor_reason = ""
			if t == "build" and Sim.build_tool == "":
				Screens.show_screen("build")
			_refresh())
		tools.add_child(b)
	var hint := Label.new()
	hint.text = "  LMB apply   RMB cancel ghost   TAB view   SPACE pause   1/2/3 speed   WASD pan"
	hint.add_theme_color_override("font_color", UiTheme.DUST)
	tools.add_child(hint)
	# a Pit's or Still's quota (Full Design v3.11, Quotas): opened by clicking one with SELECT
	quota_panel = make_frame(c, Vector2(412, 190), Vector2(220, 92))
	quota_panel.visible = false


func _refresh() -> void:
	var sim: RoseSim = Sim.ensure()
	var s: Dictionary = sim.stats()
	stats_label.text = "t %d  %s  %s  %.0f Hz   cells %d  nodes %d   slimos %d  at work %d   cargo riding %d waiting %d  landed %d  jam %d" % [
		s.get("tick", 0), VIEW_NAMES[view_index], "PAUSED" if Sim.paused else "RUN", Sim.tick_hz,
		s.get("cells", 0), s.get("nodes", 0), s.get("agents", 0), s.get("carrying", 0), s.get("riding", 0), s.get("waiting", 0), s.get("deliveries", 0), s.get("jammed", 0)]
	# the city line: tier, structures and ghosts, beds, districts, the tool in hand
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
	var line := "%s  day %d %s   %d built  %d ghost%s   beds %d/%d   %d district%s" % [
		String(sim.city_tier_name(pop)).to_upper(), Sim.day, ["DIM", "DARK", "FLOOD"][(Sim.day - 1) % 3], built, ghosts, "" if ghosts == 1 else "s", beds[1], beds[0], ds.size(), "" if ds.size() == 1 else "s"]
	if tool == "build":
		line += "   %s: %s" % [Sim.build_tool.to_upper() if Sim.build_tool != "" else "BUILD", cursor_reason if cursor_reason != "" else ("click to place" if Sim.build_tool != "" else "pick one on BUILD")]
	else:
		line += "   tool %s%s" % [tool.to_upper(), (": " + cursor_reason) if cursor_reason != "" else ""]
	var waiting: Array = Sim.last_shift.get("waiting", [])
	if waiting.size() > 0:
		line += "   %s waits for %s" % [String(waiting[0].name), String(waiting[0].cost.text)]
	city_label.text = line.left(128)
	world.redraw_city()
	cursor.queue_redraw()
	if quota_panel.visible:
		_draw_quota()

## The quota card: what the consumer keeps, whose rule it is, what it has and what is riding to it, the tier ceiling.
func open_quota(sid: int) -> void:
	quota_id = sid
	quota_panel.visible = sid >= 0
	if sid >= 0:
		_draw_quota()

func _draw_quota() -> void:
	for ch in quota_panel.get_children():
		ch.queue_free()
	var q: Dictionary = Sim.ensure().quota(quota_id)
	if q.is_empty():
		quota_panel.visible = false
		return
	var pit := String(q.kind) == "pit"
	make_label(quota_panel, String(q.name).to_upper(), Vector2(6, 4), UiTheme.CYAN)
	make_button(quota_panel, "X", Vector2(198, 1), func(): open_quota(-1))
	var keep := "KEEP %.1f nutrient" % float(q.target) if pit else "KEEP %d Enzyme Parts" % int(q.target)
	make_label(quota_panel, "%s   %s" % [keep, String(q.from).to_upper()], Vector2(6, 18), UiTheme.ACID if float(q.target) > 0.0 else UiTheme.DUST)
	var have := "stock %.2f  riding %.2f" % [float(q.stock), float(q.riding)] if pit else "store %d  riding %d" % [int(q.stock), int(q.riding)]
	make_label(quota_panel, have + ("" if bool(q.reached) else "   NOT ON THE MAT"), Vector2(6, 30), UiTheme.DUST if bool(q.reached) else UiTheme.HOT_PINK)
	if pit:
		make_label(quota_panel, "CEILING %s   cap %.0f" % [String(q.ceiling_name), float(q.cap)], Vector2(6, 42), UiTheme.DUST)
	var step := 0.5 if pit else 1.0
	var cap: float = float(q.cap) if pit else 99.0
	make_button(quota_panel, "-", Vector2(6, 58), func():
		Sim.ensure().set_quota(quota_id, maxf(0.0, float(q.target) - step), int(q.ceiling))
		_draw_quota())
	make_button(quota_panel, "+", Vector2(26, 58), func():
		Sim.ensure().set_quota(quota_id, minf(cap, float(q.target) + step), int(q.ceiling))
		_draw_quota())
	if pit:
		make_button(quota_panel, "TIER", Vector2(46, 58), func():
			Sim.ensure().set_quota(quota_id, float(q.target), (int(q.ceiling) + 1) % 5)
			_draw_quota())
	make_button(quota_panel, "DEFAULT", Vector2(90 if pit else 46, 58), func():
		Sim.ensure().clear_quota(quota_id)
		_draw_quota())
	make_label(quota_panel, "pulled from the Stash at shift end", Vector2(6, 78), UiTheme.DUST)

func _draw_cursor() -> void:
	if tool != "build" or Sim.build_tool == "" or cursor_tile.x < 0:
		return
	var d := _def(Sim.build_tool)
	if d.is_empty():
		return
	var ok := cursor_reason == ""
	var r := Rect2(cursor_tile.x * WorldView.TILE, cursor_tile.y * WorldView.TILE, int(d.w) * WorldView.TILE, int(d.h) * WorldView.TILE)
	cursor.draw_rect(r, Color(UiTheme.CYAN if ok else UiTheme.HOT_PINK, 0.25))
	cursor.draw_rect(r, UiTheme.CYAN if ok else UiTheme.HOT_PINK, false, 2.0)

func _def(id: String) -> Dictionary:
	for d in Sim.ensure().structure_defs():
		if String(d.id) == id:
			return d
	return {}

func _process(delta: float) -> void:
	var v := Vector2.ZERO
	if Input.is_key_pressed(KEY_W): v.y -= 1
	if Input.is_key_pressed(KEY_S): v.y += 1
	if Input.is_key_pressed(KEY_A): v.x -= 1
	if Input.is_key_pressed(KEY_D): v.x += 1
	if v != Vector2.ZERO:
		cam.position += v.normalized() * 360.0 * delta / cam.zoom.x
		cam.position = cam.position.round()

func set_view(i: int) -> void:
	view_index = clampi(i, 0, ZOOMS.size() - 1)
	cam.zoom = Vector2.ONE * ZOOMS[view_index]
	world.draw_crowd = view_index < 2
	world.show_districts = view_index == 1
	world.set_view_level(view_index)
	_refresh()

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_TAB: set_view((view_index + 1) % ZOOMS.size())
			KEY_SPACE: Sim.paused = not Sim.paused
			KEY_1: Sim.set_speed(1.0)
			KEY_2: Sim.set_speed(2.0)
			KEY_3: Sim.set_speed(4.0)
		_refresh()
	if event is InputEventMouseMotion and tool == "build":
		var p: Vector2 = get_global_mouse_position()
		hover_tile(int(floor(p.x / WorldView.TILE)), int(floor(p.y / WorldView.TILE)))
	if event is InputEventMouseButton and event.pressed:
		var p: Vector2 = get_global_mouse_position()
		var tx := int(floor(p.x / WorldView.TILE))
		var ty := int(floor(p.y / WorldView.TILE))
		if event.button_index == MOUSE_BUTTON_LEFT:
			apply_tool(tx, ty)
		elif event.button_index == MOUSE_BUTTON_RIGHT:
			cancel_ghost_at(tx, ty)
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			set_view(view_index - 1)
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			set_view(view_index + 1)

## The ghost cursor follows the mouse and asks the colony whether the def may stand there.
func hover_tile(tx: int, ty: int) -> void:
	cursor_tile = Vector2i(tx, ty)
	cursor_reason = String(Sim.ensure().can_place(Sim.build_tool, tx, ty)) if Sim.build_tool != "" else "nothing to place"
	_refresh()

func apply_tool(tx: int, ty: int) -> void:
	var sim: RoseSim = Sim.ensure()
	match tool:
		"select": open_quota(int(sim.consumer_at(tx, ty)))
		"bait": cursor_reason = String(sim.bait_from_stash(tx, ty))
		"salt": sim.salt(tx, ty, true)
		"sever": sim.sever(tx, ty)
		"relay": sim.place_relay(tx, ty)
		"build": place_ghost(tx, ty)
	if tool in ["bait", "salt", "sever", "relay"]:
		world.full_paint()
	_refresh()

## Place the armed def as a ghost; the reason stays on the city line when it cannot stand.
func place_ghost(tx: int, ty: int) -> int:
	if Sim.build_tool == "":
		return -1
	var sim: RoseSim = Sim.ensure()
	cursor_tile = Vector2i(tx, ty)
	cursor_reason = String(sim.can_place(Sim.build_tool, tx, ty))
	if cursor_reason != "":
		return -1
	return sim.place(Sim.build_tool, tx, ty)

## Right-click a ghost to cancel it (a paid one is cut and welded already and stays).
func cancel_ghost_at(tx: int, ty: int) -> void:
	var sim: RoseSim = Sim.ensure()
	for st in sim.structures():
		if bool(st.built):
			continue
		if tx >= int(st.x) and tx < int(st.x) + int(st.w) and ty >= int(st.y) and ty < int(st.y) + int(st.h):
			if not sim.cancel_ghost(int(st.id)):
				cursor_reason = "paid for: it stays"
			_refresh()
			return
