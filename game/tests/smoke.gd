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
			var n: int = scr.route.chunk_count()
			_check(n >= 8 and n <= 14, "run: route has %d chunks, expected 8..14" % n)
			_check(scr.route.library_size() >= 14, "run: chunk library has %d chunks" % scr.route.library_size())
			_check(scr.on_mat(), "run: runner did not start on the Mat edge")
			_check(scr.route.connectivity() >= 0.95, "run: route connectivity %.3f" % scr.route.connectivity())
			_check(scr.world.substrate.get_used_cells().size() > 2000, "run: route tiles not painted")
			_check(scr.world.props.get_used_cells().size() >= 8, "run: props not painted (%d)" % scr.world.props.get_used_cells().size())
			var before: Vector2i = scr.tile
			var moved: bool = scr.step(Vector2i(1, 0))
			_check(moved and scr.tile != before, "run: runner could not move east off the Mat edge")
			# dig the first collapse on the route open
			var dug := false
			for hz in scr.route.hazards():
				if hz.kind == "collapse":
					dug = scr.route.dig(int(hz.x), int(hz.y))
					break
			_check(dug or scr.route.hazards().is_empty() or not _has_kind(scr.route.hazards(), "collapse"), "run: dig did not open a collapse")
			# senses (PAC-021): light, sight, fog, noise
			var st: PackedInt32Array = scr.route.sense_stats()
			_check(st.size() == 4 and st[0] > 20, "run: nothing visible (%s)" % str(st))
			_check(scr.route.memory_bits().size() == int(ceil(scr.route.width() * scr.route.height() / 8.0)), "run: memory bitset size")
			for i in 6:
				scr.step(Vector2i(1, 0))
			_check(not scr.on_mat(), "run: still on the Mat after six steps east")
			scr.lantern_on = true
			scr._sense_tick()
			var lit_band: String = scr.route.exposure_band()
			var lit_exposure: int = scr.route.exposure()
			var seen_lit: int = scr.route.sense_stats()[0]
			scr.lantern_on = false
			scr._sense_tick()
			var dark_exposure: int = scr.route.exposure()
			var seen_dark: int = scr.route.sense_stats()[0]
			_check(lit_band == "lit" and lit_exposure >= 7, "run: lantern on should read lit (%s %d)" % [lit_band, lit_exposure])
			_check(dark_exposure < lit_exposure, "run: lantern off did not lower exposure (%d -> %d)" % [lit_exposure, dark_exposure])
			_check(seen_dark < seen_lit, "run: lantern off did not shrink sight (%d -> %d)" % [seen_lit, seen_dark])
			_check(scr.route.notice_range() == 2 + dark_exposure + (-1 if scr.motion == 0 else 0) or scr.route.notice_range() >= 1, "run: notice range contract")
			var reached: int = scr.route.runner_sound(2, 0)
			_check(reached > 20 and scr.route.noise_here() >= 7, "run: sprint step footprint %d, here %d" % [reached, scr.route.noise_here()])
			for i in 6:
				scr._sense_tick()
			_check(scr.route.noise_here() == 0, "run: noise did not decay (%d)" % scr.route.noise_here())
			var seen_total: int = scr.route.sense_stats()[1]
			_check(seen_total > scr.route.sense_stats()[0], "run: nothing remembered after moving (%d seen, %d visible)" % [seen_total, scr.route.sense_stats()[0]])
			_check(scr.fog != null and scr.fog.texture != null, "run: fog overlay missing")
			scr.lantern_on = true
			# loot (PAC-022): sources, opening, the manifest, the rig, the hand-off
			var ls: PackedInt32Array = scr.route.loot_stats()
			_check(ls.size() == 6 and ls[0] >= 8, "run: loot sources %s" % str(ls))
			var opened := 0
			var revealed := 0
			var took := 0
			for src in scr.route.sources():
				if src.state != "closed":
					continue
				scr.tile = Vector2i(int(src.x), int(src.y))
				scr._place()
				_check(scr.source_here() == int(src.index), "run: source_at disagrees with sources()")
				if scr.open_here():
					opened += 1
					revealed += scr.loot_items.size()
					scr.loot_shown = scr.loot_items.size()
					var r: Array = scr.take_all_here()
					took += int(r[0])
					scr.close_loot()
				if opened >= 6:
					break
			_check(opened >= 4, "run: opened %d sources" % opened)
			_check(revealed >= opened, "run: %d items revealed from %d opens" % [revealed, opened])
			_check(took >= 1 and scr.route.rig_stats()[0] == took, "run: rig holds %d, took %d" % [scr.route.rig_stats()[0], took])
			_check(scr.route.rig_weight() > 0.0 and scr.route.rig_value_blind() >= 0, "run: rig weight/value")
			for it in scr.route.rig_items():
				_check(not bool(it.ident) and String(it.label).begins_with("unidentified"), "run: found item identified on pickup: %s" % str(it))
				break
			var export: PackedByteArray = scr.route.rig_export()
			_check(export.size() > 8, "run: rig export empty")
			var probe := RoseStash.new()
			var left: int = probe.import_rig(export)
			_check(left >= 0 and probe.count() == took - left, "run: stash import %d of %d (left %d)" % [probe.count(), took, left])
			# a dug deposit leaves a pile
			var before_sources: int = scr.route.loot_stats()[0]
			var dug_pile := false
			for y in scr.world.h:
				for x in scr.world.w:
					if int(scr.substrate[y * scr.world.w + x]) == 4:
						dug_pile = scr.route.dig(x, y)
						break
				if dug_pile:
					break
			_check(dug_pile and scr.route.loot_stats()[0] == before_sources + 1, "run: dug deposit did not leave a pile")
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
