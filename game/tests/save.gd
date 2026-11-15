extends SceneTree
## Quit and relaunch (Full Design: Saves; Phase G, PAC-102): two processes. `-- write` founds the city, plays three
## shifts (each ending in its autosave), writes slot 2, plays two shifts more and quits, leaving the digests it saw;
## `-- read`, a fresh process, takes CONTINUE (the newest save: the autosave after the fifth shift) and matches its
## digest, then loads slot 2, matches the third shift's, plays the same two shifts and matches the fifth's. The
## route book and the view state ride along and are checked too. Exit 0 PASS, 1 FAIL.
##   godot --headless --path game --script res://tests/save.gd -- write
##   godot --headless --path game --script res://tests/save.gd -- read

var sim
var audio
var failures := 0
const DIR := "user://saves_test_save"
const EXPECT := "user://saves_test_save/expect.txt"

func _autoload(n: String, path: String) -> Node:
	var node: Node = load(path).new()
	node.name = n
	root.add_child(node)
	return node

func _check(ok: bool, what: String) -> void:
	if not ok:
		failures += 1
		print("[save] FAIL ", what)

func _digest() -> String:
	return "day %d  %s" % [sim.day, " ".join(sim.sim.digest_split())]

func _initialize() -> void:
	_autoload("Assets", "res://autoload/Assets.gd")
	# the sound channel, silent: the slots resolve and what plays is counted
	audio = _autoload("Audio", "res://autoload/Audio.gd")
	audio.muted = true
	_autoload("Screens", "res://autoload/Screens.gd")
	sim = _autoload("Sim", "res://autoload/Sim.gd")
	sim.profile_path = "user://profile_test_save"
	sim.save_dir = DIR
	sim.ticks_per_frame = 2
	sim.panel_hold = true
	var phase := ""
	for a in OS.get_cmdline_user_args():
		phase = a
	if phase == "write":
		_write()
	elif phase == "read":
		_read()
	else:
		print("[save] FAIL pass -- write or -- read")
		failures += 1
	print("[save] %s %s" % [phase, "PASS" if failures == 0 else "FAIL (%d)" % failures])
	quit(0 if failures == 0 else 1)

func _write() -> void:
	for slot in sim.SLOTS:
		for ext in ["", ".tmp", ".bad"]:
			DirAccess.remove_absolute(ProjectSettings.globalize_path(sim.save_path(slot) + ext))
	DirAccess.remove_absolute(ProjectSettings.globalize_path(sim.profile_path))
	sim.ensure_stash()
	sim.ensure_people()
	for i in 3:
		sim.play_shift()
	_check(sim.save_error == "", "the autosave: " + sim.save_error)
	_check(FileAccess.file_exists(sim.save_path("auto")), "no autosave after three shifts")
	# what rides with the city: a route's runs, memory and a body (the World's route book), and the view state
	var seed_v: int = sim.route_seed(2)
	sim.note_run(2, seed_v)
	sim.ensure().remember_route(2, seed_v, PackedByteArray([7, 8, 9]))
	sim.ensure().add_body(2, seed_v, {"x": 40, "y": 13, "chunk": "sump_culvert", "day": 3, "rig": PackedByteArray([1, 2]), "raided": false}, "Pell")
	sim.bookmarks = {1: {"pos": Vector2(96, 260), "view": "street"}}
	sim.overlay = "signal"
	var why: String = sim.save_game("slot2")
	_check(why == "", "slot 2: " + why)
	var d3 := _digest()
	for i in 2:
		sim.play_shift()
	var d5 := _digest()
	var f := FileAccess.open(EXPECT, FileAccess.WRITE)
	f.store_line(d3)
	f.store_line(d5)
	f.store_line(str(seed_v))
	f.close()
	print("[save] wrote: slot 2 at %s; autosave at %s; %d bytes" % [d3.left(12), d5.left(12), FileAccess.get_file_as_bytes(sim.save_path("auto")).size()])

func _read() -> void:
	var lines := FileAccess.get_file_as_string(EXPECT).split("\n", false)
	_check(lines.size() == 3, "the expected digests")
	if lines.size() != 3:
		return
	var seed_v := int(lines[2])
	_check(sim.sim == null, "a fresh process holds no city")
	var newest: String = sim.newest_save()
	_check(newest == "auto", "CONTINUE loads the newest save: %s" % newest)
	var t0 := Time.get_ticks_msec()
	var why: String = sim.load_game(newest)
	var load_ms := Time.get_ticks_msec() - t0
	_check(why == "", "load the autosave: " + why)
	_check(_digest() == lines[1], "CONTINUE: the fifth shift's city\n  wrote %s\n  read  %s" % [lines[1], _digest()])
	why = sim.load_game("slot2")
	_check(why == "", "load slot 2: " + why)
	_check(_digest() == lines[0], "slot 2: the third shift's city\n  wrote %s\n  read  %s" % [lines[0], _digest()])
	_check(sim.runs_of(2, seed_v) == 1, "the route's runs: %d" % sim.runs_of(2, seed_v))
	_check(sim.memory_of(2, seed_v) == PackedByteArray([7, 8, 9]), "the route's memory")
	var bodies: Array = sim.corpses_on(2, seed_v)
	_check(bodies.size() == 1 and String(bodies[0].runner) == "Pell" and int(bodies[0].x) == 40, "the body on the route: %s" % str(bodies))
	_check(sim.bookmarks.has(1) and sim.overlay == "signal", "the view state: %s %s" % [str(sim.bookmarks), sim.overlay])
	sim.ensure_stash()
	sim.ensure_people()
	for i in 2:
		sim.play_shift()
	_check(_digest() == lines[1], "slot 2 played on: the fifth shift's city\n  wrote %s\n  read  %s" % [lines[1], _digest()])
	print("[save] read: CONTINUE loaded in %d ms; slot 2 played on to the same city" % load_ms)
