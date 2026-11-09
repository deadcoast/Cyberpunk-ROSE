extends SceneTree
## Screenshot every screen at native 640x360 into docs/shots/. Needs a display (Xvfb works):
##   xvfb-run -s "-screen 0 1920x1080x24" godot --path game --rendering-driver opengl3 --script res://tests/shots.gd

var screen_ids: Array = []
var screen_i := -1
var frames_on_screen := 0
var assets: Node
var screens: Node
var sim: Node
var out_dir := ""

func _autoload(name: String, path: String) -> Node:
	var n := root.get_node_or_null(name)
	if n == null:
		n = (load(path) as GDScript).new()
		n.name = name
		root.add_child(n)
	return n

func _initialize() -> void:
	assets = _autoload("Assets", "res://autoload/Assets.gd")
	screens = _autoload("Screens", "res://autoload/Screens.gd")
	sim = _autoload("Sim", "res://autoload/Sim.gd")
	root.add_child((load("res://Main.tscn") as PackedScene).instantiate())
	sim.ticks_per_frame = 2
	screen_ids = screens.ids()
	out_dir = ProjectSettings.globalize_path("res://../docs/shots")
	DirAccess.make_dir_recursive_absolute(out_dir)

func _process(_delta: float) -> bool:
	frames_on_screen += 1
	if frames_on_screen == 6 and screen_i >= 0 and screen_ids[screen_i] == "colony":
		# the Street view on the Heart with the founding structures, a pod ghost placed and a shift ended so the
		# Heart's district has formed and the ghost is paid for; the cursor hovers a tile
		var scr: Node = screens.current
		var rs: RoseSim = sim.ensure()
		var heart: Dictionary = rs.structures()[0]
		sim.build_tool = "pod"
		scr.tool = "build"
		var placed := -1
		for r in range(2, 12):
			if placed >= 0:
				break
			for dy in range(-r, r + 1):
				if placed >= 0:
					break
				for dx in range(-r, r + 1):
					if absi(dx) != r and absi(dy) != r:
						continue
					placed = scr.place_ghost(int(heart.x) + 1 + dx, int(heart.y) + 1 + dy)
					if placed >= 0:
						break
		# cargo (Full Design v3.11): the curios and tech queued to sell, a quota on the Heart's Pit that pulls the
		# Chrome; at the shift end they leave the Heart as packets
		var st: RoseStash = sim.ensure_stash()
		for it in st.items():
			if String(it.category) in ["curio", "tech", "organic"] and not bool(it.pinned):
				st.queue_sell(int(it.handle), 0, sim.day)
		rs.set_quota(int(heart.id), 2.0, 1)
		sim.end_shift()
		scr.set_view(0)
		scr.cam.position = Vector2((int(heart.x) + 1) * 32 + 16, (int(heart.y) + 1) * 32 + 16)
		scr.tool = "select"
		scr.open_quota(int(heart.id))
	if frames_on_screen == 6 and screen_i >= 0 and screen_ids[screen_i] == "charter":
		# the Charter under load: Rationing signed with the Authority the Heart's district paid
		sim.ensure().sign_law("rationing")
		screens.current._redraw()
	if frames_on_screen == 6 and screen_i >= 0 and screen_ids[screen_i] == "stash":
		# the Stash with a card open and a sale and a feed queued
		var scr: Node = screens.current
		var st: RoseStash = scr.stash
		var first_chrome := -1
		for it in st.items():
			if String(it.category) == "curio":
				st.queue_sell(int(it.handle), 0, sim.day)
				break
		for it in st.items():
			if String(it.category) == "chrome" and first_chrome < 0:
				first_chrome = int(it.handle)
			elif String(it.category) == "chrome":
				st.queue_feed(int(it.handle), 0, sim.day)
				break
		scr._select(first_chrome)
	if frames_on_screen == 6 and screen_i >= 0 and screen_ids[screen_i] == "bench":
		# the Bench with a Stash item slotted into the scrap sword and a job on the queue
		var st: RoseStash = sim.ensure_stash()
		var blade: int = st.add(RoseItems.new().find("prt_blade_stock"), 90, false)
		st.add(RoseItems.new().find("prt_wire"), 70, false)
		var b2: int = st.add(RoseItems.new().find("chr_toe_claws"), 60, false)
		var housing := -1
		for h in st.candidates("housing"):
			housing = int(h)
			break
		st.craft("shiv", PackedInt32Array([b2, housing]), "Sable", sim.day)
		for it in st.items():
			if int(it.handle) == blade:
				sim.bench_slot = it
		screens.show_screen("bench")
		var scr: Node = screens.current
		for h in st.candidates("housing"):
			if not scr.filled.has(int(h)):
				scr._fill(int(h))
				break
	if frames_on_screen == 12 and screen_i >= 0:
		var id: String = screen_ids[screen_i]
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/" + id + ".png")
		if id == "colony":
			screens.current.set_view(1)
	if frames_on_screen == 14 and screen_i >= 0 and screen_ids[screen_i] == "colony":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/colony_district.png")
		screens.current.set_view(2)
	if frames_on_screen == 16 and screen_i >= 0 and screen_ids[screen_i] == "colony":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/colony_stack.png")
		sim.build_tool = ""
	if frames_on_screen == 13 and screen_i >= 0 and screen_ids[screen_i] == "run":
		# a second Run shot: standing at the first crate or locker with its manifest unrolled
		var scr: Node = screens.current
		for src in scr.route.sources():
			if src.state == "closed" and (src.kind == "crate" or src.kind == "locker"):
				if scr.route.debug_teleport(int(src.x), int(src.y)):
					scr.step_with(0, 0, 0)
					scr.step_with(scr.B_INTERACT, 0, 0)
					scr.step_with(0, 0, 0)
					scr.loot_shown = scr.loot_items.size()
					scr._redraw_found()
				break
	if frames_on_screen == 16 and screen_i >= 0 and screen_ids[screen_i] == "run":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/run_loot.png")
	if frames_on_screen >= 18:
		screen_i += 1
		frames_on_screen = 0
		if screen_i < screen_ids.size():
			screens.show_screen(screen_ids[screen_i])
		else:
			print("[shots] wrote %d screens to %s" % [screen_ids.size() + 3, out_dir])
			quit(0)
			return true
	return false
