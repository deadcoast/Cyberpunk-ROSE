extends Node2D
## Cyberpunk: ROSE — greybox shell. Godot owns the clock, the camera and the picture; RoseSim owns every number.

const TILE := 16                       # Street view: one tile = 16 px
const ZOOMS := [1.0, 0.25, 1.0 / 16.0] # Street, District, Stack  (px per tile: 16, 4, 1)
const VIEW_NAMES := ["STREET", "DISTRICT", "STACK"]

@export var seed_value: int = 11
@export var agents: int = 3000
@export var grow_ticks: int = 500
@export var tick_hz: float = 2.0

var sim: RoseSim
var world: Node2D
var cam: Camera2D
var stats_label: Label
var hints_label: Label

var paused := false
var speed_index := 1                   # 0:1 Hz 1:2 Hz 2:4 Hz
var view_index := 0
var accum := 0.0
var ticks_per_frame := 0               # headless tests: step this many ticks per frame, ignore the clock
var tool := "feed"
var hover := Vector2i(-1, -1)
var last_stats := {}

func _ready() -> void:
	world = $World
	cam = $Camera
	stats_label = $UI/Stats
	hints_label = $UI/Hints
	sim = RoseSim.new()
	sim.start_scale(seed_value, agents, grow_ticks)
	world.bind(sim)
	world.full_redraw()
	cam.zoom = Vector2.ONE * ZOOMS[view_index]
	cam.position = Vector2(sim.width() * TILE / 2.0, sim.height() * TILE / 2.0)
	hints_label.text = "LMB feed 5   RMB sever   S salt   B bait   R relay   Space pause   1/2/3 speed   Tab view   WASD pan   wheel zoom"
	_refresh_stats()

func set_speed(index: int) -> void:
	speed_index = clampi(index, 0, 2)
	tick_hz = [1.0, 2.0, 4.0][speed_index]

func _process(delta: float) -> void:
	_pan(delta)
	if ticks_per_frame > 0:
		for i in ticks_per_frame:
			_tick()
		return
	if paused:
		return
	accum += delta
	var period := 1.0 / tick_hz
	var n := 0
	while accum >= period and n < 8:
		accum -= period
		_tick()
		n += 1

func _tick() -> void:
	sim.step()
	world.on_tick()
	_refresh_stats()

func _refresh_stats() -> void:
	last_stats = sim.stats() as Dictionary
	var s: Dictionary = last_stats
	stats_label.text = "t %d  %s  %s  %.1f Hz\ncells %d  nodes %d  reach %d\nslimos %d  carrying %d  cargo %d  delivered %d  jam %d\nstep %d us" % [
		s.get("tick", 0), VIEW_NAMES[view_index], "PAUSED" if paused else "RUN", tick_hz,
		s.get("cells", 0), s.get("nodes", 0), s.get("reach", 0),
		s.get("agents", 0), s.get("carrying", 0), s.get("packets", 0), s.get("deliveries", 0), s.get("jammed", 0),
		s.get("step_us", 0)]

func _pan(delta: float) -> void:
	var v := Vector2.ZERO
	if Input.is_key_pressed(KEY_W): v.y -= 1
	if Input.is_key_pressed(KEY_S) and not Input.is_key_pressed(KEY_SHIFT): v.y += 1
	if Input.is_key_pressed(KEY_A): v.x -= 1
	if Input.is_key_pressed(KEY_D): v.x += 1
	if v != Vector2.ZERO:
		cam.position += v.normalized() * 240.0 * delta / cam.zoom.x
	cam.position = cam.position.round()

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_SPACE: paused = not paused
			KEY_1: set_speed(0)
			KEY_2: set_speed(1)
			KEY_3: set_speed(2)
			KEY_TAB: _cycle_view()
			KEY_B: tool = "bait"
			KEY_R: tool = "relay"
			KEY_F: tool = "feed"
		if event.keycode == KEY_S and event.shift_pressed:
			tool = "salt"
		_refresh_stats()
	if event is InputEventMouseButton and event.pressed:
		var tile := _mouse_tile()
		match event.button_index:
			MOUSE_BUTTON_LEFT: _apply_tool(tile)
			MOUSE_BUTTON_RIGHT: sim.sever(tile.x, tile.y)
			MOUSE_BUTTON_WHEEL_UP: _zoom_step(-1)
			MOUSE_BUTTON_WHEEL_DOWN: _zoom_step(1)
		world.full_redraw()
	if event is InputEventMouseMotion:
		hover = _mouse_tile()
		world.set_hover(hover)

func _apply_tool(tile: Vector2i) -> void:
	match tool:
		"feed": sim.feed(tile.x, tile.y, 5.0)
		"bait": sim.bait(tile.x, tile.y, 0.4)
		"salt": sim.salt(tile.x, tile.y, true)
		"relay": sim.place_relay(tile.x, tile.y)

func _mouse_tile() -> Vector2i:
	var p: Vector2 = get_global_mouse_position()
	return Vector2i(int(floor(p.x / TILE)), int(floor(p.y / TILE)))

func _cycle_view() -> void:
	view_index = (view_index + 1) % ZOOMS.size()
	cam.zoom = Vector2.ONE * ZOOMS[view_index]
	world.set_view(view_index)

func _zoom_step(dir: int) -> void:
	view_index = clampi(view_index + dir, 0, ZOOMS.size() - 1)
	cam.zoom = Vector2.ONE * ZOOMS[view_index]
	world.set_view(view_index)
