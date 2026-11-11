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
	root.add_child((load("res://Main.tscn") as PackedScene).instantiate())
	out_dir = ProjectSettings.globalize_path("res://../docs/shots")
	DirAccess.make_dir_recursive_absolute(out_dir)

func _shot(name: String) -> void:
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png(out_dir + "/play_" + name + ".png")
	shots += 1

var t0 := 0
var runner_id := -1
var mend_id := -1

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
				print("[play] %d frames, %d shots -> %s" % [t, shots, out_dir])
				quit(0)
				return true
	if t > 4000:
		print("[play] timed out at phase %d" % phase)
		quit(1)
		return true
	return false
