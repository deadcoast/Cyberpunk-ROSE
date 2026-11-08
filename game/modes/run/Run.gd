extends ScreenBase
## Expedition: direct control of a Runner on a route assembled by the `run` crate (RoseRun) from the authored
## chunks, at Street zoom. PAC-020 gave the route, props, the Mat edge as start and extraction, digging. PAC-021
## adds the senses: light, symmetric sight, fog with route memory, noise footprints; the fog overlay draws them and
## the HUD shows the one number that matters, exposure. PAC-022 adds loot: sources on the route (lockers, crates,
## corpses, dug piles), opening with hands or the pry bar, the manifest unrolling into a loot panel, taking into the
## rig, and the rig handed to the Stash on extraction. Threats (PAC-023) and the full extraction/death rules
## (PAC-024) build on this screen.
##
## Traversal: slimos are ooze and crawl any surface, so the Runner moves in four directions through every passable
## tile of the side-on cross-section; shafts need no ladders.

const DIG_TIME := 1.2        # s per collapse/deposit tile (design: 4-10 s; tuned down until loot lands in PAC-022)
const SENSE_HZ := 20.0       # the senses tick; fixed so a recorded run replays bit-exact
const LANTERN_LIGHT := 6     # ger_lantern gear light from data/items.toml
const VEINS_D2 := 2          # vein glow of a Mod-Depth 2 Runner
const EYE := 3               # the Runner's Eye stat until Runner sheets land (PAC-032)
const KIT := ["ger_lantern", "ger_pry_bar", "org_antiseptic", "prt_salt"]   # quick slots carried in
const UNROLL := 0.22         # s between manifest lines (Pass 2, Pillar 1: item by item, rare last)
const TIER_COLOR := {"common": UiTheme.DUST, "uncommon": UiTheme.CYAN, "rare": UiTheme.HOT_PINK, "blackline": UiTheme.OOZE_GLOW, "relic": UiTheme.VESTMENT}

var world: WorldView
var fog: FogOverlay
var cam: Camera2D
var runner: Sprite2D
var route: RoseRun
var seed_value := 11
var stratum := 1
var tile := Vector2i(1, 12)
var move_cd := 0.0
var clock := 0.0
var sense_accum := 0.0
var hp := 1.0
var stamina := 1.0
var hud: Label
var card: Label
var gem: Label
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
var lantern_on := true
var dimming := false
var sneaking := false
var motion := 0            # 0 still 1 sneak 2 walk 3 sprint 4 dig, as the senses tick reads it
var memory_bits := PackedByteArray()   # what this route remembered before this run (from the save in PAC-026)
var has_pry_bar := true
var sources := []                      # RoseRun.sources() snapshot, refreshed on change
var source_prop := {}                  # source index -> [slot_id, open_state, empty_state]
var loot_panel: Panel
var loot_title: Label
var loot_lines: VBoxContainer
var loot_foot: Label
var loot_open := -1                    # source index shown in the panel
var loot_items := []                   # the open source's items, unrolling
var loot_shown := 0
var loot_t := 0.0
var rig_line: Label

func _ready() -> void:
	screen_title = "RUN  -  S%d ROUTE %d" % [stratum, seed_value]
	super._ready()
	route = RoseRun.new()
	route.set_mat_lume(Sim.ensure().gene(4))
	var n := route.assemble(seed_value, stratum, 0)
	if n == 0:
		push_error("Run: no route assembled")
	if not memory_bits.is_empty():
		route.restore_memory(memory_bits)
	route.begin_run(Sim.run_nonce(), Sim.runs_of(stratum, seed_value), Sim.day, 8, 5, 18.0)
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
	runner.z_index = 5
	add_child(runner)
	fog = FogOverlay.new()
	fog.setup(route.width(), route.height())
	add_child(fog)
	cam = Camera2D.new()
	add_child(cam)
	cam.make_current()
	_place()
	_sense_tick()

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
	# props: loot sources carry their state; the rest are marks
	_refresh_sources()
	for s in route.spawns():
		match String(s.kind):
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
	gem = make_label(c, "", Vector2(0, 24), UiTheme.OOZE_GLOW)
	rig_line = make_label(c, "", Vector2(0, 36), UiTheme.CHALK)
	make_label(c, "WASD move  SHIFT sprint  CTRL sneak  Q dim veins  L lantern  F dig  E open / extract  TAB stash", Vector2(190, 284), UiTheme.DUST)
	var quick := HBoxContainer.new()
	quick.position = Vector2(480, 250)
	c.add_child(quick)
	for id in KIT:
		var f := Panel.new()
		f.custom_minimum_size = Vector2(36, 36)
		quick.add_child(f)
		make_slot(f, icon_for(id), Vector2(2, 2))
	# the loot panel: hidden until a source opens; the manifest unrolls into it
	loot_panel = make_frame(c, Vector2(392, 48), Vector2(240, 196))
	loot_panel.visible = false
	loot_title = make_label(loot_panel, "", Vector2(6, 4), UiTheme.CYAN)
	loot_lines = VBoxContainer.new()
	loot_lines.position = Vector2(6, 18)
	loot_lines.add_theme_constant_override("separation", 1)
	loot_panel.add_child(loot_lines)
	loot_foot = make_label(loot_panel, "", Vector2(6, 176), UiTheme.DUST)

func _refresh_sources() -> void:
	sources = route.sources()
	for s in sources:
		var spec: Array
		match String(s.kind):
			"locker": spec = ["prop.locker", "open", "empty"]
			"crate": spec = ["prop.crate", "open", "empty"]
			"corpse": spec = ["prop.corpse", "picked", "picked"]
			"pile": spec = ["prop.pile", "full", "picked"]
			_: spec = ["prop.crate", "open", "empty"]
		source_prop[int(s.index)] = spec
		var state: String
		match String(s.state):
			"open": state = spec[1]
			"empty": state = spec[2]
			_: state = "fresh" if s.kind == "corpse" else ("full" if s.kind == "pile" else "closed")
		world.put_prop(int(s.x), int(s.y), spec[0], state)

func source_here() -> int:
	return route.source_at(tile.x, tile.y)

func _source(idx: int) -> Dictionary:
	for s in sources:
		if int(s.index) == idx:
			return s
	return {}

## Open the source under the Runner (or show the open one); the pry bar is used when carried.
func open_here() -> bool:
	var idx := source_here()
	if idx < 0:
		return false
	var s := _source(idx)
	if s.is_empty():
		return false
	if s.state == "closed":
		var method := 1 if has_pry_bar else 0
		var r: Dictionary = route.open_source(idx, method)
		if not bool(r.get("ok", false)):
			return false
		_refresh_sources()
		loot_open = idx
		loot_items = route.source_items(idx)
		loot_shown = 0
		loot_t = 0.0
		for ch in loot_lines.get_children():
			ch.queue_free()
		var how := "forced" if bool(r.forced) else ("pried" if method == 1 and (s.kind == "crate" or bool(s.locked)) else "opened")
		var extra := "   FIRST-RUN FIND" if bool(r.generous) else ""
		loot_title.text = "%s %s   tier %d   %d items%s" % [String(s.kind).to_upper(), how, int(s.tier), int(r.revealed), extra]
		loot_foot.text = "..." if int(r.revealed) > 0 else "nothing in it.   E close"
		loot_panel.visible = true
		fog.feed(route.overlay_rgb())
		return true
	if s.state == "open" or s.state == "empty":
		loot_open = idx
		loot_items = route.source_items(idx)
		loot_shown = loot_items.size()
		_redraw_loot()
		loot_panel.visible = true
		return true
	return false

func _redraw_loot() -> void:
	for ch in loot_lines.get_children():
		ch.queue_free()
	var s := _source(loot_open)
	loot_title.text = "%s   tier %d   %d items" % [String(s.get("kind", "")).to_upper(), int(s.get("tier", 0)), loot_items.size()]
	for k in mini(loot_shown, loot_items.size()):
		_add_loot_line(loot_items[k])
	loot_foot.text = ("F take all   E close" if not loot_items.is_empty() else "empty.   E close")

func _add_loot_line(it: Dictionary) -> void:
	var l := Label.new()
	var tier := String(it.tier)
	l.text = "%-30s %dx%d %4.1fkg c%d" % [String(it.label).left(30), int(it.w), int(it.h), float(it.weight), int(it.condition)]
	l.add_theme_color_override("font_color", TIER_COLOR.get(tier, UiTheme.DUST))
	loot_lines.add_child(l)

func take_all_here() -> Array:
	if loot_open < 0:
		return [0, 0]
	var r: PackedInt32Array = route.take_all(loot_open)
	_refresh_sources()
	loot_items = route.source_items(loot_open)
	loot_shown = loot_items.size()
	_redraw_loot()
	var left := int(r[1]) if r.size() > 1 else 0
	loot_foot.text = ("took %d   %d left: no room in the rig   E close" % [int(r[0]), left]) if left > 0 else ("took %d.   E close" % int(r[0]))
	return [int(r[0]), left]

func close_loot() -> void:
	loot_panel.visible = false
	loot_open = -1

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

## One step; the footstep is the sound of the current motion on the tile stepped onto.
func step(dir: Vector2i) -> bool:
	var t := tile + dir
	if _open(t):
		tile = t
		_place()
		_sense_tick()
		match motion:
			1: route.runner_sound(0, 0)
			3: route.runner_sound(2, 0)
			_: route.runner_sound(1, 0)
		return true
	return false

## Dig the tile in `dir`; returns true when the dig finished this call. Digging is loud every tick.
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
			_refresh_sources()
		dig_target = Vector2i(-1, -1)
		dig_t = 0.0
		return true
	return false

func _sense_tick() -> void:
	var gear_light := LANTERN_LIGHT if lantern_on else 0
	route.tick(tile.x, tile.y, VEINS_D2, gear_light, dimming, false, EYE, motion)
	if motion == 4:
		route.runner_sound(3, 0)
	fog.feed(route.overlay_rgb())

func _process(delta: float) -> void:
	clock += delta
	move_cd -= delta
	var sprint := Input.is_key_pressed(KEY_SHIFT) and stamina > 0.05 and not sneaking
	sneaking = Input.is_key_pressed(KEY_CTRL)
	dimming = Input.is_key_pressed(KEY_Q) and stamina > 0.02
	var d := Vector2i.ZERO
	if Input.is_key_pressed(KEY_W): d = Vector2i(0, -1)
	elif Input.is_key_pressed(KEY_S): d = Vector2i(0, 1)
	elif Input.is_key_pressed(KEY_A): d = Vector2i(-1, 0)
	elif Input.is_key_pressed(KEY_D): d = Vector2i(1, 0)
	var digging := Input.is_key_pressed(KEY_F) and d != Vector2i.ZERO
	if digging:
		motion = 4
		dig(d, delta)
	elif d != Vector2i.ZERO:
		motion = 1 if sneaking else (3 if sprint else 2)
		if move_cd <= 0.0:
			if step(d):
				move_cd = 0.26 if sneaking else (0.09 if sprint else 0.16)
				if sprint:
					stamina = maxf(0.0, stamina - 0.03)
			runner.flip_h = d.x < 0
	else:
		motion = 0
	if not digging:
		dig_target = Vector2i(-1, -1)
		dig_t = 0.0
	if dimming:
		stamina = maxf(0.0, stamina - delta * 0.12)
	if extracting:
		extract_t += delta
		if not can_extract():
			extracting = false
			extract_t = 0.0
		elif extract_t >= 3.0:
			Sim.pending_haul = route.rig_export()
			Sim.pending_haul_from = "S%d route %d" % [stratum, seed_value]
			Sim.note_run(stratum, seed_value)
			Screens.show_screen("stash")
			return
	stamina = minf(1.0, stamina + delta * (0.0 if dimming else 0.08))
	sense_accum += delta
	var period := 1.0 / SENSE_HZ
	while sense_accum >= period:
		sense_accum -= period
		if extracting:
			route.runner_sound(7, 0)
		_sense_tick()
	anim_t += delta
	if anim_t > 0.12:
		anim_t = 0.0
		frame = (frame + 1) % 8
		runner.region_rect = Rect2(frame * 32, 0, 32, 48)
	# the manifest unrolls one line at a time; the notable comes last with a longer beat
	if loot_panel.visible and loot_shown < loot_items.size():
		loot_t += delta
		var beat := UNROLL * (2.5 if loot_shown == loot_items.size() - 1 and String(loot_items[loot_shown].tier) in ["rare", "blackline", "relic"] else 1.0)
		if loot_t >= beat:
			loot_t = 0.0
			_add_loot_line(loot_items[loot_shown])
			loot_shown += 1
			if loot_shown == loot_items.size():
				loot_foot.text = "F take all   E close"
	if loot_open >= 0 and source_here() != loot_open:
		close_loot()
	var src := source_here()
	var where := "ON THE MAT: E extracts" if on_mat() else ("UNDER A GRATE: E extracts for a toll" if on_grate() else "off the Mat")
	if src >= 0 and loot_open < 0:
		var sd := _source(src)
		var st := String(sd.get("state", ""))
		if st == "closed":
			where = "%s%s: E %s" % [String(sd.kind).to_upper(), " (LOCKED)" if bool(sd.get("locked", false)) else "", "pry" if has_pry_bar and (sd.kind == "crate" or bool(sd.get("locked", false))) else ("force" if bool(sd.get("locked", false)) else "open")]
		elif st == "open":
			where = "%s open: E look" % String(sd.kind).to_upper()
	var rs: PackedInt32Array = route.rig_stats()
	rig_line.text = "rig %d items  %d free  %.1f/%.0f kg%s   blind value %d e" % [rs[0] if rs.size() > 0 else 0, rs[1] if rs.size() > 1 else 0, route.rig_weight(), route.rig_carry_cap(), "  OVERWEIGHT" if route.rig_overweight() else "", route.rig_value_blind()]
	var act := ""
	if digging and dig_target.x >= 0:
		act = "   digging %d%%" % int(dig_t / DIG_TIME * 100.0)
	elif extracting:
		act = "   extracting %d%%" % int(extract_t / 3.0 * 100.0)
	var pace: String = ["still", "sneaking", "walking", "SPRINTING", "DIGGING"][motion]
	hud.text = "run %02d:%02d   %s   tile %d,%d   %s   %s%s" % [int(clock) / 60, int(clock) % 60, current_chunk(), tile.x, tile.y, where, pace, act]
	var st: PackedInt32Array = route.sense_stats()
	card.text = "route: %d chunks   %d containers   seen %d tiles (%d now)   %s" % [route.chunk_count(), containers.size(), st[1] if st.size() > 1 else 0, st[0] if st.size() > 0 else 0, "lantern ON" if lantern_on else "lantern off"]
	gem.text = "lume %d   sees %d   exposure %d %s   noticed within %d   noise here %d%s" % [route.lume_emit(), route.see_radius(), route.exposure(), route.exposure_band().to_upper(), route.notice_range(), route.noise_here(), "   VEINS DIMMED" if dimming else ""]

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_E and loot_open >= 0:
			close_loot()
		elif event.keycode == KEY_E and source_here() >= 0:
			open_here()
		elif event.keycode == KEY_F and loot_open >= 0 and loot_shown == loot_items.size():
			take_all_here()
		elif event.keycode == KEY_E and can_extract():
			extracting = true
			extract_t = 0.0
		elif event.keycode == KEY_L:
			lantern_on = not lantern_on
			_sense_tick()
		elif event.keycode == KEY_TAB:
			Screens.show_screen("stash")
