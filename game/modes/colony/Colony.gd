extends ScreenBase
## Colony: the world at Street / District / Stack zoom, the clock, the tools. Phase A shell: view, clock, feed and sever.

const ZOOMS := [1.0, 0.25, 1.0 / 32.0]
const VIEW_NAMES := ["STREET", "DISTRICT", "STACK"]

var world: WorldView
var cam: Camera2D
var view_index := 0
var stats_label: Label
var tool := "feed"

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
	Sim.ticked.connect(_refresh)
	_refresh()

func _build_content(c: Control) -> void:
	stats_label = make_label(c, "", Vector2(0, 0), UiTheme.CYAN)
	var tools := HBoxContainer.new()
	tools.position = Vector2(0, 280)
	tools.add_theme_constant_override("separation", 4)
	c.add_child(tools)
	for t in ["feed", "bait", "salt", "sever", "relay"]:
		var b := Button.new()
		b.text = t.to_upper()
		b.focus_mode = Control.FOCUS_NONE
		b.pressed.connect(func(): tool = t)
		tools.add_child(b)
	var hint := Label.new()
	hint.text = "  LMB apply   TAB view   SPACE pause   1/2/3 speed   WASD pan"
	hint.add_theme_color_override("font_color", UiTheme.DUST)
	tools.add_child(hint)
	make_gauge(c, "ui.gauge.attention", Vector2(520, 0), 0.3, "ATTENTION")

func _refresh() -> void:
	var s: Dictionary = Sim.ensure().stats()
	stats_label.text = "t %d  %s  %s  %.0f Hz   cells %d  nodes %d   slimos %d  carrying %d  cargo %d  delivered %d  jam %d" % [
		s.get("tick", 0), VIEW_NAMES[view_index], "PAUSED" if Sim.paused else "RUN", Sim.tick_hz,
		s.get("cells", 0), s.get("nodes", 0), s.get("agents", 0), s.get("carrying", 0), s.get("packets", 0), s.get("deliveries", 0), s.get("jammed", 0)]

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
	if event is InputEventMouseButton and event.pressed:
		var p: Vector2 = get_global_mouse_position()
		var tx := int(floor(p.x / WorldView.TILE))
		var ty := int(floor(p.y / WorldView.TILE))
		if event.button_index == MOUSE_BUTTON_LEFT:
			apply_tool(tx, ty)
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			set_view(view_index - 1)
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			set_view(view_index + 1)

func apply_tool(tx: int, ty: int) -> void:
	var sim: RoseSim = Sim.ensure()
	match tool:
		"feed": sim.feed(tx, ty, 5.0)
		"bait": sim.bait(tx, ty, 0.4)
		"salt": sim.salt(tx, ty, true)
		"sever": sim.sever(tx, ty)
		"relay": sim.place_relay(tx, ty)
	world.full_paint()
