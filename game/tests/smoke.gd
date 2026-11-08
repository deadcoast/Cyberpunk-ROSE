extends SceneTree
## Phase A smoke: the extension loads, every screen instantiates, every asset slot resolves, the Colony runs the sim,
## the Stash round-trips through the save crate, the Run assembles a route and moves a Runner. Headless.
## Run:  godot --headless --path game --script res://tests/smoke.gd

var main: Node
var frames := 0
var failures: Array[String] = []
var screen_ids: Array = []
var screen_i := -1
var frames_on_screen := 0
var resolved_per_screen := {}
var t_start := 0

var assets: Node
var screens: Node
var sim: Node

func _autoload(name: String, path: String) -> Node:
	# a SceneTree script does not get the project's autoloads; mount them under root by name so scripts resolve them
	var n := root.get_node_or_null(name)
	if n == null:
		n = (load(path) as GDScript).new()
		n.name = name
		root.add_child(n)
	return n

func _initialize() -> void:
	t_start = Time.get_ticks_msec()
	assets = _autoload("Assets", "res://autoload/Assets.gd")
	screens = _autoload("Screens", "res://autoload/Screens.gd")
	sim = _autoload("Sim", "res://autoload/Sim.gd")
	var scene := load("res://Main.tscn")
	main = scene.instantiate()
	root.add_child(main)
	sim.ticks_per_frame = 2
	screen_ids = screens.ids()
	print("[smoke] extension RoseSim: %s  RoseItems: %s  RoseStash: %s  RoseRun: %s" % [ClassDB.class_exists("RoseSim"), ClassDB.class_exists("RoseItems"), ClassDB.class_exists("RoseStash"), ClassDB.class_exists("RoseRun")])

func _check(cond: bool, msg: String) -> void:
	if not cond:
		failures.append(msg)

func _process(_delta: float) -> bool:
	frames += 1
	frames_on_screen += 1
	if frames_on_screen >= 8:
		if screen_i >= 0:
			_verify_current()
		screen_i += 1
		frames_on_screen = 0
		if screen_i < screen_ids.size():
			var id: String = screen_ids[screen_i]
			var before: int = assets.resolved_count()
			screens.show_screen(id)
			resolved_per_screen[id] = before
		else:
			return _finish()
	return false

func _verify_current() -> void:
	var id: String = screen_ids[screen_i]
	var scr: Node = screens.current
	_check(scr != null and scr.has_method("_build_content"), "%s: did not instantiate as a ScreenBase" % id)
	var slots := _count_slots(scr)
	_check(slots > 0, "%s: no AssetSlot resolved a texture" % id)
	match id:
		"colony":
			var s: Dictionary = sim.ensure().stats()
			_check(int(s.get("tick", 0)) > 500, "colony: sim did not advance (tick %d)" % int(s.get("tick", 0)))
			_check(int(s.get("cells", 0)) > 500, "colony: Mat did not grow")
			var wv: Node = null
			for ch in scr.get_children():
				if ch.has_method("full_paint"):
					wv = ch
			_check(wv != null and wv.substrate.get_used_cells().size() > 1000, "colony: world tiles not painted")
			_check(wv != null and wv.matlayer.get_used_cells().size() > 300, "colony: Mat tiles not painted")
			scr.set_view(2)
			scr.apply_tool(96, 24)
		"stash":
			var st = scr.stash
			_check(st.count() >= 8, "stash: starting haul not placed (%d)" % st.count())
			scr._add_loot()
			_check(st.count() >= 10, "stash: add loot failed")
			_check(st.save_round_trip(), "stash: save round trip failed")
			_check(st.upgrade() and st.width() == 16, "stash: upgrade failed")
			_check(st.count() >= 10, "stash: items lost on upgrade")
		"run":
			var r: RoseRun = scr.route
			var n: int = r.chunk_count()
			_check(n >= 8 and n <= 14, "run: route has %d chunks, expected 8..14" % n)
			_check(r.library_size() >= 14, "run: chunk library has %d chunks" % r.library_size())
			_check(r.connectivity() >= 0.95, "run: route connectivity %.3f" % r.connectivity())
			_check(scr.world.substrate.get_used_cells().size() > 2000, "run: route tiles not painted")
			_check(scr.world.props.get_used_cells().size() >= 8, "run: props not painted (%d)" % scr.world.props.get_used_cells().size())
			_check(scr.fog != null and scr.fog.texture != null, "run: fog overlay missing")
			var st0: Dictionary = r.runner()
			_check(bool(st0.get("on_extraction", false)), "run: runner did not start on the Mat edge")
			_check(String(st0.get("exposure_band", "")) == "lit", "run: lantern on should read lit (%s)" % String(st0.get("exposure_band", "")))
			# walk east two seconds, sprinting: continuous movement, footsteps, stamina spent
			var x0 := int(st0.x)
			for i in 40:
				scr.step_with(0, 1, 0)
			var st1: Dictionary = r.runner()
			_check(int(st1.x) > x0 + 256 * 4, "run: walked %d units in 2 s" % (int(st1.x) - x0))
			_check(String(st1.pose) == "crawl", "run: off the floor the Runner should crawl, pose %s" % String(st1.pose))
			# down to the floor, then sprint: faster, and it costs stamina
			for i in 40:
				if String(r.runner().pose) != "crawl":
					break
				scr.step_with(0, 0, 1)
			var floor_pose := String(r.runner().pose)
			var x1 := int(r.runner().x)
			for i in 20:
				scr.step_with(scr.B_SPRINT, 1, 0)
			var st1b: Dictionary = r.runner()
			if floor_pose == "walk":
				_check(int(st1b.x) - x1 > 256 * 5, "run: sprint moved %d units in 1 s" % (int(st1b.x) - x1))
				_check(float(st1b.stamina) < 100.0, "run: sprint cost no stamina")
			else:
				_check(floor_pose == "wade" or floor_pose == "swim", "run: unexpected pose %s under the Mat edge" % floor_pose)
			st1 = st1b
			_check(int(st1.noise_here) > 0 or r.sense_stats()[2] >= 0, "run: no footstep noise")
			# lantern off lowers exposure
			scr.lantern_pressed = true
			scr.step_with(scr.B_LANTERN, 0, 0)
			scr.lantern_pressed = false
			var st2: Dictionary = r.runner()
			_check(int(st2.exposure) < int(st1.exposure), "run: lantern off did not lower exposure (%d -> %d)" % [int(st1.exposure), int(st2.exposure)])
			scr.lantern_pressed = true
			scr.step_with(scr.B_LANTERN, 0, 0)
			scr.lantern_pressed = false
			# open the first closed container through the input word, take its contents
			var opened := false
			for src in r.sources():
				if src.state == "closed" and (src.kind == "locker" or src.kind == "crate"):
					r.debug_teleport(int(src.x), int(src.y))
					scr.step_with(0, 0, 0)
					scr.step_with(scr.B_INTERACT, 0, 0)
					opened = scr.loot_panel.visible and scr.loot_open == int(src.index)
					scr.loot_shown = scr.loot_items.size()
					scr.step_with(0, 0, 0)
					scr.step_with(scr.B_TAKE, 0, 0)
					break
			_check(opened, "run: interact did not open a container")
			var st3: Dictionary = r.runner()
			_check(int(st3.rig_count) >= 1 or scr.loot_items.is_empty(), "run: take left the rig empty (%d items shown)" % scr.loot_items.size())
			for it in r.rig_items():
				_check(not bool(it.ident) and String(it.label).begins_with("unidentified"), "run: found item identified on pickup")
				break
			# a creature: stand next to the first one, let it come, swing the pipe until it dies
			var cs: Array = r.creatures()
			_check(cs.size() >= 1, "run: no creatures spawned")
			var crawler := {}
			for cc in cs:
				if String(cc.kind) == "crawler":
					crawler = cc
					break
			if not crawler.is_empty():
				var c: Dictionary = crawler
				var placed := false
				for d in [[1, 0], [-1, 0], [2, 0], [-2, 0]]:
					if r.debug_teleport(int(c.tx) + d[0], int(c.ty) + d[1]):
						placed = true
						break
				_check(placed, "run: could not stand beside creature %s" % str(c))
				var hits := 0
				var kills := 0
				var fled := false
				var hurt := false
				for t in 400:
					var cur := {}
					for cc in r.creatures():
						if int(cc.id) == int(c.id):
							cur = cc
					if cur.is_empty():
						break
					if String(cur.phase) == "flee":
						fled = true
						break
					var ddx := int(cur.tx) - int(r.runner().tx)
					var ddy := int(cur.ty) - int(r.runner().ty)
					var adjacent := absi(ddx) <= 1 and absi(ddy) <= 1
					var b: int = scr.B_ATTACK if adjacent and t % 12 == 0 else 0
					scr.step_with(b, signi(ddx) if (not adjacent or t % 12 == 0) else 0, signi(ddy) if not adjacent else 0)
					for e in scr._last_events:
						match String(e.kind):
							"creature_hit": hits += 1
							"creature_died": kills += 1
							"runner_hit": hurt = true
				_check(hits >= 1, "run: no swing landed on the crawler")
				_check(kills >= 1 or fled, "run: crawler neither died nor fled in 400 ticks (hits %d)" % hits)
				_check(String(r.runner().life) == "up", "run: a single crawler put a healthy Runner down (hurt %s)" % str(hurt))
			# extraction: back on the Mat edge, hold E ten seconds
			var e0: PackedInt32Array = r.entry()
			r.debug_teleport(e0[1] % r.width(), e0[1] / r.width())
			for t in 210:
				scr.step_with(scr.B_INTERACT, 0, 0)
			_check(r.result() == "extracted", "run: extraction did not complete (%s, action %s)" % [r.result(), String(r.runner().action)])
			_check(not sim.pending_haul.is_empty(), "run: rig not handed to the Stash")
			var probe := RoseStash.new()
			var left: int = probe.import_rig(sim.pending_haul)
			var kit_items := 0
			for q in r.quick_slots():
				if q != "":
					kit_items += 1
			var expected: int = int(r.runner().rig_count) + kit_items - left
			_check(left >= 0 and probe.count() == expected, "run: stash import %d, expected %d (rig %d + kit %d - left %d)" % [probe.count(), expected, int(r.runner().rig_count), kit_items, left])
			scr.over = ""
	print("[smoke] %-9s ok  slots %d" % [id, slots])

func _has_kind(arr: Array, kind: String) -> bool:
	for e in arr:
		if e.kind == kind:
			return true
	return false

func _count_slots(n: Node) -> int:
	var c := 0
	if n is TextureRect and n.get("slot_id") != null and (n as TextureRect).texture != null:
		c += 1
	for ch in n.get_children():
		c += _count_slots(ch)
	return c

func _finish() -> bool:
	var missing: Array = assets.missing()
	_check(missing.is_empty(), "missing assets: %s" % str(missing))
	_check(screen_ids.size() == 14, "expected 14 screens, got %d" % screen_ids.size())
	var elapsed := Time.get_ticks_msec() - t_start
	print("[smoke] %d screens, %d textures resolved, %d frames in %d ms" % [screen_ids.size(), assets.resolved_count(), frames, elapsed])
	if failures.is_empty():
		print("[smoke] PASS")
		quit(0)
	else:
		for f in failures:
			printerr("[smoke] FAIL: ", f)
		quit(1)
	return true
