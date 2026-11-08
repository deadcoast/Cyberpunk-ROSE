extends ScreenBase
## Expedition: the view and controller of a run. Every rule lives in Rust (RoseRun over run::sim::RunSim): this
## screen gathers the input word at 20 Hz, steps the simulation, and draws its state and events — the route and
## its props, the Runner at a sub-tile position, creatures, the fog/light/noise overlay, the HUD (sections, stamina,
## the light gem), the loot panel with its unrolling manifest, hit flashes and labels. On extraction the rig goes to
## the Stash; on death the Runner's record stays in the run (PAC-024 adds the corpse and recovery).

const SIM_HZ := 20.0
const UNROLL := 0.22
const TIER_COLOR := {"common": UiTheme.DUST, "uncommon": UiTheme.CYAN, "rare": UiTheme.HOT_PINK, "blackline": UiTheme.OOZE_GLOW, "relic": UiTheme.VESTMENT}
const KIT := ["ger_lantern", "ger_pry_bar", "org_antiseptic", "prt_salt"]
const WEAPON := "ger_pipe"
const BODY := 5
const SECTION_LABEL := ["HEAD", "TORSO", "ARMS", "LEGS"]
# input word bits (run::sim::RunInput::pack)
const B_SPRINT := 1
const B_SNEAK := 2
const B_DIM := 4
const B_ATTACK := 8
const B_SHOOT := 16
const B_DODGE := 32
const B_INTERACT := 64
const B_DIG := 128
const B_TAKE := 256
const B_LANTERN := 512

var world: WorldView
var fog: FogOverlay
var cam: Camera2D
var runner: Sprite2D
var route: RoseRun
var seed_value := 11
var stratum := 1
var sim_accum := 0.0
var clock := 0.0
var frame := 0
var anim_t := 0.0
var hud: Label
var gem: Label
var rig_line: Label
var body_lines: Array[Label] = []
var status_line: Label
var stamina_fill: Control
var sources := []
var creature_nodes := {}      # id -> Sprite2D
var flashes := []             # [node, t]
var labels: Node2D
var loot_panel: Panel
var loot_title: Label
var loot_lines: VBoxContainer
var loot_foot: Label
var loot_open := -1
var loot_items := []
var loot_shown := 0
var loot_t := 0.0
var lantern_pressed := false
var over := ""
var over_t := 0.0
var state := {}
var last_salt := PackedByteArray()
var memory_bits := PackedByteArray()
var _last_events: Array = []
var scripted: Array = []          # [buttons, dx, dy] set by a test script instead of the keyboard
var runner_pose := ""
var latched := 0

func _ready() -> void:
	screen_title = "RUN  -  S%d ROUTE %d" % [stratum, seed_value]
	super._ready()
	route = RoseRun.new()
	route.set_mat_lume(Sim.ensure().gene(4))
	if route.assemble(seed_value, stratum, 0) == 0:
		push_error("Run: no route assembled")
	if not memory_bits.is_empty():
		route.restore_memory(memory_bits)
	route.begin_run(Sim.run_nonce(), Sim.runs_of(stratum, seed_value), Sim.day, PackedStringArray(KIT), WEAPON, BODY)
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
	runner.offset = Vector2(-16, -40)
	runner.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	runner.z_index = 5
	add_child(runner)
	labels = Node2D.new()
	labels.z_index = 20
	add_child(labels)
	fog = FogOverlay.new()
	fog.setup(route.width(), route.height())
	add_child(fog)
	cam = Camera2D.new()
	add_child(cam)
	cam.make_current()
	state = route.runner()
	_sync_view()

func _paint_route() -> void:
	world.paint_substrate(route.substrate(), route.width(), route.height())
	for i in route.entry():
		var x := i % world.w
		var y := i / world.w
		world.put_mat(x, y, 3 if y == 12 else 2, "fed")
		world.put_mat(x + 1, y, 2, "fed")
		world.put_mat(x + 2, y, 1, "")
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

func _refresh_sources() -> void:
	sources = route.sources()
	for s in sources:
		var slot: String
		var closed := "closed"
		var opened := "open"
		var empty := "empty"
		match String(s.kind):
			"locker": slot = "prop.locker"
			"crate": slot = "prop.crate"
			"corpse":
				slot = "prop.corpse"; closed = "fresh"; opened = "picked"; empty = "picked"
			"pile":
				slot = "prop.pile"; closed = "full"; opened = "full"; empty = "picked"
			"scout body":
				slot = "prop.corpse"; closed = "fresh"; opened = "picked"; empty = "picked"
			"rat corpse":
				slot = "prop.pile"; closed = "full"; opened = "full"; empty = "picked"
			_: slot = "prop.crate"
		var st: String = {"closed": closed, "open": opened, "empty": empty}.get(String(s.state), closed)
		world.put_prop(int(s.x), int(s.y), slot, st)

func _build_content(c: Control) -> void:
	hud = make_label(c, "", Vector2(0, 0), UiTheme.CYAN)
	gem = make_label(c, "", Vector2(0, 12), UiTheme.OOZE_GLOW)
	rig_line = make_label(c, "", Vector2(0, 24), UiTheme.CHALK)
	# the body: four sections, bottom-left, over the stamina bar
	for i in 4:
		var l := make_label(c, "", Vector2(0, 236 + i * 10), UiTheme.CHALK)
		body_lines.append(l)
	stamina_fill = make_gauge(c, "ui.gauge.ooze", Vector2(0, 280), 1.0, "STAMINA")
	status_line = make_label(c, "", Vector2(104, 280), UiTheme.HOT_PINK)
	make_label(c, "WASD  SHIFT sprint  CTRL sneak  Q dim  SPACE dodge  J swing  K shoot  F dig  E open/extract  R take  1-4 kit  L lantern", Vector2(0, 298), UiTheme.DUST)
	var quick := HBoxContainer.new()
	quick.position = Vector2(480, 250)
	quick.name = "Quick"
	c.add_child(quick)
	for id in KIT:
		var f := Panel.new()
		f.custom_minimum_size = Vector2(36, 36)
		quick.add_child(f)
		make_slot(f, icon_for(id), Vector2(2, 2))
	loot_panel = make_frame(c, Vector2(392, 40), Vector2(240, 200))
	loot_panel.visible = false
	loot_title = make_label(loot_panel, "", Vector2(6, 4), UiTheme.CYAN)
	loot_lines = VBoxContainer.new()
	loot_lines.position = Vector2(6, 18)
	loot_lines.add_theme_constant_override("separation", 1)
	loot_panel.add_child(loot_lines)
	loot_foot = make_label(loot_panel, "", Vector2(6, 180), UiTheme.DUST)

## The input word for this tick, from the keyboard (or a script).
func _gather() -> Array:
	if not scripted.is_empty():
		return scripted
	var b := 0
	var dx := 0
	var dy := 0
	if Input.is_key_pressed(KEY_A): dx -= 1
	if Input.is_key_pressed(KEY_D): dx += 1
	if Input.is_key_pressed(KEY_W): dy -= 1
	if Input.is_key_pressed(KEY_S): dy += 1
	if Input.is_key_pressed(KEY_SHIFT): b |= B_SPRINT
	if Input.is_key_pressed(KEY_CTRL): b |= B_SNEAK
	if Input.is_key_pressed(KEY_Q): b |= B_DIM
	if Input.is_key_pressed(KEY_J): b |= B_ATTACK
	if Input.is_key_pressed(KEY_K): b |= B_SHOOT
	if Input.is_key_pressed(KEY_SPACE): b |= B_DODGE
	if Input.is_key_pressed(KEY_E): b |= B_INTERACT
	if Input.is_key_pressed(KEY_F): b |= B_DIG
	if Input.is_key_pressed(KEY_R): b |= B_TAKE
	if lantern_pressed: b |= B_LANTERN
	var quick := 0
	for i in 4:
		if Input.is_key_pressed(KEY_1 + i):
			quick = i + 1
	b |= (quick & 7) << 10
	return [b, dx, dy]

## Step the simulation with an explicit word (the smoke test drives this).
func step_with(buttons: int, dx: int, dy: int) -> void:
	route.step(buttons, dx, dy)
	state = route.runner()
	_handle_events()
	_sync_view()

func _process(delta: float) -> void:
	clock += delta
	if over != "":
		over_t += delta
		if over_t > 3.0:
			if over == "extracted":
				Screens.show_screen("stash")
			else:
				Screens.show_screen("colony")
		return
	# presses are latched between ticks so a tap shorter than a tick (50 ms) is never dropped
	var now := _gather()
	latched |= int(now[0])
	sim_accum += delta
	var period := 1.0 / SIM_HZ
	var steps := 0
	while sim_accum >= period and steps < 4:
		sim_accum -= period
		var w := _gather()
		step_with(int(w[0]) | latched, w[1], w[2])
		latched = 0
		lantern_pressed = false
		steps += 1
	# manifest unroll
	if loot_panel.visible and loot_shown < loot_items.size():
		loot_t += delta
		var last_notable := loot_shown == loot_items.size() - 1 and String(loot_items[loot_shown].tier) in ["rare", "blackline", "relic"]
		if loot_t >= UNROLL * (2.5 if last_notable else 1.0):
			loot_t = 0.0
			_add_loot_line(loot_items[loot_shown])
			loot_shown += 1
			if loot_shown == loot_items.size():
				loot_foot.text = "R take all   walk away to close"
	# flashes fade
	var i := flashes.size() - 1
	while i >= 0:
		flashes[i][1] -= delta
		if flashes[i][1] <= 0.0:
			flashes[i][0].queue_free()
			flashes.remove_at(i)
		else:
			flashes[i][0].modulate.a = clampf(flashes[i][1] * 2.0, 0.0, 1.0)
		i -= 1
	# runner animation
	anim_t += delta
	if anim_t > (0.08 if state.get("sprinting", false) else 0.12):
		anim_t = 0.0
		if state.get("moved", false):
			frame = (frame + 1) % 8

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_L:
			lantern_pressed = true
		elif event.keycode == KEY_TAB:
			Screens.show_screen("stash")

func _handle_events() -> void:
	_last_events = route.events()
	for e in _last_events:
		match String(e.kind):
			"opened":
				_refresh_sources()
				loot_open = int(e.source)
				loot_items = route.source_items(loot_open)
				loot_shown = 0
				loot_t = 0.0
				for ch in loot_lines.get_children():
					ch.queue_free()
				var src := _source(loot_open)
				var how := "forced" if bool(e.forced) else "opened"
				loot_title.text = "%s %s   tier %d   %d items%s" % [String(src.get("kind", "")).to_upper(), how, int(src.get("tier", 0)), int(e.revealed), "   FIRST-RUN FIND" if bool(e.generous) else ""]
				loot_foot.text = "..." if int(e.revealed) > 0 else "nothing in it"
				loot_panel.visible = true
			"took":
				_refresh_sources()
				loot_items = route.source_items(loot_open)
				loot_shown = loot_items.size()
				_redraw_loot()
				var left := int(e.left)
				loot_foot.text = ("took %d   %d left: no room" % [int(e.taken), left]) if left > 0 else ("took %d" % int(e.taken))
			"dug":
				world.repaint_tile(int(e.x), int(e.y), int(route.substrate()[int(e.y) * world.w + int(e.x)]))
				_refresh_sources()
			"runner_hit":
				_flash(runner.position + Vector2(0, -24), "-%d %s" % [int(e.damage), String(e.section)], UiTheme.HOT_PINK)
				runner.modulate = Color(1.0, 0.5, 0.6)
			"bleed":
				_flash(runner.position + Vector2(0, -24), "bleeding", UiTheme.HOT_PINK)
			"creature_hit":
				var n: Sprite2D = creature_nodes.get(int(e.id))
				if n != null:
					_flash(n.position + Vector2(0, -20), "%d%s" % [int(e.damage), "!" if bool(e.behind) else ""], UiTheme.ACID)
					n.modulate = Color(1.3, 1.3, 1.3)
			"creature_died":
				_flash(Vector2(int(e.x) * 32 + 16, int(e.y) * 32), "%s killed" % String(e.creature), UiTheme.ACID)
				_refresh_sources()
			"swing":
				var r := ColorRect.new()
				r.color = UiTheme.CHALK
				r.position = Vector2(int(e.x) * 32 + 4, int(e.y) * 32 + 12)
				r.size = Vector2(24, 2)
				r.z_index = 15
				add_child(r)
				flashes.append([r, 0.15])
			"shot":
				var line := Line2D.new()
				line.width = 1.0
				line.default_color = UiTheme.ACID if bool(e.by_runner) else UiTheme.VESTMENT
				line.add_point(Vector2(int(e.x0) * 32 + 16, int(e.y0) * 32 + 12))
				line.add_point(Vector2(int(e.x1) * 32 + 16, int(e.y1) * 32 + 12))
				line.z_index = 15
				add_child(line)
				flashes.append([line, 0.12])
			"burst":
				_flash(Vector2(int(e.x) * 32 + 16, int(e.y) * 32), "BURST", UiTheme.OOZE_GLOW)
			"salted":
				_paint_salt()
			"phase":
				var n2: Sprite2D = creature_nodes.get(int(e.id))
				if n2 != null and String(e.phase) in ["alert", "suspicious"]:
					_flash(n2.position + Vector2(0, -20), "!" if e.phase == "alert" else "?", UiTheme.VESTMENT)
			"down":
				_flash(runner.position + Vector2(0, -28), "DOWN", UiTheme.HOT_PINK)
			"dead":
				over = "died"
				over_t = 0.0
				_flash(runner.position + Vector2(0, -28), "RUNNER DEAD", UiTheme.HOT_PINK)
			"extracted":
				over = "extracted"
				over_t = 0.0
				Sim.pending_haul = route.rig_export()
				Sim.pending_haul_from = "S%d route %d" % [stratum, seed_value]
				Sim.note_run(stratum, seed_value)
				_flash(runner.position + Vector2(0, -28), "EXTRACTED", UiTheme.OOZE_GLOW)

func _flash(pos: Vector2, text: String, color: Color) -> void:
	var l := Label.new()
	l.text = text
	l.position = pos + Vector2(-text.length() * 2, 0)
	l.add_theme_color_override("font_color", color)
	l.theme = UiTheme.build()
	labels.add_child(l)
	flashes.append([l, 0.9])

func _paint_salt() -> void:
	var s: PackedByteArray = route.salt()
	for i in s.size():
		if s[i] != 0 and (i >= last_salt.size() or last_salt[i] == 0):
			world.put_prop(i % world.w, i / world.w, "tile.salt", "", i % 2)
	last_salt = s

func _sync_view() -> void:
	var px := float(state.get("x", 0)) / 256.0 * WorldView.TILE
	var py := float(state.get("y", 0)) / 256.0 * WorldView.TILE
	runner.position = Vector2(px, py)
	runner.flip_h = int(state.get("facing", 1)) < 0
	var pose := String(state.get("pose", "walk"))
	if pose != runner_pose:
		runner_pose = pose
		if pose == "walk" or pose == "wade":
			runner.texture = Assets.texture("slimo.d2.walk")
			runner.offset = Vector2(-16, -40)
		else:
			runner.texture = Assets.texture("slimo.d2.crawl")
			runner.offset = Vector2(-24, -24)
		frame = 0
	var rfw := 32 if runner_pose == "walk" or runner_pose == "wade" else 48
	var rfh := 48 if rfw == 32 else 32
	runner.region_rect = Rect2(frame * rfw, 0, rfw, rfh)
	runner.modulate = runner.modulate.lerp(Color.WHITE, 0.3)
	if String(state.get("life", "up")) != "up":
		runner.rotation_degrees = 90
	cam.position = runner.position
	fog.feed(route.overlay_rgb())
	# creatures
	var seen := {}
	for c in route.creatures():
		var id := int(c.id)
		seen[id] = true
		var n: Sprite2D = creature_nodes.get(id)
		if n == null:
			n = Sprite2D.new()
			n.texture = Assets.texture(String(c.sprite))
			n.region_enabled = true
			n.centered = false
			n.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			n.z_index = 4
			add_child(n)
			creature_nodes[id] = n
		var tex := n.texture
		var fh := tex.get_height() if tex != null else 32
		var fw := 32 if fh <= 32 else (48 if fh == 64 else 64)
		if String(c.kind) == "maw":
			fw = 64
		var frames := maxi(1, int(tex.get_width() / fw)) if tex != null else 1
		var f := int(clock * 6.0) % frames
		n.region_rect = Rect2(f * fw, 0, fw, fh)
		n.offset = Vector2(-fw / 2.0, -fh + 8)
		n.position = Vector2(float(c.x) / 256.0 * WorldView.TILE, float(c.y) / 256.0 * WorldView.TILE)
		n.flip_h = int(c.facing) < 0
		var vis := bool(c.visible)
		n.visible = vis or bool(c.heard)
		n.modulate = n.modulate.lerp(Color.WHITE if vis else Color(0.25, 0.25, 0.3, 0.6), 0.3)
		if bool(c.windup):
			n.modulate = Color(1.0, 0.8, 0.8)
	for id in creature_nodes.keys():
		if not seen.has(id):
			creature_nodes[id].queue_free()
			creature_nodes.erase(id)
	# the loot panel closes when the Runner leaves the source
	if loot_open >= 0 and int(state.get("source_here", -1)) != loot_open:
		loot_panel.visible = false
		loot_open = -1
	_hud()

func _source(idx: int) -> Dictionary:
	for s in sources:
		if int(s.index) == idx:
			return s
	return {}

func _redraw_loot() -> void:
	for ch in loot_lines.get_children():
		ch.queue_free()
	var s := _source(loot_open)
	loot_title.text = "%s   tier %d   %d items" % [String(s.get("kind", "")).to_upper(), int(s.get("tier", 0)), loot_items.size()]
	for k in mini(loot_shown, loot_items.size()):
		_add_loot_line(loot_items[k])

func _add_loot_line(it: Dictionary) -> void:
	var l := Label.new()
	l.text = "%-30s %dx%d %4.1fkg c%d" % [String(it.label).left(30), int(it.w), int(it.h), float(it.weight), int(it.condition)]
	l.add_theme_color_override("font_color", TIER_COLOR.get(String(it.tier), UiTheme.DUST))
	loot_lines.add_child(l)

func _hud() -> void:
	var act := String(state.get("action", "idle"))
	var prog := int(float(state.get("action_progress", 0.0)) * 100.0)
	var where := ""
	var src := int(state.get("source_here", -1))
	if src >= 0:
		var sd := _source(src)
		var st := String(sd.get("state", ""))
		if st == "closed":
			where = "%s%s: E open" % [String(sd.get("kind", "")).to_upper(), " (LOCKED)" if bool(sd.get("locked", false)) else ""]
		elif st == "open":
			where = "%s open: R take" % String(sd.get("kind", "")).to_upper()
	elif bool(state.get("on_extraction", false)):
		where = "ON THE MAT: hold E to extract"
	var pace := "still"
	if bool(state.get("moved", false)):
		pace = "sneaking" if bool(state.get("sneaking", false)) else ("SPRINTING" if bool(state.get("sprinting", false)) else String(state.get("pose", "walk")) + "ing")
	var action := ""
	if act != "idle":
		action = "   %s %d%%" % [act.to_upper(), prog]
	var cs: PackedInt32Array = route.run_stats()
	hud.text = "run %02d:%02d   tile %d,%d   %s   %s%s   %d creatures" % [int(clock) / 60, int(clock) % 60, int(state.get("tx", 0)), int(state.get("ty", 0)), where, pace, action, cs[6] if cs.size() > 6 else 0]
	gem.text = "lume %d   sees %d   exposure %d %s   noticed within %d   noise here %d%s%s" % [int(state.get("lume", 0)), int(state.get("see_radius", 0)), int(state.get("exposure", 0)), String(state.get("exposure_band", "")).to_upper(), int(state.get("notice_range", 0)), int(state.get("noise_here", 0)), "   VEINS DIMMED" if bool(state.get("dimming", false)) else "", "" if bool(state.get("lantern", true)) else "   lantern off"]
	rig_line.text = "rig %d items  %d free  %.1f/%.0f kg%s   blind value %d e   %s" % [int(state.get("rig_count", 0)), int(state.get("rig_free", 0)), float(state.get("weight", 0.0)), float(state.get("carry", 0.0)), "  OVERWEIGHT" if bool(state.get("overweight", false)) else "", int(state.get("blind_value", 0)), String(state.get("melee", ""))]
	var secs: PackedInt32Array = state.get("sections", PackedInt32Array())
	var maxs: PackedInt32Array = state.get("max_sections", PackedInt32Array())
	for i in 4:
		if secs.size() == 4:
			var l: Label = body_lines[i]
			l.text = "%-5s %2d/%2d %s" % [SECTION_LABEL[i], secs[i], maxs[i], "#".repeat(int(ceil(float(secs[i]) / maxf(1.0, float(maxs[i])) * 10.0)))]
			l.add_theme_color_override("font_color", UiTheme.HOT_PINK if secs[i] == 0 else (UiTheme.DUST if secs[i] * 2 < maxs[i] else UiTheme.CHALK))
	var st_fill: float = float(state.get("stamina", 100.0)) / maxf(1.0, float(state.get("max_stamina", 100.0)))
	if stamina_fill.get_child_count() >= 2:
		var fill: Control = stamina_fill.get_child(1)
		fill.size = Vector2(96 * clampf(st_fill, 0.0, 1.0), 8)
		fill.custom_minimum_size = fill.size
	var statuses: PackedStringArray = state.get("statuses", PackedStringArray())
	var life := String(state.get("life", "up"))
	status_line.text = ("DOWN  %d s" % int(int(state.get("down_left", 0)) / 20) if life == "down" else ("DEAD" if life == "dead" else "")) + ("  " + "  ".join(statuses).to_upper() if statuses.size() > 0 else "")
