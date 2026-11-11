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
	# the Market first: the colony's shots send a Sweep down grate 5, whose bleach cuts the cord to its market
	screen_ids.erase("market")
	screen_ids.push_front("market")
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
		# a Stockpile Rack beside the Heart, paid with four cheap Parts and built over the shifts; its stockpile at
		# priority 4 draws the Organics and Parts out of the grid, and they ride the Mat to its cells
		for k in 10:
			st.add(RoseItems.new().find("prt_wire"), 40, false)
		var rack := -1
		for r in range(3, 14):
			if rack >= 0:
				break
			for dx in range(-r, r + 1):
				rack = int(rs.place_turned("rack", int(heart.x) + 1 + dx, int(heart.y) + 1 + r, false))
				if rack < 0:
					rack = int(rs.place_turned("rack", int(heart.x) + 1 + dx, int(heart.y) + 1 - r, false))
				if rack >= 0:
					break
		for k in 3:
			sim.end_shift()
			sim.day += 1
			var info: Dictionary = rs.structure_info(rack, 0)
			if bool(info.get("built", false)):
				break
		var rinfo: Dictionary = rs.structure_info(rack, 0)
		for ss in rs.structures():
			if int(ss.id) == rack and bool(rinfo.get("built", false)):
				var cells := PackedInt32Array()
				for yy in range(int(ss.y) - 1, int(ss.y) + 2):
					for xx in range(int(ss.x) - 2, int(ss.x) + 4):
						cells.append(yy * rs.width() + xx)
				if String(rs.paint_stockpile(rack, cells)) == "":
					var z := int(rs.zone_at(int(ss.x) - 1, int(ss.y)))
					for zz in rs.zones():
						if int(zz.anchor) == rack:
							z = int(zz.id)
					rs.set_zone_priority(z, 4)
					rs.set_zone_filter(z, PackedStringArray(["organic", "part"]), 0, 0)
		sim.end_shift()
		for t in 80:
			rs.step()
		scr.set_view(0)
		var focus := heart
		for ss in rs.structures():
			if int(ss.id) == rack:
				focus = ss
		scr.cam.position = Vector2(int(focus.x) * 32 - 96, int(focus.y) * 32 + 16)
		scr.drop_tool()
		# the Rack selected (its stockpile's card: what it holds, its priority and filter), the camera on it with the
		# card to its left; a cut marked and a line of salt by the Heart
		var sel := rack if bool(rs.structure_info(rack, 0).get("built", false)) else int(heart.id)
		scr.selected = {"kind": "structure", "id": sel}
		scr.card_sig = ""
		scr._draw_card()
		var hx := int(heart.x) + 1
		var hy := int(heart.y) + 1
		for dx in range(4, 12):
			if String(rs.tile_info(hx + dx, hy - 2).mat) == "cord":
				rs.mark_cut(hx + dx, hy - 2)
				break
		st.add(RoseItems.new().find("prt_salt"), 30, false)
		for dy in 3:
			if rs.lay_salt(hx - 6, hy + 2 + dy) == "":
				scr.world.paint_salt(hx - 6, hy + 2 + dy)
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
	if frames_on_screen == 6 and screen_i >= 0 and screen_ids[screen_i] == "market":
		# the Market under load (Full Design v3.16): the Nomads' tab with its first offer taken, a stock entry picked
		# and ordered, a curio riding to their stall to sell
		var scr: Node = screens.current
		var st: RoseStash = sim.ensure_stash()
		for it in st.items():
			if String(it.category) == "curio" and not bool(it.pinned):
				st.queue_sell(int(it.handle), 0, sim.day)
				break
		var offers: Array = scr.m.get("offers", [])
		if offers.size() > 0:
			scr.picked_offer = int(offers[0].id)
			scr._accept()
		for e in scr.m.get("stock", []):
			if bool(e.open) and String(e.item) != "":
				scr.picked_entry = int(e.entry)
				scr._order()
				break
	if frames_on_screen == 13 and screen_i >= 0 and screen_ids[screen_i] == "market":
		# a faction whose street the Mat does not reach: what it would trade, the verbs greyed and why
		var scr: Node = screens.current
		for t in scr.m.get("factions", []):
			if not bool(t.reached):
				scr._tab(int(t.index))
				break
	if frames_on_screen == 16 and screen_i >= 0 and screen_ids[screen_i] == "market":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/market_far.png")
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
		# Pressure (Full Design v3.14): a Sweep sent down the first reached grate and walked a minute of city time —
		# the lamp carriers' white light, the sprayers' bleach, the bodies on the map and the gauges over it
		var scr: Node = screens.current
		var rs: RoseSim = sim.ensure()
		scr.set_view(0)
		var zone := 0
		for z in rs.attention():
			if bool(z.reached):
				zone = int(z.number) - 1
				break
		rs.debug_send("sweep", zone)
		scr.selected = {}
		scr.card.visible = false
		# 50 s: down the shaft and sweeping its source
		for t in 100:
			rs.step()
		var bodies: Array = rs.threat_bodies()
		if not bodies.is_empty():
			scr.cam.position = Vector2(int(bodies[0].x) * 32 + 16, int(bodies[0].y) * 32 - 32)
	if frames_on_screen == 19 and screen_i >= 0 and screen_ids[screen_i] == "colony":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/colony_sweep.png")
		# the next Flood shift with water in it: the shift ends until one rises, the camera on the first wet tile
		var rs: RoseSim = sim.ensure()
		for k in 6:
			sim.end_shift()
			sim.day += 1
			if not rs.flooded_cells().is_empty():
				break
		var wet: PackedInt32Array = rs.flooded_cells()
		if not wet.is_empty():
			var i: int = wet[wet.size() / 2]
			screens.current.cam.position = Vector2((i % rs.width()) * 32 + 16, (i / rs.width()) * 32 + 16)
	if frames_on_screen == 22 and screen_i >= 0 and screen_ids[screen_i] == "colony":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/colony_flood.png")
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
		# raiders and a Flood (Full Design v3.14): a gang down a few tiles off and the water rising three, a minute
		# and a half of run time
		var scr: Node = screens.current
		scr.route.debug_flood(3)
		scr.loot_panel.visible = false
		for t in 1300:
			if scr.over != "":
				break
			scr.step_with(0, 0, 0)
		# then a gang down within the Runner's sight, two seconds on
		var r: Dictionary = scr.route.runner()
		var down := false
		for rad in range(3, 8):
			for dx in [rad, -rad]:
				for dy in range(-2, 7):
					if not down and scr.route.debug_raid(int(r.tx) + dx, int(r.ty) + dy, 2, 1):
						down = true
		print("[shots] raiders down: ", down)
		for t in 40:
			if scr.over != "":
				break
			scr.step_with(0, 0, 0)
	if frames_on_screen == 19 and screen_i >= 0 and screen_ids[screen_i] == "run":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/run_pressure.png")
	if frames_on_screen >= (24 if screen_i >= 0 and screen_ids[screen_i] in ["colony", "run"] else 18):
		screen_i += 1
		frames_on_screen = 0
		if screen_i < screen_ids.size():
			screens.show_screen(screen_ids[screen_i])
		else:
			print("[shots] wrote %d screens to %s" % [screen_ids.size() + 6, out_dir])
			quit(0)
			return true
	return false
