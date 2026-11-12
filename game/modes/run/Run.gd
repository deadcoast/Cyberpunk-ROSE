extends ScreenBase
## Expedition: the view and controller of a run. Every rule lives in Rust (RoseRun over run::sim::RunSim): this
## screen gathers the input word at 20 Hz, steps the simulation, and draws its state and events — the route and
## its props, the Runner at a sub-tile position, creatures, followers, the fog/light/noise overlay, and the Run HUD
## of Full Design v3.3: the senses on the Runner (exposure and noise arcs, the notice ring, the channel ring), the
## shift clock and route strip, the body silhouette, stamina and statuses, the hand and quick slots, follower
## chips, one prompt line, and the grid-to-grid loot panel. The Route Board hands the route, crew and rig over in
## Sim.launch; without it (tests) the run starts on its own defaults.

const SIM_HZ := 20.0
const UNROLL := 0.22
const KIT := ["ger_lantern", "org_antiseptic", "prt_salt", "org_stim"]
const WEAPON := "ger_pipe"
const BODY := 5
const SECTION_LABEL := ["HEAD", "TORSO", "ARMS", "LEGS"]
const CELL := 32
const HINT_S := 6.0
const LOOT_CAM_SHIFT := 120.0
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
const B_DROP := 32768
const ORDER_SHIFT := 13
## A run without the Route Board (tests): two plain followers and the plain Runner (RosePeople.plain_runner).
const FOLLOWERS := [{"id": 101, "name": "Pell", "body": 5, "damage": 4, "back_off_pct": 30, "bonded": false}, {"id": 102, "name": "Rook", "body": 4, "damage": 4, "back_off_pct": 30, "bonded": false}]
const RUNNER_NAME := "the Runner"

var world: WorldView
var fog: FogOverlay
var cam: Camera2D
var runner: Sprite2D
var route: RoseRun
var seed_value := 11
var stratum := 1
var route_name := "SUMP EAST"
var runner_name := RUNNER_NAME
var crew := {}                    # {runner_id, followers} from the Route Board; empty for a run without records
var sim_accum := 0.0
var clock := 0.0
var frame := 0
var anim_t := 0.0
var sources := []
var creature_nodes := {}      # id -> Sprite2D
var follower_nodes := {}      # id -> Sprite2D
var order_pressed := 0
var flashes := []             # [node, t]
var labels: Node2D
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
# ---- the HUD
var rings: Node2D                 # exposure and noise arcs, notice ring, channel ring, in world space
var water_fx: Node2D              # a Flood shift's risen water in Flood Light, its surface a Cyan line
var risen := {}                   # tile -> true: what the Flood turned to Water this run
var ring_exposure := 0.0
var ring_noise := 0.0
var noise_peak := 0.0
var noise_peak_t := 0.0
var ring_notice := 0.0
var ring_channel := -1.0
var ring_channel_color := UiTheme.OOZE_GLOW
var clock_label: Label
var route_label: Label
## Ivan over the comm (Full Design v3.19: barks during runs are his), under the route strip
var bark_label: Label
var kills_label: Label
var strip: Control
var strip_cells: Array[ColorRect] = []
var body_fills := {}              # section -> [ColorRect, ...]
var stamina_fill: Control
var status_tags: HBoxContainer
var down_label: Label
var prompt_frame: Panel
var prompt_key: Label
var prompt_verb: Label
var hint_label: Label
var hand_slot: Control
var quick_slot_nodes: Array[Control] = []
var quick_icons: Array[AssetSlot] = []
var hand_icon: AssetSlot
var quick_ids := PackedStringArray()
var hand_id := ""
var slot_flash := {}              # slot index -> t
var chips: Control
var chip_nodes := {}              # follower id -> {name, bar, order}
var summary: Label
var summary_frame: Panel
var overtime_flash_t := 0.0
var kills_seen := 0
var cadence_path := ""
# ---- the loot panel
var loot_panel: Panel
var loot_title: Label
var loot_found: Control
var loot_rig: Control
var loot_foot: Label
var loot_open := -1
var loot_items := []
var loot_shown := 0
var loot_t := 0.0
var loot_no_room := {}            # item k -> true after a take left it behind
var cam_shift := 0.0

func _ready() -> void:
	var launch: Dictionary = Sim.launch
	if not launch.is_empty():
		stratum = int(launch.get("stratum", stratum))
		seed_value = int(launch.get("seed", seed_value))
		route_name = String(launch.get("name", route_name))
		runner_name = String(launch.get("runner_name", runner_name))
		crew = {"runner_id": int(launch.get("runner_id", -1)), "followers": launch.get("followers", [])}
		if int(crew.runner_id) < 0:
			crew = {}
	screen_title = "RUN  -  S%d %s" % [stratum, route_name]
	# the simulation first: the HUD is built from the route (chunk count) in _build_content
	route = RoseRun.new()
	route.set_mat_lume(Sim.ensure().gene(4))
	if route.assemble(seed_value, stratum, 0) == 0:
		push_error("Run: no route assembled")
	if memory_bits.is_empty():
		memory_bits = Sim.memory_of(stratum, seed_value)
	if not memory_bits.is_empty():
		route.restore_memory(memory_bits)
	route.set_shift(Sim.day, Sim.flood_roll(), Sim.next_flood_roll())
	var rig: PackedByteArray = launch.get("rig", PackedByteArray())
	var followers: Array = launch.get("followers", FOLLOWERS)
	var runner_setup: Dictionary = launch.get("runner", RosePeople.plain_runner(BODY))
	if rig.is_empty():
		route.begin_run(Sim.run_nonce(), Sim.runs_of(stratum, seed_value), Sim.day, PackedStringArray(KIT), WEAPON, runner_setup, followers, Sim.corpses_on(stratum, seed_value))
	else:
		route.begin_run_rig(Sim.run_nonce(), Sim.runs_of(stratum, seed_value), Sim.day, rig, runner_setup, followers, Sim.corpses_on(stratum, seed_value))
	super._ready()
	nav.visible = false
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
	rings = Node2D.new()
	rings.z_index = 6
	rings.draw.connect(_draw_rings)
	water_fx = Node2D.new()
	water_fx.z_index = 1
	water_fx.draw.connect(_draw_water)
	add_child(water_fx)
	add_child(rings)
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
	# top: the route, the shift clock over the route strip, kills — each in a small frame so it reads over lit tiles
	var tl := make_frame(c, Vector2(0, 0), Vector2(120, 14))
	route_label = make_label(tl, "", Vector2(4, 2), UiTheme.DUST)
	bark_label = make_label(c, "", Vector2(2, 16), UiTheme.OOZE_GLOW)
	var strip_w := maxi(64, route.chunk_count() * 8)
	var tc := make_frame(c, Vector2(316 - strip_w / 2 - 4, 0), Vector2(strip_w + 8, 22))
	clock_label = make_label(tc, "", Vector2(4, 2), UiTheme.CYAN)
	strip = Control.new()
	strip.position = Vector2(4, 13)
	tc.add_child(strip)
	var tr := make_frame(c, Vector2(548, 0), Vector2(84, 14))
	kills_label = make_label(tr, "", Vector2(4, 2), UiTheme.CHALK)
	for i in route.chunk_count():
		var cell := ColorRect.new()
		cell.position = Vector2(i * 8, 0)
		cell.size = Vector2(6, 4)
		cell.color = UiTheme.SUMP
		strip.add_child(cell)
		strip_cells.append(cell)
	# bottom-left: the body silhouette, stamina, statuses, in one frame
	var bl := make_frame(c, Vector2(0, 250), Vector2(230, 46))
	var body := Control.new()
	body.position = Vector2(4, 4)
	bl.add_child(body)
	# [section, x, y, w, h]: head, torso, two arms, two legs
	for part in [[0, 8, 0, 8, 8], [1, 6, 9, 12, 16], [2, 1, 9, 4, 14], [2, 19, 9, 4, 14], [3, 6, 26, 5, 12], [3, 13, 26, 5, 12]]:
		var back := ColorRect.new()
		back.position = Vector2(part[1], part[2])
		back.size = Vector2(part[3], part[4])
		back.color = UiTheme.SUMP
		body.add_child(back)
		var fill := ColorRect.new()
		fill.position = back.position
		fill.size = back.size
		fill.color = UiTheme.OOZE_GLOW
		fill.set_meta("part", [part[1], part[2], part[3], part[4]])
		body.add_child(fill)
		if not body_fills.has(part[0]):
			body_fills[part[0]] = []
		body_fills[part[0]].append(fill)
	stamina_fill = make_gauge(bl, "ui.gauge.ooze", Vector2(32, 6), 1.0, "")
	make_label(bl, "STAMINA", Vector2(132, 4), UiTheme.DUST)
	status_tags = HBoxContainer.new()
	status_tags.position = Vector2(32, 18)
	status_tags.add_theme_constant_override("separation", 6)
	bl.add_child(status_tags)
	down_label = make_label(bl, "", Vector2(32, 30), UiTheme.HOT_PINK)
	# bottom centre: the prompt in its own frame (hidden when empty) and the first-six-seconds hint
	prompt_frame = make_frame(c, Vector2(250, 278), Vector2(100, 14))
	prompt_key = make_label(prompt_frame, "", Vector2(4, 2), UiTheme.CYAN)
	prompt_verb = make_label(prompt_frame, "", Vector2(4, 2), UiTheme.CHALK)
	hint_label = make_label(c, "WASD move  SHIFT sprint  CTRL sneak  Q dim veins  SPACE dodge  J swing  K shoot  F dig  E open/extract  R take  V drop  1-4 kit  L lantern  Z/X/C orders", Vector2(0, 298), UiTheme.DUST)
	# bottom-right: the hand and the quick slots
	hand_slot = _hud_slot(c, Vector2(432, 254), "J")
	hand_icon = hand_slot.get_meta("icon")
	for i in 4:
		var sl := _hud_slot(c, Vector2(476 + i * 40, 254), str(i + 1))
		quick_slot_nodes.append(sl)
		quick_icons.append(sl.get_meta("icon"))
	# right edge: follower chips
	chips = Control.new()
	chips.position = Vector2(556, 100)
	c.add_child(chips)
	# the end-of-run summary, centred in its own frame when the run is over
	summary_frame = make_frame(c, Vector2(116, 150), Vector2(400, 18))
	summary_frame.visible = false
	summary = make_label(summary_frame, "", Vector2(6, 4), UiTheme.CHALK)
	# the loot panel: found on the left, the rig grid on the right
	loot_panel = make_frame(c, Vector2(256, 46), Vector2(376, 200))
	loot_panel.visible = false
	loot_title = make_label(loot_panel, "", Vector2(6, 4), UiTheme.CYAN)
	loot_found = Control.new()
	loot_found.position = Vector2(6, 18)
	loot_panel.add_child(loot_found)
	loot_rig = Control.new()
	loot_rig.position = Vector2(110, 18)
	loot_panel.add_child(loot_rig)
	loot_foot = make_label(loot_panel, "", Vector2(6, 182), UiTheme.DUST)

## A 36 x 36 slot frame with its key and an icon holder.
func _hud_slot(parent: Control, pos: Vector2, key: String) -> Control:
	var f := make_frame(parent, pos, Vector2(36, 36))
	var icon := AssetSlot.new()
	icon.position = Vector2(2, 2)
	f.add_child(icon)
	f.set_meta("icon", icon)
	make_label(f, key, Vector2(2, -1), UiTheme.DUST)
	return f

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
	if Input.is_key_pressed(KEY_V): b |= B_DROP
	if Input.is_key_pressed(KEY_Z): order_pressed = 1
	elif Input.is_key_pressed(KEY_X): order_pressed = 2
	elif Input.is_key_pressed(KEY_C): order_pressed = 3
	b |= (order_pressed & 3) << ORDER_SHIFT
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
		order_pressed = 0
		steps += 1
	var line: String = Sim.ensure().bark()
	bark_label.text = ("IVAN: " + line) if line != "" else ""
	# manifest unroll: one tile per beat, the notable last on a longer one
	if loot_panel.visible and loot_shown < loot_items.size():
		loot_t += delta
		var last_notable := loot_shown == loot_items.size() - 1 and String(loot_items[loot_shown].tier) in ["rare", "blackline", "relic"]
		if loot_t >= UNROLL * (2.5 if last_notable else 1.0):
			loot_t = 0.0
			loot_shown += 1
			_redraw_found()
			if loot_shown == loot_items.size():
				loot_foot.text = "R  TAKE ALL   -   walk away to close"
	# the camera eases east while the loot panel is open so the Runner stays clear of it
	cam_shift = lerpf(cam_shift, LOOT_CAM_SHIFT if loot_panel.visible else 0.0, minf(1.0, delta * 8.0))
	cam.position = runner.position + Vector2(cam_shift, 0)
	# the hint fades after the first seconds; a used quick slot and overtime flash
	if hint_label.visible:
		hint_label.modulate.a = clampf((HINT_S - clock) / 1.5, 0.0, 1.0)
		if clock > HINT_S:
			hint_label.visible = false
	for k in slot_flash.keys():
		slot_flash[k] -= delta
		var node: Control = hand_slot if k == -1 else quick_slot_nodes[k]
		node.modulate = Color.WHITE.lerp(UiTheme.ACID, clampf(slot_flash[k] * 3.0, 0.0, 1.0))
		if slot_flash[k] <= 0.0:
			slot_flash.erase(k)
	if overtime_flash_t > 0.0:
		overtime_flash_t -= delta
	# the noise arc holds its peak for 0.3 s so a footstep reads at 20 Hz
	noise_peak_t -= delta
	if noise_peak_t <= 0.0:
		noise_peak = 0.0
	rings.position = runner.position
	rings.queue_redraw()
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
				loot_no_room.clear()
				var src := _source(loot_open)
				var how := "FORCED" if bool(e.forced) else "OPEN"
				loot_title.text = "%s %s   T%d   %d%s" % [String(src.get("kind", "")).to_upper(), how, int(src.get("tier", 0)), int(e.revealed), "   FIRST-RUN FIND" if bool(e.generous) else ""]
				loot_foot.text = "..." if int(e.revealed) > 0 else "nothing in it   -   walk away to close"
				loot_panel.visible = true
				_redraw_found()
				_redraw_rig_grid()
			"took":
				_refresh_sources()
				loot_items = route.source_items(loot_open)
				loot_shown = loot_items.size()
				var left := int(e.left)
				for k in loot_items.size():
					loot_no_room[k] = true
				_redraw_found()
				_redraw_rig_grid()
				loot_foot.text = ("took %d   %d left: NO ROOM" % [int(e.taken), left]) if left > 0 else ("took %d" % int(e.taken))
			"used":
				slot_flash[int(e.slot)] = 0.4
			"overtime":
				overtime_flash_t = 1.5
				var what := []
				if int(e.packs) > 0:
					what.append("%d more from a burrow" % int(e.packs))
				if int(e.patrols) > 0:
					what.append("a patrol")
				_flash(runner.position + Vector2(0, -40), "OVERTIME +%d  %s" % [int(e.minute), ", ".join(what)], UiTheme.HOT_PINK)
			"dug":
				world.repaint_tile(int(e.x), int(e.y), int(route.substrate()[int(e.y) * world.w + int(e.x)]))
				_refresh_sources()
			"water_rose":
				# the Flood shift's water: repaint what turned to Water
				var sub: PackedByteArray = route.substrate()
				for i in e.tiles:
					world.repaint_tile(int(i) % world.w, int(i) / world.w, int(sub[int(i)]))
					risen[int(i)] = true
				water_fx.queue_redraw()
				_flash(runner.position + Vector2(0, -40), "THE WATER RISES  +%d" % int(e.level), UiTheme.CYAN)
			"raiders":
				_flash(Vector2(int(e.x) * 32 + 16, int(e.y) * 32 - 8), "RAIDERS x%d" % int(e.n), UiTheme.MUZZLE)
			"raided":
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
				kills_seen += 1
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
				Sim.ensure().bark_event("runner_down", String(Sim.launch.get("runner_name", "the Runner")))
			"dead":
				over = "died"
				over_t = 0.0
				_flash(runner.position + Vector2(0, -28), "RUNNER DEAD", UiTheme.HOT_PINK)
				_finish_run()
			"extracted":
				over = "extracted"
				over_t = 0.0
				_flash(runner.position + Vector2(0, -28), "EXTRACTED", UiTheme.OOZE_GLOW)
				_finish_run()
			"follower_hit":
				var fn: Sprite2D = follower_nodes.get(int(e.id))
				if fn != null:
					_flash(fn.position + Vector2(0, -36), "-%d" % int(e.damage), UiTheme.HOT_PINK)
					fn.modulate = Color(1.0, 0.5, 0.6)
			"follower_down":
				var fn2: Sprite2D = follower_nodes.get(int(e.id))
				if fn2 != null:
					_flash(fn2.position + Vector2(0, -36), "DOWN", UiTheme.HOT_PINK)
			"follower_strike":
				var fn3: Sprite2D = follower_nodes.get(int(e.id))
				if fn3 != null:
					_flash(fn3.position + Vector2(0, -36), "%d" % int(e.damage), UiTheme.ACID)
			"ordered":
				_flash(runner.position + Vector2(0, -28), String(e.order).to_upper(), UiTheme.CHALK)
			"picked_up":
				_flash(runner.position + Vector2(0, -28), "CARRIED", UiTheme.CHALK)
			"set_down":
				_flash(runner.position + Vector2(0, -28), "SET DOWN", UiTheme.DUST)
			"dropped":
				_refresh_sources()
				_flash(runner.position + Vector2(0, -28), "dropped", UiTheme.DUST)

## The run is over: hand its outcome to the colony and show the summary line.
func _finish_run() -> void:
	var o: Dictionary = route.outcome()
	if o.is_empty():
		return
	Sim.take_outcome(stratum, seed_value, runner_name, o, crew)
	var parts := []
	parts.append(String(o.result).to_upper())
	parts.append("%d kills" % int(o.kills))
	parts.append("%d chunks seen" % int(o.chunks_seen))
	if String(o.result) == "extracted":
		parts.append("rig worth %d e blind" % int(o.blind_value))
	elif o.has("corpse"):
		parts.append("body in %s" % String(o.corpse.chunk))
	if int(o.get("regrown_count", 0)) > 0:
		parts.append("%d Spored regrew" % int(o.regrown_count))
	if int(o.get("mat_haul_count", 0)) > 0:
		parts.append("the Mat hauled %d" % int(o.mat_haul_count))
	var fe: PackedInt32Array = o.get("followers_extracted", PackedInt32Array())
	var fs: PackedInt32Array = o.get("followers_stranded", PackedInt32Array())
	var fd: PackedInt32Array = o.get("followers_dead", PackedInt32Array())
	if fe.size() + fs.size() + fd.size() > 0:
		parts.append("followers: %d out, %d stranded, %d dead" % [fe.size(), fs.size(), fd.size()])
	var rep: Dictionary = o.get("report", {})
	if not rep.is_empty() and int(rep.get("pulls", 0)) >= 2:
		parts.append("a pull every %d s" % int(round(float(rep.pull_gap_median))))
	var lines: PackedStringArray = Sim.last_outcome.get("record_lines", PackedStringArray())
	for i in mini(lines.size(), 2):
		parts.append(String(lines[i]))
	summary.text = "   ".join(parts)
	summary.add_theme_color_override("font_color", UiTheme.OOZE_GLOW if String(o.result) == "extracted" else UiTheme.HOT_PINK)
	_write_cadence_log(o)
	summary_frame.size = Vector2(summary.text.length() * 4 + 12, 18)
	summary_frame.position = Vector2(316 - summary_frame.size.x / 2, 150)
	summary_frame.visible = true

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
	cam.position = runner.position + Vector2(cam_shift, 0)
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
	# followers
	var seen_f := {}
	for f in route.followers():
		var fid := int(f.id)
		var n: Sprite2D = follower_nodes.get(fid)
		if n == null:
			n = Sprite2D.new()
			n.texture = Assets.texture("slimo.d1.walk")
			n.region_enabled = true
			n.region_rect = Rect2(0, 0, 32, 48)
			n.centered = false
			n.offset = Vector2(-16, -40)
			n.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			n.z_index = 4
			add_child(n)
			follower_nodes[fid] = n
		n.position = Vector2(float(f.x) / 256.0 * WorldView.TILE, float(f.y) / 256.0 * WorldView.TILE)
		n.flip_h = int(f.facing) < 0
		n.region_rect = Rect2((int(clock * 8.0) % 8) * 32, 0, 32, 48)
		n.rotation_degrees = 0 if String(f.life) == "up" else 90
		n.modulate = n.modulate.lerp(Color.WHITE if String(f.life) == "up" else Color(0.5, 0.5, 0.55), 0.3)
		if bool(f.windup):
			n.modulate = Color(1.0, 0.8, 0.8)
		seen_f[fid] = true
		_chip(f)
	for id in chip_nodes.keys():
		if not seen_f.has(id):
			chip_nodes[id].frame.queue_free()
			chip_nodes.erase(id)
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

## The cadence log goes to the save folder, where the player can read it (Full Design, The cadence budget and its log).
func _write_cadence_log(o: Dictionary) -> void:
	var dir := "user://telemetry"
	DirAccess.make_dir_recursive_absolute(dir)
	var header := "route S%d %s  seed %d  day %d  runner %s  %s" % [stratum, route_name, seed_value, Sim.day, runner_name, String(o.get("result", ""))]
	var text: String = route.cadence_log(header)
	if text.is_empty():
		return
	cadence_path = "%s/run_%03d_S%d_%d.log" % [dir, Sim.day, stratum, seed_value]
	var f := FileAccess.open(cadence_path, FileAccess.WRITE)
	if f != null:
		f.store_string(text)
		f.close()

## A follower's chip at the right edge: name, HP bar, order letter; Hot Pink when down.
func _chip(f: Dictionary) -> void:
	var id := int(f.id)
	if not chip_nodes.has(id):
		var i := chip_nodes.size()
		var frame_ := make_frame(chips, Vector2(0, i * 24), Vector2(72, 20))
		var name := make_label(frame_, "", Vector2(4, 2), UiTheme.CHALK)
		var back := ColorRect.new()
		back.position = Vector2(4, 13)
		back.size = Vector2(52, 3)
		back.color = UiTheme.VOID
		frame_.add_child(back)
		var bar := ColorRect.new()
		bar.position = back.position
		bar.size = back.size
		bar.color = UiTheme.OOZE_GLOW
		frame_.add_child(bar)
		var order := make_label(frame_, "", Vector2(60, 9), UiTheme.DUST)
		chip_nodes[id] = {"frame": frame_, "name": name, "bar": bar, "order": order}
	var ch: Dictionary = chip_nodes[id]
	var life := String(f.life)
	var frac := float(f.hp) / maxf(1.0, float(f.max_hp))
	ch.name.text = String(f.name) + ("  CARRYING" if bool(f.carrying) else ("  " + life.to_upper() if life != "up" else ""))
	ch.name.add_theme_color_override("font_color", UiTheme.HOT_PINK if life != "up" else UiTheme.CHALK)
	ch.bar.size = Vector2(52 * clampf(frac, 0.0, 1.0), 3)
	ch.bar.color = UiTheme.HOT_PINK if frac <= 0.5 else UiTheme.OOZE_GLOW
	ch.order.text = String(f.order).left(1).to_upper()

## The senses drawn on the Runner: the exposure arc (left, Ooze Glow), the noise arc (right, Cyan), the notice
## ring at its true radius, and the channel ring while extracting or digging.
## The Flood's risen water, as the Colony draws it: Flood Light over the tile, a Cyan surface where air is above.
func _draw_water() -> void:
	if risen.is_empty():
		return
	var sub: PackedByteArray = route.substrate()
	for i in risen:
		var r := Rect2((i % world.w) * 32, (i / world.w) * 32, 32, 32)
		water_fx.draw_rect(r, Color(UiTheme.DRAIN_LIGHT, 0.55))
		water_fx.draw_rect(Rect2(r.position + Vector2(4, 12), Vector2(10, 1)), Color(UiTheme.CYAN, 0.6))
		water_fx.draw_rect(Rect2(r.position + Vector2(18, 22), Vector2(10, 1)), Color(UiTheme.CYAN, 0.6))
		if i - world.w >= 0 and int(sub[i - world.w]) != 5:
			water_fx.draw_rect(Rect2(r.position, Vector2(32, 2)), UiTheme.CYAN)

func _draw_rings() -> void:
	var r := 22.0
	var c := Vector2(0, -18)
	# tracks
	rings.draw_arc(c, r, PI * 0.5, PI * 1.5, 24, Color(UiTheme.SUMP, 0.8), 2.0)
	rings.draw_arc(c, r, -PI * 0.5, PI * 0.5, 24, Color(UiTheme.SUMP, 0.8), 2.0)
	if ring_exposure > 0.0:
		var a0 := PI * 1.5
		rings.draw_arc(c, r, a0, a0 - PI * clampf(ring_exposure, 0.0, 1.0), 24, UiTheme.OOZE_GLOW, 2.0)
	if ring_noise > 0.0:
		var b0 := -PI * 0.5
		rings.draw_arc(c, r, b0, b0 + PI * clampf(ring_noise, 0.0, 1.0), 24, UiTheme.CYAN, 2.0)
	if ring_notice > 0.0:
		# a dotted circle at the true notice range: dots about 6 px long, 6 px apart
		var nr := ring_notice * CELL
		var n := int(maxf(24.0, nr / 2.0))
		for k in n:
			if k % 2 == 0:
				var a := TAU * k / n
				rings.draw_arc(c, nr, a, a + TAU / n, 2, Color(UiTheme.DUST, 0.28), 1.0)
	if ring_channel >= 0.0:
		rings.draw_arc(c, r + 5, -PI * 0.5, -PI * 0.5 + TAU * clampf(ring_channel, 0.0, 1.0), 32, ring_channel_color, 3.0)

## An item as a tile at its real size; Hot Pink when it could not be taken.
func _item_tile(parent: Control, pos: Vector2, it: Dictionary, no_room: bool) -> void:
	make_item_tile(parent, pos, int(it.w), int(it.h), String(it.tier), String(it.id), bool(it.ident), CELL, UiTheme.HOT_PINK if no_room else Color.TRANSPARENT)

## The found side: the revealed items packed first-fit into a 3 x 5 cell grid.
func _redraw_found() -> void:
	for ch in loot_found.get_children():
		ch.queue_free()
	var used := []
	for y in 5:
		used.append([false, false, false])
	var hidden := 0
	for k in mini(loot_shown, loot_items.size()):
		var it: Dictionary = loot_items[k]
		var w := int(it.w)
		var h := int(it.h)
		var placed := false
		for y in 5:
			for x in 3:
				if placed or x + w > 3 or y + h > 5:
					continue
				var free := true
				for yy in range(y, y + h):
					for xx in range(x, x + w):
						if used[yy][xx]:
							free = false
				if free:
					for yy in range(y, y + h):
						for xx in range(x, x + w):
							used[yy][xx] = true
					_item_tile(loot_found, Vector2(x * CELL, y * CELL), it, loot_no_room.has(k))
					placed = true
		if not placed:
			hidden += 1
	if hidden > 0:
		make_label(loot_found, "+%d more" % hidden, Vector2(2, 150), UiTheme.CHALK)

## The rig side: the backpack grid with its placed items.
func _redraw_rig_grid() -> void:
	for ch in loot_rig.get_children():
		ch.queue_free()
	for y in 5:
		for x in 8:
			var cell := ColorRect.new()
			cell.position = Vector2(x * CELL, y * CELL)
			cell.size = Vector2(CELL - 1, CELL - 1)
			cell.color = Color8(23, 26, 38)
			loot_rig.add_child(cell)
	for it in route.rig_items():
		_item_tile(loot_rig, Vector2(int(it.x) * CELL, int(it.y) * CELL), it, false)
	make_label(loot_rig, "RIG  %d free" % int(state.get("rig_free", 0)), Vector2(200, 162), UiTheme.DUST)

func _set_prompt(key: String, verb: String, color: Color = UiTheme.CHALK) -> void:
	prompt_key.text = key
	prompt_verb.text = verb
	prompt_verb.position = Vector2(4 + key.length() * 4 + (8 if key != "" else 0), 2)
	prompt_verb.add_theme_color_override("font_color", color)
	var chars := key.length() + (2 if key != "" else 0) + verb.length()
	prompt_frame.visible = chars > 0
	prompt_frame.size = Vector2(chars * 4 + 8, 14)
	prompt_frame.position = Vector2(316 - prompt_frame.size.x / 2, 278)

func _hud() -> void:
	# the shift clock and the route strip
	var sh: Dictionary = route.shift()
	var left := int(sh.get("left_ticks", 0))
	var ot := int(sh.get("overtime_ticks", 0))
	if ot > 0:
		var os := ot / 20
		clock_label.text = "OVERTIME +%d:%02d" % [os / 60, os % 60]
		clock_label.add_theme_color_override("font_color", UiTheme.HOT_PINK if overtime_flash_t <= 0.0 or int(overtime_flash_t * 6.0) % 2 == 0 else UiTheme.VESTMENT)
	else:
		var ls := left / 20
		clock_label.text = "SHIFT %d:%02d" % [ls / 60, ls % 60]
		clock_label.add_theme_color_override("font_color", UiTheme.ACID if ls < 60 else UiTheme.CYAN)
	# the Flood shift: when the water comes, then how high it is
	var flood_rise := int(sh.get("flood_rise", -1))
	if flood_rise > 0:
		var into := int(sh.get("tick", 0)) / 20
		var flood_s := int(sh.get("flood_s", 0))
		if into < flood_s:
			var fin := flood_s - into
			clock_label.text += "   FLOOD +%d IN %d:%02d" % [flood_rise, fin / 60, fin % 60]
		else:
			clock_label.text += "   WATER +%d OF %d" % [int(sh.get("water", 0)), flood_rise]
	route_label.text = "S%d %s   %s" % [stratum, route_name, runner_name]
	var cs: PackedInt32Array = route.run_stats()
	kills_label.text = "KILLS %d   %d ALIVE" % [kills_seen, cs[6] if cs.size() > 6 else 0]
	var seen: PackedByteArray = route.chunk_seen()
	var here := int(state.get("chunk", -1))
	for i in strip_cells.size():
		var c: ColorRect = strip_cells[i]
		if i == here:
			c.color = UiTheme.CYAN
		elif i < seen.size() and seen[i] != 0:
			c.color = UiTheme.CHALK
		else:
			c.color = UiTheme.SUMP
	# the body
	var secs: PackedInt32Array = state.get("sections", PackedInt32Array())
	var maxs: PackedInt32Array = state.get("max_sections", PackedInt32Array())
	if secs.size() == 4:
		for sec in 4:
			var frac := float(secs[sec]) / maxf(1.0, float(maxs[sec]))
			for fill in body_fills.get(sec, []):
				var part: Array = fill.get_meta("part")
				var fh := int(round(part[3] * frac))
				fill.position = Vector2(part[0], part[1] + part[3] - fh)
				fill.size = Vector2(part[2], fh)
				if secs[sec] == 0:
					fill.color = UiTheme.HOT_PINK if int(clock * 4.0) % 2 == 0 else UiTheme.SUMP
					fill.size = Vector2(part[2], part[3])
					fill.position = Vector2(part[0], part[1])
				else:
					fill.color = UiTheme.HOT_PINK if frac <= 0.5 else UiTheme.OOZE_GLOW
	var st_fill: float = float(state.get("stamina", 100.0)) / maxf(1.0, float(state.get("max_stamina", 100.0)))
	if stamina_fill.get_child_count() >= 2:
		var fill: Control = stamina_fill.get_child(1)
		fill.size = Vector2(96 * clampf(st_fill, 0.0, 1.0), 8)
		fill.custom_minimum_size = fill.size
	# statuses as tags; Down counts down
	var statuses: PackedStringArray = state.get("statuses", PackedStringArray())
	var tags := []
	for st in statuses:
		tags.append(String(st).to_upper())
	if bool(state.get("overweight", false)):
		tags.append("OVERWEIGHT")
	if bool(state.get("dimming", false)):
		tags.append("DIMMED")
	if not bool(state.get("lantern", true)):
		tags.append("LANTERN OFF")
	var under := int(state.get("under", 0))
	if under > 0:
		tags.append("UNDER %d s" % int(ceil(float(int(state.get("drown", 300)) - under) / 20.0)))
	var have := []
	for ch in status_tags.get_children():
		have.append(ch.text)
	if have != tags:
		for ch in status_tags.get_children():
			ch.queue_free()
		for t in tags:
			var l := Label.new()
			l.text = t
			l.add_theme_color_override("font_color", UiTheme.HOT_PINK if t in ["BLEEDING", "BLINDED", "CHROME SHOCK", "FUNGAL FEVER"] or t.begins_with("UNDER") else (UiTheme.ACID if t == "OVERWEIGHT" else UiTheme.DUST))
			status_tags.add_child(l)
	var life := String(state.get("life", "up"))
	down_label.text = "DOWN  %d s" % int(int(state.get("down_left", 0)) / 20) if life == "down" else ("DEAD" if life == "dead" else "")
	# the senses on the Runner
	ring_exposure = float(state.get("exposure", 0)) / 15.0
	var nh := float(state.get("noise_here", 0))
	if nh > noise_peak:
		noise_peak = nh
		noise_peak_t = 0.3
	ring_noise = noise_peak / 15.0
	ring_notice = float(state.get("notice_range", 0))
	var act := String(state.get("action", "idle"))
	if act == "extract" or act == "dig":
		ring_channel = float(state.get("action_progress", 0.0))
		ring_channel_color = UiTheme.OOZE_GLOW if act == "extract" else UiTheme.DUST
	else:
		ring_channel = -1.0
	# the prompt
	var src := int(state.get("source_here", -1))
	if life != "up" or over != "":
		_set_prompt("", "")
		ring_channel = -1.0
	elif act == "extract":
		_set_prompt("HOLD E", "EXTRACTING  %d%%" % int(float(state.get("action_progress", 0.0)) * 100.0), UiTheme.OOZE_GLOW)
	elif act == "dig":
		_set_prompt("F", "DIGGING  %d%%" % int(float(state.get("action_progress", 0.0)) * 100.0))
	elif src >= 0:
		var sd := _source(src)
		var kind := String(sd.get("kind", "")).to_upper()
		match String(sd.get("state", "")):
			"closed":
				if bool(sd.get("locked", false)):
					_set_prompt("E", "PRY %s  (locked)" % kind, UiTheme.ACID)
				else:
					_set_prompt("E", "OPEN %s" % kind)
			"open":
				_set_prompt("R", "TAKE ALL")
			_:
				_set_prompt("", "%s  empty" % kind, UiTheme.DUST)
	elif bool(state.get("on_extraction", false)):
		_set_prompt("HOLD E", "EXTRACT")
	else:
		_set_prompt("", "")
	# the hand and the quick slots
	var hid := String(state.get("hand", ""))
	if hid != hand_id:
		hand_id = hid
		hand_icon.slot_id = icon_for(hid) if hid != "" else ""
		hand_icon.texture = null
		hand_icon.refresh()
	var q: PackedStringArray = route.quick_slots()
	if q != quick_ids:
		quick_ids = q
		for i in 4:
			var id := String(q[i]) if i < q.size() else ""
			quick_icons[i].slot_id = icon_for(id) if id != "" else ""
			quick_icons[i].texture = null
			quick_icons[i].refresh()
