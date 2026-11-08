extends ScreenBase
## Expedition: direct control of a Runner on a route assembled by the `run` crate (RoseRun) from the authored
## chunks, at Street zoom. PAC-020: the route map, its props, the Mat edge as start and extraction, digging through
## collapses and deposits. Fog, light and noise (PAC-021), loot (PAC-022), threats (PAC-023) and the full
## extraction/death rules (PAC-024) build on this screen.
##
## Traversal: slimos are ooze and crawl any surface, so the Runner moves in four directions through every passable
## tile of the side-on cross-section; shafts need no ladders.

const DIG_TIME := 1.2   # s per collapse/deposit tile (design: 4-10 s; tuned down until loot lands in PAC-022)

var world: WorldView
var cam: Camera2D
var runner: Sprite2D
var route: RoseRun
var seed_value := 11
var stratum := 1
var tile := Vector2i(1, 12)
var move_cd := 0.0
var clock := 0.0
var hp := 1.0
var stamina := 1.0
var hud: Label
var card: Label
var frame := 0
var anim_t := 0.0
var passable: PackedByteArray
var substrate: PackedByteArray
var entry_tiles := {}      # tile index -> true
var grate_tiles := {}
var chunk_of: PackedInt32Array
var chunk_info: Array
var containers: Array
var dig_target := Vector2i(-1, -1)
var dig_t := 0.0
var extract_t := 0.0
var extracting := false

func _ready() -> void:
	screen_title = "RUN  -  S%d ROUTE %d" % [stratum, seed_value]
	super._ready()
	route = RoseRun.new()
	var n := route.assemble(seed_value, stratum, 0)
	if n == 0:
		push_error("Run: no route assembled")
	world = WorldView.new()
	world.bind_sim = false
	world.draw_crowd = false
	add_child(world)
	_paint_route()
	runner = Sprite2D.new()
	runner.texture = Assets.texture("slimo.d2.walk")
	runner.region_enabled = true
	runner.region_rect = Rect2(0, 0, 32, 48)
	runner.centered = false
	runner.offset = Vector2(0, -16)
	runner.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	add_child(runner)
	cam = Camera2D.new()
	add_child(cam)
	cam.make_current()
	_place()

func _paint_route() -> void:
	substrate = route.substrate()
	passable = route.passable()
	chunk_of = route.chunk_of()
	chunk_info = route.chunks()
	containers = route.containers()
	world.paint_substrate(substrate, route.width(), route.height())
	# the Mat edge: ooze on the entry window and two tiles in, the Heart's last cord
	entry_tiles.clear()
	for i in route.entry():
		entry_tiles[i] = true
		var x := i % world.w
		var y := i / world.w
		world.put_mat(x, y, 3 if y == 12 else 2, "fed")
		world.put_mat(x + 1, y, 2, "fed")
		world.put_mat(x + 2, y, 1, "")
	grate_tiles.clear()
	for i in route.grates():
		grate_tiles[i] = true
	# props
	for c in containers:
		world.put_prop(int(c.x), int(c.y), "prop.crate" if c.kind == "crate" else "prop.locker", "closed")
	for s in route.spawns():
		match String(s.kind):
			"corpse": world.put_prop(int(s.x), int(s.y), "prop.corpse", "fresh")
			"sewerat": world.put_prop(int(s.x), int(s.y), "prop.burrow", "quiet")
			"patrol": world.put_prop(int(s.x), int(s.y), "prop.patrol_mark", "")
	for l in route.lights():
		world.put_prop(int(l.x), int(l.y), "prop.lamp", "lit")
	for hz in route.hazards():
		if hz.kind == "gas":
			world.put_prop(int(hz.x), int(hz.y), "prop.gas", "", (int(hz.x) + int(hz.y)) % 4)
	# start on the middle entry tile
	var e: PackedInt32Array = route.entry()
	if e.size() > 0:
		var mid: int = e[int(e.size() / 2.0)]
		tile = Vector2i(mid % world.w, mid / world.w)

func _build_content(c: Control) -> void:
	make_gauge(c, "ui.gauge.ooze", Vector2(0, 270), 1.0, "HP")
	make_gauge(c, "ui.gauge.ooze", Vector2(0, 284), 1.0, "STAMINA")
	hud = make_label(c, "", Vector2(0, 0), UiTheme.CYAN)
	card = make_label(c, "", Vector2(0, 12), UiTheme.DUST)
	make_label(c, "WASD move   SHIFT sprint   F dig   E extract on the Mat or a grate   TAB stash", Vector2(260, 284), UiTheme.DUST)
	var quick := HBoxContainer.new()
	quick.position = Vector2(480, 250)
	c.add_child(quick)
	for id in ["ger_lantern", "org_antiseptic", "org_stim", "prt_salt"]:
		var f := Panel.new()
		f.custom_minimum_size = Vector2(36, 36)
		quick.add_child(f)
		make_slot(f, icon_for(id), Vector2(2, 2))

func _idx(t: Vector2i) -> int:
	return t.y * world.w + t.x

func _in_bounds(t: Vector2i) -> bool:
	return t.x >= 0 and t.y >= 0 and t.x < world.w and t.y < world.h

func _open(t: Vector2i) -> bool:
	return _in_bounds(t) and passable[_idx(t)] != 0

func _diggable(t: Vector2i) -> bool:
	if not _in_bounds(t):
		return false
	var s := int(substrate[_idx(t)])
	return s == 4 or s == 8

func on_mat() -> bool:
	return entry_tiles.has(_idx(tile))

func on_grate() -> bool:
	return grate_tiles.has(_idx(tile))

func can_extract() -> bool:
	return on_mat() or on_grate()

func current_chunk() -> String:
	var ci := int(chunk_of[_idx(tile)])
	if ci < 0 or ci >= chunk_info.size():
		return "-"
	return String(chunk_info[ci].id)

func _place() -> void:
	runner.position = Vector2(tile.x * WorldView.TILE, tile.y * WorldView.TILE)
	cam.position = runner.position + Vector2(16, 16)

func step(dir: Vector2i) -> bool:
	var t := tile + dir
	if _open(t):
		tile = t
		_place()
		return true
	return false

## Dig the tile in `dir`; returns true when the dig finished this call.
func dig(dir: Vector2i, dt: float) -> bool:
	var t := tile + dir
	if not _diggable(t):
		dig_target = Vector2i(-1, -1)
		dig_t = 0.0
		return false
	if t != dig_target:
		dig_target = t
		dig_t = 0.0
	dig_t += dt
	if dig_t >= DIG_TIME:
		if route.dig(t.x, t.y):
			substrate = route.substrate()
			passable = route.passable()
			world.repaint_tile(t.x, t.y, int(substrate[_idx(t)]))
		dig_target = Vector2i(-1, -1)
		dig_t = 0.0
		return true
	return false

func _process(delta: float) -> void:
	clock += delta
	move_cd -= delta
	var sprint := Input.is_key_pressed(KEY_SHIFT) and stamina > 0.05
	var d := Vector2i.ZERO
	if Input.is_key_pressed(KEY_W): d = Vector2i(0, -1)
	elif Input.is_key_pressed(KEY_S): d = Vector2i(0, 1)
	elif Input.is_key_pressed(KEY_A): d = Vector2i(-1, 0)
	elif Input.is_key_pressed(KEY_D): d = Vector2i(1, 0)
	var digging := Input.is_key_pressed(KEY_F) and d != Vector2i.ZERO
	if digging:
		dig(d, delta)
	elif move_cd <= 0.0 and d != Vector2i.ZERO:
		if step(d):
			move_cd = 0.09 if sprint else 0.16
			if sprint:
				stamina = maxf(0.0, stamina - 0.03)
		runner.flip_h = d.x < 0
	if not digging:
		dig_target = Vector2i(-1, -1)
		dig_t = 0.0
	if extracting:
		extract_t += delta
		if not can_extract():
			extracting = false
			extract_t = 0.0
		elif extract_t >= 3.0:
			Screens.show_screen("stash")
			return
	stamina = minf(1.0, stamina + delta * 0.08)
	anim_t += delta
	if anim_t > 0.12:
		anim_t = 0.0
		frame = (frame + 1) % 8
		runner.region_rect = Rect2(frame * 32, 0, 32, 48)
	var where := "ON THE MAT: E extracts" if on_mat() else ("UNDER A GRATE: E extracts for a toll" if on_grate() else "off the Mat")
	var act := ""
	if digging and dig_target.x >= 0:
		act = "   digging %d%%" % int(dig_t / DIG_TIME * 100.0)
	elif extracting:
		act = "   extracting %d%%" % int(extract_t / 3.0 * 100.0)
	hud.text = "run %02d:%02d   %s   tile %d,%d   %s   %s%s" % [int(clock) / 60, int(clock) % 60, current_chunk(), tile.x, tile.y, where, "sprint" if sprint else "quiet", act]
	card.text = "route: %d chunks   %d containers   %d lights   lume 6  noise %s" % [route.chunk_count(), containers.size(), route.lights().size(), "loud" if digging else "low"]

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_E and can_extract():
			extracting = true
			extract_t = 0.0
		elif event.keycode == KEY_TAB:
			Screens.show_screen("stash")
