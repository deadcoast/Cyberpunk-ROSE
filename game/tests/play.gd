extends SceneTree
## Scripted play under a display, for review: the Route Board packs a rig and departs, then the Run is driven frame
## by frame through the input word; frames at the moments that matter land in docs/shots/play_*.png.
##   xvfb-run -s "-screen 0 1920x1080x24" godot --path game --rendering-driver opengl3 --script res://tests/play.gd

var assets: Node
var screens: Node
var sim: Node
var scr: Node
var out_dir := ""
var t := 0
var shots := 0
var phase := 0
var crawler_id := -1
var take_t := 0

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
	# the tests' own profile, fresh each time: nothing lands in the player's
	sim.profile_path = "user://profile_test"
	DirAccess.remove_absolute(ProjectSettings.globalize_path(sim.profile_path))
	root.add_child((load("res://Main.tscn") as PackedScene).instantiate())
	# the campaign's panel sets play when the script shows them, not on their own
	sim.panel_hold = true
	out_dir = ProjectSettings.globalize_path("res://../docs/shots")
	DirAccess.make_dir_recursive_absolute(out_dir)

func _shot(name: String) -> void:
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png(out_dir + "/play_" + name + ".png")
	shots += 1

var t0 := 0
var runner_id := -1
var mend_id := -1
var slab := {}
var shrine_id := -1

func _hp(id: int) -> int:
	for st in sim.ensure().structures():
		if int(st.id) == id:
			return int(st.hp)
	return -1

func _process(_delta: float) -> bool:
	t += 1
	if t == 1:
		# the Route Board first: pack a rig from the Stash, pick the S1 card, depart
		screens.show_screen("route")
		scr = screens.current
		return false
	if t == 6:
		for it in sim.ensure_stash().items():
			if String(it.id) in ["ger_pipe", "ger_lantern", "prt_salt", "org_antiseptic", "ger_pry_bar"]:
				scr._pack(int(it.handle))
		scr._select(0)
		return false
	if t == 12:
		_shot("00_board")
		runner_id = scr.runner_id
		scr._depart()
		scr = screens.current
		t0 = t
		return false
	if t < 12:
		return false
	match phase:
		0:
			# stand on the Mat edge, lantern on
			scr.scripted = [0, 0, 0]
			if t == t0 + 10:
				_shot("01_start")
				phase = 1
		1:
			# walk east with the lantern for three seconds
			scr.scripted = [0, 1, 0]
			if t == t0 + 10 + 180:
				_shot("02_walked")
				phase = 2
		2:
			# sprint on: louder, faster
			scr.scripted = [scr.B_SPRINT, 1, 0]
			if t == t0 + 10 + 180 + 60:
				_shot("03_sprint")
				phase = 3
		3:
			# lantern off and dim veins: the dark
			if t == t0 + 10 + 180 + 61:
				scr.lantern_pressed = true
			scr.scripted = [scr.B_DIM, 0, 0]
			if t == t0 + 10 + 180 + 120:
				_shot("04_dark")
				phase = 4
		4:
			# go stand by a crawler and fight it
			scr.lantern_pressed = true
			scr.scripted = [0, 0, 0]
			var cs: Array = scr.route.creatures()
			for c in cs:
				if String(c.kind) == "crawler":
					crawler_id = int(c.id)
					for d in [[-2, 0], [2, 0], [-1, 0], [1, 0], [0, -1], [0, 1], [-2, -1], [2, -1]]:
						if scr.route.debug_teleport(int(c.tx) + d[0], int(c.ty) + d[1]):
							break
					break
			phase = 5
		5:
			var alive := false
			var cx := 0
			var cy := 0
			for c in scr.route.creatures():
				if int(c.id) == crawler_id:
					alive = true
					cx = int(c.tx)
					cy = int(c.ty)
			var rs: Dictionary = scr.route.runner()
			if not alive or String(rs.life) != "up":
				_shot("06_kill")
				phase = 6
			else:
				var ddx := cx - int(rs.tx)
				var ddy := cy - int(rs.ty)
				var adjacent := absi(ddx) <= 1 and absi(ddy) <= 1
				if adjacent:
					scr.scripted = [scr.B_ATTACK if t % 14 == 0 else 0, signi(ddx) if t % 14 == 0 else 0, 0]
				else:
					scr.scripted = [0, signi(ddx), signi(ddy)]
				if shots == 5 and adjacent and t % 14 == 7:
					_shot("05_fight")
				if t % 100 == 0:
					var cinfo := {}
					for c in scr.route.creatures():
						if int(c.id) == crawler_id:
							cinfo = c
					print("[play] t%d runner %d,%d pose %s action %s | crawler %d,%d %s hp %d aw %.2f vis %s" % [t, int(rs.tx), int(rs.ty), String(rs.pose), String(rs.action), cx, cy, String(cinfo.get("phase", "?")), int(cinfo.get("hp", 0)), float(cinfo.get("awareness", 0.0)), str(cinfo.get("visible", false))])
				if t > 1500:
					phase = 6
		6:
			# open a container
			var done := false
			for src in scr.route.sources():
				if src.state == "closed" and (src.kind == "crate" or src.kind == "locker"):
					var ok: bool = scr.route.debug_teleport(int(src.x), int(src.y))
					print("[play] teleport to %s at %d,%d: %s" % [src.kind, int(src.x), int(src.y), str(ok)])
					done = ok
					break
			scr.scripted = [0, 0, 0]
			phase = 7 if done else 8
		7:
			# one idle tick so the press registers as an edge, then press
			scr.scripted = [scr.B_INTERACT, 0, 0]
			phase = 12
		12:
			scr.scripted = [0, 0, 0]
			if scr.loot_panel.visible and scr.loot_shown == scr.loot_items.size():
				_shot("07_loot")
				phase = 8
			elif t > 2600:
				print("[play] loot panel never opened; src_here %d" % int(scr.route.runner().source_here))
				phase = 8
		8:
			scr.scripted = [scr.B_TAKE, 0, 0]
			take_t = t
			phase = 13
		13:
			# let the take land before leaving the source
			scr.scripted = [0, 0, 0]
			if t >= take_t + 12:
				phase = 9
		9:
			# extraction
			var e0: PackedInt32Array = scr.route.entry()
			scr.route.debug_teleport(e0[1] % scr.route.width(), e0[1] / scr.route.width())
			scr.scripted = [scr.B_INTERACT, 0, 0]
			phase = 10
		10:
			scr.scripted = [scr.B_INTERACT, 0, 0]
			if String(scr.route.runner().action) == "extract" and float(scr.route.runner().action_progress) > 0.4:
				_shot("08_extracting")
				phase = 11
		11:
			scr.scripted = [scr.B_INTERACT, 0, 0]
			if scr.over == "extracted":
				_shot("09_extracted")
				# home: the Stash takes the haul and the shift report; sell a curio and feed the Heart
				screens.show_screen("stash")
				scr = screens.current
				phase = 14
		14:
			if t % 10 == 0:
				var st: RoseStash = scr.stash
				var curio := -1
				var chrome := -1
				for it in st.items():
					if String(it.category) == "curio" and curio < 0:
						curio = int(it.handle)
					if String(it.category) == "chrome" and chrome < 0:
						chrome = int(it.handle)
				if curio >= 0:
					st.queue_sell(curio, 0, sim.day)
				# and the rest of what a stall buys and the city does not use, for the Market's orders to come
				for it in st.items():
					if String(it.category) in ["curio", "tech", "contraband"] and int(it.handle) != curio and not bool(it.pinned):
						st.queue_sell(int(it.handle), 0, sim.day)
				scr.filter_cat = 1
				scr._select(chrome)
				phase = 15
		15:
			if t % 10 == 5:
				_shot("10_stash")
				# the Runner's card after the run: a run on the record, needs moved, Scav XP
				screens.show_screen("slimo")
				scr = screens.current
				scr.selected = runner_id
				scr._redraw()
				phase = 16
		16:
			if t % 10 == 5:
				_shot("11_slimo")
				# the Market (Full Design v3.16): Open Grates signed on the Charter, then the Corpo remnants' intel and
				# the Gangs' protection ordered at the Nomads' stall over grate 5
				var why: String = sim.ensure().sign_law("open_grates")
				print("[play] Open Grates: %s" % ("signed" if why == "" else why))
				screens.show_screen("market")
				scr = screens.current
				phase = 17
		17:
			if t % 10 == 5:
				for svc in [["Corpo remnants", "intel"], ["Gangs", "protection"]]:
					for tab in scr.m.get("factions", []):
						if String(tab.name) == svc[0]:
							scr._tab(int(tab.index))
					for e in scr.m.get("stock", []):
						if String(e.service) == svc[1]:
							scr.picked_entry = int(e.entry)
							scr._order()
							print("[play] %s from the %s: %s" % [svc[1], svc[0], scr.status])
				phase = 18
		18:
			if t % 10 == 5:
				_shot("12_market")
				# the shift ends with the ledger short: the orders wait; the sales ride to grate 5 and land while the
				# city's clock runs; the next shift's end pays them and intel runs from the shift after
				sim.end_shift()
				sim.day += 1
				var rs: RoseSim = sim.ensure()
				var steps := 0
				while sim.ensure_stash().eddies() < 70 and steps < 2400:
					rs.step()
					steps += 1
				print("[play] the orders waited a shift; %d ticks later the ledger holds %d e" % [steps, int(sim.ensure_stash().eddies())])
				sim.end_shift()
				sim.day += 1
				screens.show_screen("colony")
				scr = screens.current
				scr.set_view(2)
				phase = 19
		19:
			if t % 10 == 5:
				_shot("13_intel")
				var z: Array = sim.ensure().attention()
				print("[play] after the shift: intel shows %s; ledger %d e" % [str(z.map(func(q): return bool(q.get("exact", false)))), int(sim.ensure_stash().eddies())])
				# mending (Full Design v3.17): the Bench left at 55 of its HP, as a flamer would leave it; its card
				var rs: RoseSim = sim.ensure()
				for st in rs.structures():
					if String(st.def) == "bench" and bool(st.built):
						mend_id = int(st.id)
						rs.debug_hp(mend_id, int(int(st.hp_max) * 0.55))
						rs.refresh_movers(sim.ensure_people(), sim.day)
						scr.set_view(0)
						scr.cam.position = Vector2(int(st.x) * 32 + 16, int(st.y) * 32 + 48)
						scr.selected = {"kind": "structure", "id": mend_id}
						scr.card_sig = ""
						scr._draw_card()
				phase = 20
		20:
			if t % 10 == 5:
				_shot("14_mending")
				var before := _hp(mend_id)
				sim.end_shift()
				sim.day += 1
				scr.card_sig = ""
				scr._draw_card()
				print("[play] the Bench mended from %d to %d HP at the shift's end" % [before, _hp(mend_id)])
				phase = 21
		21:
			if t % 10 == 5:
				_shot("15_mended")
				# research (Full Design v3.18): a Blueprint I in the Stash as a haul brings it; the Bench's Research tab,
				# STUDY as its button does it
				var st: RoseStash = sim.ensure_stash()
				st.add(RoseItems.new().find("tec_blueprint_chrome1"), 80, true)
				screens.show_screen("bench")
				scr = screens.current
				scr._set_tab("research")
				scr.research_msg = sim.ensure().research_pick("study", "tec_blueprint_chrome1")
				scr._replan()
				scr._draw_research()
				print("[play] study: %s" % ("picked" if scr.research_msg == "" else scr.research_msg))
				# the Job grid: the D3s put Research at 4, as a player paints the column (Research is the job index 8)
				for row in sim.ensure().job_grid(sim.ensure_people()):
					if String(row.depth) == "D3":
						sim.ensure_people().set_priority(int(row.id), 8, 4)
				phase = 22
		22:
			if t % 10 == 5:
				_shot("16_study")
				# the shifts run until Chrome tier 2 opens; then the Sphincter housing picked and RESEARCH pressed
				var rs: RoseSim = sim.ensure()
				var shifts := 0
				while int((rs.tech().open as PackedInt32Array)[0]) < 2 and shifts < 20:
					sim.end_shift()
					sim.day += 1
					shifts += 1
				print("[play] Chrome tier 2 open after %d shifts" % shifts)
				# the node's Parts and Chrome, as the Stash holds them
				var st: RoseStash = sim.ensure_stash()
				for k in 6:
					st.add(RoseItems.new().find("prt_pipe"), 70, true)
				for k in 2:
					st.add(RoseItems.new().find("chr_optic_slag"), 70, true)
				scr._set_tab("research")
				scr.picked_node = "c2_gate"
				scr.research_msg = rs.research_pick("node", "c2_gate")
				scr._replan()
				scr._draw_research()
				print("[play] node: %s" % ("picked" if scr.research_msg == "" else scr.research_msg))
				phase = 23
		23:
			if t % 10 == 5:
				_shot("17_node")
				var rs: RoseSim = sim.ensure()
				var shifts := 0
				var done := false
				while not done and shifts < 30:
					sim.end_shift()
					sim.day += 1
					shifts += 1
					for n in rs.tech().nodes:
						if String(n.id) == "c2_gate" and String(n.state) == "researched":
							done = true
				print("[play] the Sphincter housing researched after %d more shifts" % shifts)
				sim.build_tool = "gate"
				screens.show_screen("build")
				scr = screens.current
				phase = 24
		24:
			if t % 10 == 5:
				_shot("18_unlocked")
				# organs (Full Design v3.18): the genome as a Garden feed plan leaves it; the shift's end grows them
				var rs: RoseSim = sim.ensure()
				rs.debug_gene(4, 0.7)
				rs.debug_gene(2, 0.7)
				rs.debug_gene(1, 0.7)
				sim.end_shift()
				sim.day += 1
				screens.show_screen("colony")
				scr = screens.current
				scr.set_view(0)
				for o in rs.organs():
					if String(o.id) == "bulb":
						scr.cam.position = Vector2(int(o.x) * 32 + 16, int(o.y) * 32 + 48)
						scr.selected = {"kind": "tile", "x": int(o.x), "y": int(o.y)}
						scr.card_sig = ""
						scr._draw_card()
						print("[play] a Lume Bulb at %d,%d" % [int(o.x), int(o.y)])
						break
				phase = 25
		25:
			if t % 10 == 5:
				_shot("19_organ")
				screens.show_screen("strain")
				scr = screens.current
				phase = 26
		26:
			if t % 10 == 5:
				_shot("20_strain")
				sim.codex_tab = "STRAINBOOK"
				screens.show_screen("codex")
				scr = screens.current
				phase = 27
		27:
			if t % 10 == 5:
				_shot("21_strainbook")
				# a long campaign's Bench (development: every node), a Hive strain: a Relay Shrine on a Node, built
				var rs: RoseSim = sim.ensure()
				rs.debug_tech_all()
				rs.debug_gene(6, 0.7)
				rs.debug_gene(8, 0.7)
				sim.end_shift()
				sim.day += 1
				print("[play] Hive held: %s" % str(rs.strain().classes.filter(func(c): return String(c.id) == "hive")[0].held))
				screens.show_screen("colony")
				scr = screens.current
				var heart: Dictionary = rs.structures()[0]
				# the Shrine's Parts and Chrome, as the Stash holds them
				var st0: RoseStash = sim.ensure_stash()
				for k in 6:
					st0.add(RoseItems.new().find("prt_pipe"), 70, true)
				for k in 2:
					st0.add(RoseItems.new().find("chr_optic_slag"), 70, true)
				sim.build_tool = "shrine"
				shrine_id = -1
				for r in range(2, 14):
					if shrine_id >= 0:
						break
					for dy in range(-r, r + 1):
						if shrine_id >= 0:
							break
						for dx in range(-r, r + 1):
							if absi(dx) != r and absi(dy) != r:
								continue
							var x := int(heart.x) + 1 + dx
							var y := int(heart.y) + 1 + dy
							if String(rs.tile_info(x, y).get("mat", "")) == "node":
								shrine_id = scr.place_ghost(x, y)
								if shrine_id >= 0:
									break
				sim.build_tool = ""
				scr.tool = ""
				var shifts := 0
				while shifts < 8:
					var built := false
					for st in rs.structures():
						if int(st.id) == shrine_id and bool(st.built):
							built = true
					if built:
						break
					sim.end_shift()
					sim.day += 1
					shifts += 1
				print("[play] the Relay Shrine on a Node (%d) built after %d shifts" % [shrine_id, shifts])
				# its rules, as the card's words set them: when signal > 0 then dim; when white light < 8 shut gates
				rs.set_nerve_rule(shrine_id, 0, 0, true, 0.0, 5)
				rs.set_nerve_rule(shrine_id, 1, 5, false, 8.0, 1)
				for k in 20:
					rs.step()
				for st in rs.structures():
					if int(st.id) == shrine_id:
						scr.set_view(0)
						scr.cam.position = Vector2(int(st.x) * 32 + 16, int(st.y) * 32 + 48)
				scr.selected = {"kind": "structure", "id": shrine_id}
				scr.card_sig = ""
				scr._draw_card()
				phase = 28
		28:
			if t % 10 == 5:
				_shot("22_nerve")
				var n: Dictionary = sim.ensure().nerve_rules(shrine_id)
				print("[play] the Shrine's rules: %s" % str(n.rules.map(func(r): return "%s %s %s" % [String(r.reading_name), String(r.action_name), "holds" if bool(r.held) else "-"])))
				# a route loop: the play's run came home once; the Board shows it, a second run home, then LOOP IT
				print("[play] S1 runs home: %d" % int((sim.ensure().route_loops().homes as Dictionary).get(1, 0)))
				sim.ensure().note_route_home(1)
				screens.show_screen("route")
				scr = screens.current
				phase = 29
		29:
			if t % 10 == 5:
				var why: String = sim.ensure().set_route_loop(sim.ensure_people(), 1, scr.runner_id)
				print("[play] loop with %s: %s" % [sim.slimo_name(scr.runner_id), "set" if why == "" else why])
				scr._refresh_cards()
				phase = 30
		30:
			if t % 10 == 5:
				_shot("23_loop")
				sim.end_shift()
				sim.day += 1
				for l in sim.people_lines:
					if String(l).contains("loop"):
						print("[play] %s" % String(l))
				# the handoff: a bred strain's proposal round a jam, on the Stack view
				var rs: RoseSim = sim.ensure()
				rs.debug_gene(9, 0.75)
				print("[play] the Mat proposes %d tiles" % rs.debug_proposal())
				screens.show_screen("colony")
				scr = screens.current
				var p: Dictionary = rs.proposal()
				var tiles: PackedInt32Array = p.tiles
				if tiles.size() > 0:
					var w: int = rs.width()
					scr.set_view(0)
					scr.cam.position = Vector2((tiles[0] % w) * 32 + 16, (tiles[0] / w) * 32 + 48)
					scr.selected = {"kind": "tile", "x": tiles[0] % w, "y": tiles[0] / w}
					scr.card_sig = ""
					scr._draw_card()
				phase = 31
		31:
			if t % 10 == 5:
				_shot("24_handoff")
				# the breach (Full Design v3.19): the slab over the first grate, its card, the Sump Grate Key
				var rs: RoseSim = sim.ensure()
				sim.ensure_stash().add(RoseItems.new().find("key_grate_sump"), 100, true)
				var hs: Dictionary = rs.structures()[0]
				var heart := Vector2i(int(hs.x), int(hs.y))
				var best := {}
				for cr in rs.crossings():
					if int(cr.below) == 1 and (best.is_empty() or absi(int(cr.x) - heart.x) < absi(int(best.x) - heart.x)):
						best = cr
				slab = best
				scr.set_view(0)
				scr.cam.position = Vector2(int(slab.x) * 32 + 16, int(slab.y) * 32 + 64)
				scr.selected = {"kind": "tile", "x": int(slab.x), "y": int(slab.y)}
				scr.card_sig = ""
				scr._draw_card()
				phase = 32
		32:
			if t % 10 == 5:
				_shot("25_slab")
				var rs: RoseSim = sim.ensure()
				var why: String = rs.open_crossing(int(slab.tile))
				print("[play] the Sump Grate Key on the slab at %d,%d: %s" % [int(slab.x), int(slab.y), "open" if why == "" else why])
				rs.debug_climb(int(slab.tile), 6)
				sim.end_shift()
				sim.day += 1
				print("[play] the city's stratum: S%d" % rs.city_stratum())
				for l in sim.people_lines:
					if String(l).contains("reached"):
						print("[play] %s" % String(l))
				if scr.world.refresh_strata():
					scr.world.full_paint()
				# the frame over the slab: the Culverts revealed above, the Mat up the open shaft
				scr.cam.position = Vector2(int(slab.x) * 32 + 16, int(slab.y) * 32 - 40)
				scr.selected = {"kind": "tile", "x": int(slab.x), "y": int(slab.y)}
				scr.card_sig = ""
				scr._draw_card()
				phase = 33
		33:
			if t % 10 == 5:
				_shot("26_breach")
				# Ivan on the map (Full Design v3.19): selected by his portrait, sent along the Mat to the first market
				screens.show_screen("colony")
				scr = screens.current
				var rs: RoseSim = sim.ensure()
				var w: int = rs.width()
				var v: Dictionary = rs.ivan()
				# Rally at the Heart, where the founders sleep
				print("[play] RALLY: %s" % rs.ivan_rally(sim.ensure_people()))
				# the first market's cell: the tile of Mat nearest its grate
				var g: int = int(rs.attention().filter(func(z): return bool(z.reached))[0].grate)
				var best := Vector2i(int(v.x), int(v.y))
				var bd := 1 << 30
				for dy in range(0, 6):
					for dx in range(-4, 5):
						var x := (g % w) + dx
						var y := (g / w) + dy
						var ti: Dictionary = rs.tile_info(x, y)
						if String(ti.get("mat", "")) in ["cord", "node", "tip"] and absi(dx) + dy < bd:
							bd = absi(dx) + dy
							best = Vector2i(x, y)
				print("[play] Ivan sent to %d,%d: %s" % [best.x, best.y, rs.ivan_order(best.x, best.y)])
				var walked := 0
				while walked < 2000 and int(rs.ivan().goal_x) >= 0:
					rs.step()
					walked += 1
				v = rs.ivan()
				print("[play] Ivan at %d,%d after %d ticks, at market %d" % [int(v.x), int(v.y), walked, int(v.at_market)])
				scr.selected = {"kind": "ivan"}
				scr.cam.position = Vector2(int(v.x) * 32 + 16, int(v.y) * 32 + 64)
				scr.card_sig = ""
				scr._draw_card()
				phase = 40
		40:
			if t % 10 == 5:
				_shot("26a_ivan")
				var rs: RoseSim = sim.ensure()
				rs.debug_earn(2)
				print("[play] PARLEY: %s" % rs.ivan_parley())
				rs.bark_event("law", "Curfew")
				var said := ""
				for k in 60:
					rs.step()
					if String(rs.bark()) != "":
						said = rs.bark()
						break
				print("[play] Ivan says: %s" % said)
				scr.card_sig = ""
				scr._draw_card()
				scr._refresh_lines(rs)
				phase = 41
		41:
			if t % 10 == 5:
				_shot("26b_parley")
				phase = 42
		42:
			if t % 10 == 5:
				# chapter 3 (Full Design v3.19): the Culverts reached, the envoy comes down the grate nearest the Heart
				var rs: RoseSim = sim.ensure()
				rs.debug_chapter(3)
				sim.end_shift()
				sim.day += 1
				var camp: Dictionary = rs.campaign()
				print("[play] %s, the envoy %s at %d" % [String(camp.card), String(camp.accord), int(camp.envoy)])
				var w: int = rs.width()
				scr.set_view(0)
				scr.cam.position = Vector2((int(camp.envoy) % w) * 32 + 16, (int(camp.envoy) / w) * 32 + 96)
				scr.selected = {}
				scr.card_sig = ""
				scr._draw_card()
				scr._refresh_lines(rs)
				scr.world.threat_bodies.queue_redraw()
				print("[play] bodies: %s" % str(rs.threat_bodies().map(func(b): return "%s %d,%d" % [String(b.role), int(b.x), int(b.y)])))
				phase = 34
		34:
			if t % 10 == 5:
				_shot("27_envoy")
				# the chapter's opening panels, as the screen plays them
				sim.panel_replay = "ch3.open"
				sim.panel_return = "colony"
				screens.show_screen("panels")
				scr = screens.current
				phase = 35
		35:
			if t % 10 == 5:
				_shot("28_panel")
				screens.show_screen("colony")
				scr = screens.current
				var why: String = sim.ensure().answer_accord(sim.ensure_people(), true)
				print("[play] ACCEPT: %s" % ("the accord stands" if why == "" else why))
				scr._refresh_lines(sim.ensure())
				phase = 36
		36:
			if t % 10 == 5:
				_shot("29_accord")
				var rs: RoseSim = sim.ensure()
				print("[play] the Tithe on the Charter: %s; %s" % [str(rs.laws().any(func(l): return String(l.id) == "tithe" and String(l.state) in ["signed", "spreading"])), String(rs.campaign().card)])
				# the ending: a Hive strain, chapter 7's Great Cleansing come and gone
				rs.debug_gene(6, 0.82)
				rs.debug_gene(8, 0.8)
				rs.debug_chapter(7)
				sim.end_shift()
				sim.day += 1
				var camp: Dictionary = rs.campaign()
				print("[play] the ending: %s, the warchest %d" % [String(camp.ending), int(camp.warchest)])
				sim.panel_replay = "end.%s" % String(camp.ending)
				screens.show_screen("panels")
				scr = screens.current
				phase = 37
		37:
			if t % 10 == 5:
				_shot("30_ending_panel")
				screens.show_screen("ending")
				scr = screens.current
				phase = 38
		38:
			if t % 10 == 5:
				_shot("31_ending")
				print("[play] %d frames, %d shots -> %s" % [t, shots, out_dir])
				quit(0)
				return true
	if t > 4000:
		print("[play] timed out at phase %d" % phase)
		quit(1)
		return true
	return false
