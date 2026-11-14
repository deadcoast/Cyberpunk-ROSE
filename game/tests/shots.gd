extends SceneTree
## Screenshot every screen at native 640x360 into assets/shots/. Needs a display (Xvfb works):
##   xvfb-run -s "-screen 0 1920x1080x24" godot --path game --rendering-driver opengl3 --script res://tests/shots.gd
## What each shot stages is held to happening (_expect): a shot whose staging was refused shows a screen that is not
## the one its name says, so the script exits 1 naming the stage. A shift the staging waits on is a played shift.

var screen_ids: Array = []
var screen_i := -1
var frames_on_screen := 0
var assets: Node
var screens: Node
var sim: Node
var out_dir := ""
## the stages held to happening, and the ones that did not
var stages := 0
var failed := PackedStringArray()

func _expect(stage: String, ok: bool, why: String = "") -> bool:
	stages += 1
	if not ok:
		failed.append(stage)
		print("[shots] FAIL %s%s" % [stage, (": " + why) if why != "" else ""])
	return ok

func _autoload(name: String, path: String) -> Node:
	var n := root.get_node_or_null(name)
	if n == null:
		n = (load(path) as GDScript).new()
		n.name = name
		root.add_child(n)
	return n

func _initialize() -> void:
	# the engine's own random stream (New Run's offered seed) fixed, so the frames repeat byte for byte
	seed(11)
	assets = _autoload("Assets", "res://autoload/Assets.gd")
	screens = _autoload("Screens", "res://autoload/Screens.gd")
	sim = _autoload("Sim", "res://autoload/Sim.gd")
	sim.today_fixed = 20261113
	# the tests' own profile, fresh each time: nothing lands in the player's
	sim.profile_path = "user://profile_test"
	DirAccess.remove_absolute(ProjectSettings.globalize_path(sim.profile_path))
	root.add_child((load("res://Main.tscn") as PackedScene).instantiate())
	sim.ticks_per_frame = 2
	# the campaign's panel sets play when the script shows them, not on their own
	sim.panel_hold = true
	screen_ids = screens.ids()
	# the Market first: the colony's shots send a Sweep down grate 5, whose bleach cuts the cord to its market
	screen_ids.erase("market")
	screen_ids.push_front("market")
	out_dir = ProjectSettings.globalize_path("res://../assets/shots")
	DirAccess.make_dir_recursive_absolute(out_dir)

func _process(_delta: float) -> bool:
	frames_on_screen += 1
	if frames_on_screen == 6 and screen_i >= 0 and screen_ids[screen_i] == "colony":
		# the Street view on the Heart with the founding structures, a pod ghost placed and a shift ended so the
		# Heart's district has formed and the ghost is paid for; the cursor hovers a tile
		var scr: Node = screens.current
		var rs: RoseSim = sim.ensure()
		var heart: Dictionary = rs.structures()[0]
		# the genome as a Garden-leaning feed plan would leave it: the shift's end grows its organs (Full Design
		# v3.18, Organs) for the Strain Lab and the Codex
		rs.debug_gene(4, 0.7)
		rs.debug_gene(2, 0.7)
		rs.debug_gene(1, 0.7)
		rs.debug_gene(6, 0.56)
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
		_expect("a pod ghost placed by the Heart", placed >= 0)
		# cargo (Full Design v3.11): the curios and tech queued to sell, a quota on the Heart's Pit that pulls the
		# Chrome; at the shift end they leave the Heart as packets
		var st: RoseStash = sim.ensure_stash()
		for it in st.items():
			if String(it.category) in ["curio", "tech", "organic"] and not bool(it.pinned):
				st.queue_sell(int(it.handle), 0, sim.day)
		_expect("a quota on the Heart's Pit", rs.set_quota(int(heart.id), 2.0, 1))
		# a Stockpile Rack beside the Heart, paid with four cheap Parts and built over the shifts; its stockpile at
		# priority 4 draws the Organics and Parts out of the grid, and they ride the Mat to its cells
		var wires := 0
		for k in 10:
			wires += 1 if st.add(RoseItems.new().find("prt_wire"), 40, false) >= 0 else 0
		_expect("the Rack's Parts into the Stash", wires == 10, "%d of 10 placed" % wires)
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
		_expect("a Stockpile Rack placed beside the Heart", rack >= 0)
		for k in 3:
			sim.play_shift()
			var info: Dictionary = rs.structure_info(rack, 0)
			if bool(info.get("built", false)):
				break
		var rinfo: Dictionary = rs.structure_info(rack, 0)
		_expect("the Rack built", bool(rinfo.get("built", false)), "not after 3 shifts")
		var painted := false
		for ss in rs.structures():
			if int(ss.id) == rack and bool(rinfo.get("built", false)):
				var cells := PackedInt32Array()
				for yy in range(int(ss.y) - 1, int(ss.y) + 2):
					for xx in range(int(ss.x) - 2, int(ss.x) + 4):
						cells.append(yy * rs.width() + xx)
				var why: String = rs.paint_stockpile(rack, cells)
				painted = _expect("the Rack's stockpile painted", why == "", why)
				if painted:
					var z := int(rs.zone_at(int(ss.x) - 1, int(ss.y)))
					for zz in rs.zones():
						if int(zz.anchor) == rack:
							z = int(zz.id)
					rs.set_zone_priority(z, 4)
					rs.set_zone_filter(z, PackedStringArray(["organic", "part"]), 0, 0)
		_expect("the stockpile's cells", painted)
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
		var cut := "no cord in reach"
		for dx in range(4, 12):
			if String(rs.tile_info(hx + dx, hy - 2).mat) == "cord":
				cut = rs.mark_cut(hx + dx, hy - 2)
				break
		_expect("a cut marked on the cord", cut == "", cut)
		_expect("Salt into the Stash", st.add(RoseItems.new().find("prt_salt"), 30, false) >= 0, "the grid is full")
		var salted := 0
		for dy in 3:
			var why: String = rs.lay_salt(hx - 6, hy + 2 + dy)
			if why == "":
				scr.world.paint_salt(hx - 6, hy + 2 + dy)
				salted += 1
		_expect("a line of salt by the Heart", salted > 0)
	if frames_on_screen == 6 and screen_i >= 0 and screen_ids[screen_i] == "charter":
		# the Charter under load: Rationing signed with the Authority the Heart's district paid
		var why: String = sim.ensure().sign_law("rationing")
		_expect("Rationing signed", why == "", why)
		screens.current._redraw()
	if frames_on_screen == 6 and screen_i >= 0 and screen_ids[screen_i] == "stash":
		# the Stash with a card open and a sale and a feed queued
		var scr: Node = screens.current
		var st: RoseStash = scr.stash
		# what the shot queues, as a haul brings it (the colony's shots sold the founding's curios): a curio to sell, a
		# Chrome to feed
		_expect("a curio and a Chrome into the Stash", st.add(RoseItems.new().find("cur_street_sign"), 70, false) >= 0 and st.add(RoseItems.new().find("chr_optic_slag"), 70, false) >= 0, "the grid is full")
		var first_chrome := -1
		var sold := false
		var fed := false
		for it in st.items():
			if String(it.category) == "curio":
				sold = st.queue_sell(int(it.handle), 0, sim.day)
				break
		for it in st.items():
			if String(it.category) == "chrome" and first_chrome < 0:
				first_chrome = int(it.handle)
			elif String(it.category) == "chrome":
				fed = st.queue_feed(int(it.handle), 0, sim.day)
				break
		_expect("a curio queued to sell on the Stash", sold)
		_expect("a Chrome queued to feed on the Stash", fed)
		_expect("a Chrome's card open on the Stash", first_chrome >= 0)
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
		_expect("the Nomads have an offer", offers.size() > 0)
		if offers.size() > 0:
			scr.picked_offer = int(offers[0].id)
			scr._accept()
			_expect("the Nomads' first offer taken", scr.picked_offer == -1, scr.status)
		var ordered := false
		for e in scr.m.get("stock", []):
			if bool(e.open) and String(e.item) != "":
				scr.picked_entry = int(e.entry)
				scr._order()
				ordered = String(scr.status).begins_with("ordered")
				break
		_expect("a stock entry ordered", ordered, scr.status)
	if frames_on_screen == 13 and screen_i >= 0 and screen_ids[screen_i] == "market":
		# a faction whose street the Mat does not reach: what it would trade, the verbs greyed and why
		var scr: Node = screens.current
		var far := false
		for t in scr.m.get("factions", []):
			if not bool(t.reached):
				scr._tab(int(t.index))
				far = true
				break
		_expect("a faction the Mat does not reach", far)
	if frames_on_screen == 16 and screen_i >= 0 and screen_ids[screen_i] == "market":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/market_far.png")
	if frames_on_screen == 6 and screen_i >= 0 and screen_ids[screen_i] == "bench":
		# the Bench with a Stash item slotted into the scrap sword and a job on the queue
		var st: RoseStash = sim.ensure_stash()
		var blade: int = st.add(RoseItems.new().find("prt_blade_stock"), 90, false)
		st.add(RoseItems.new().find("prt_wire"), 70, false)
		var b2: int = st.add(RoseItems.new().find("chr_toe_claws"), 60, false)
		# two housings, as a haul brings them (the colony's builders spent the founding's Pipes): one for the queued
		# shiv, one for the sword on the Bench
		for k in 2:
			st.add(RoseItems.new().find("prt_pipe"), 70, false)
		var housing := -1
		for h in st.candidates("housing"):
			housing = int(h)
			break
		_expect("the scrap sword's parts into the Stash", blade >= 0 and b2 >= 0 and housing >= 0)
		_expect("a shiv on the Bench's queue", st.craft("shiv", PackedInt32Array([b2, housing]), "Sable", sim.day))
		for it in st.items():
			if int(it.handle) == blade:
				sim.bench_slot = it
		screens.show_screen("bench")
		var scr: Node = screens.current
		for h in st.candidates("housing"):
			if not scr.filled.has(int(h)):
				scr._fill(int(h))
				break
	if frames_on_screen == 14 and screen_i >= 0 and screen_ids[screen_i] == "bench":
		# the Research tab: a Blueprint I studied (Chrome tier 2's), the Sphincter housing picked on the tree
		var st: RoseStash = sim.ensure_stash()
		_expect("the blueprints into the Stash", st.add(RoseItems.new().find("tec_blueprint_chrome1"), 80, true) >= 0 and st.add(RoseItems.new().find("tec_blueprint_myc2"), 80, true) >= 0, "the grid is full")
		var why: String = sim.ensure().research_pick("study", "tec_blueprint_chrome1")
		_expect("STUDY Blueprint I", why == "", why)
		var scr: Node = screens.current
		scr.picked_node = "c2_gate"
		scr._set_tab("research")
	if frames_on_screen == 18 and screen_i >= 0 and screen_ids[screen_i] == "bench":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/bench_research.png")
		screens.current._set_tab("craft")
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
		_expect("the Sweep's bodies on the map", not bodies.is_empty())
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
		_expect("a Flood shift with water in it", not wet.is_empty(), "dry after 6 shifts")
		if not wet.is_empty():
			var i: int = wet[wet.size() / 2]
			screens.current.cam.position = Vector2((i % rs.width()) * 32 + 16, (i / rs.width()) * 32 + 16)
	if frames_on_screen == 22 and screen_i >= 0 and screen_ids[screen_i] == "colony":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/colony_flood.png")
	if frames_on_screen == 13 and screen_i >= 0 and screen_ids[screen_i] == "run":
		# a second Run shot: standing at the first crate or locker with its manifest unrolled
		var scr: Node = screens.current
		var stood := false
		for src in scr.route.sources():
			if src.state == "closed" and (src.kind == "crate" or src.kind == "locker"):
				stood = scr.route.debug_teleport(int(src.x), int(src.y))
				if stood:
					scr.step_with(0, 0, 0)
					scr.step_with(scr.B_INTERACT, 0, 0)
					scr.step_with(0, 0, 0)
					scr.loot_shown = scr.loot_items.size()
					scr._redraw_found()
				break
		_expect("the Runner at a closed container", stood)
		_expect("its manifest unrolled", stood and scr.loot_items.size() > 0)
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
		_expect("a gang down in the Runner's sight", down)
		for t in 40:
			if scr.over != "":
				break
			scr.step_with(0, 0, 0)
	if frames_on_screen == 19 and screen_i >= 0 and screen_ids[screen_i] == "run":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/run_pressure.png")
	if frames_on_screen >= (24 if screen_i >= 0 and screen_ids[screen_i] in ["colony", "run", "bench"] else 18):
		screen_i += 1
		frames_on_screen = 0
		if screen_i < screen_ids.size():
			screens.show_screen(screen_ids[screen_i])
		else:
			print("[shots] wrote %d screens to %s" % [screen_ids.size() + 6, out_dir])
			if failed.is_empty():
				print("[shots] PASS %d stages" % stages)
				quit(0)
			else:
				print("[shots] FAIL %d of %d stages: %s" % [failed.size(), stages, ", ".join(failed)])
				quit(1)
			return true
	return false
