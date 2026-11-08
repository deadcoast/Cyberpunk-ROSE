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
