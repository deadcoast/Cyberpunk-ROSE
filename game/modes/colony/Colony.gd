extends ScreenBase
## Colony: the world at Street / District / Stack zoom, the clock, the tools, and the city on it (Full Design v3.9):
## structures and ghosts drawn on their footprints, the build tool placing the def the Build menu armed (the ghost
## cursor is Cyan where it may stand, Hot Pink with the reason where it may not), districts named in District view.

const ZOOMS := [1.0, 0.25, 1.0 / 32.0]
const VIEW_NAMES := ["STREET", "DISTRICT", "STACK"]

var world: WorldView
var cam: Camera2D
var view_index := 0
var stats_label: Label
var city_label: Label
var tool := "feed"
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
	for t in ["feed", "bait", "salt", "sever", "relay", "build"]:
		var b := Button.new()
		b.text = t.to_upper()
		b.focus_mode = Control.FOCUS_NONE
		b.pressed.connect(func():
			tool = t
			if t == "build" and Sim.build_tool == "":
				Screens.show_screen("build")
			_refresh())
		tools.add_child(b)
	var hint := Label.new()
	hint.text = "  LMB apply   RMB cancel ghost   TAB view   SPACE pause   1/2/3 speed   WASD pan"
	hint.add_theme_color_override("font_color", UiTheme.DUST)
	tools.add_child(hint)


func _refresh() -> void:
	var sim: RoseSim = Sim.ensure()
	var s: Dictionary = sim.stats()
	stats_label.text = "t %d  %s  %s  %.0f Hz   cells %d  nodes %d   slimos %d  carrying %d  cargo %d  delivered %d  jam %d" % [
		s.get("tick", 0), VIEW_NAMES[view_index], "PAUSED" if Sim.paused else "RUN", Sim.tick_hz,
		s.get("cells", 0), s.get("nodes", 0), s.get("agents", 0), s.get("carrying", 0), s.get("packets", 0), s.get("deliveries", 0), s.get("jammed", 0)]
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
		line += "   tool %s" % tool.to_upper()
	var waiting: Array = Sim.last_shift.get("waiting", [])
	if waiting.size() > 0:
		line += "   %s waits for %s" % [String(waiting[0].name), String(waiting[0].cost.text)]
	city_label.text = line.left(128)
	world.redraw_city()
	cursor.queue_redraw()

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
	world.draw_crowd = view_index == 0
	world.show_districts = view_index == 1
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
		"feed": sim.feed(tx, ty, 5.0)
		"bait": sim.bait(tx, ty, 0.4)
		"salt": sim.salt(tx, ty, true)
		"sever": sim.sever(tx, ty)
		"relay": sim.place_relay(tx, ty)
		"build": place_ghost(tx, ty)
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
