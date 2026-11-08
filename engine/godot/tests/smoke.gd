extends SceneTree
## Headless smoke test: loads the extension, builds the scale scenario, runs the greybox scene for 240 frames
## at 2 sim ticks per frame, exercises every player verb, and checks the numbers.
## Run:  godot --headless --path godot --script res://tests/smoke.gd

var main: Node2D
var frames := 0
var failures: Array[String] = []
var t_start := 0

func _initialize() -> void:
	t_start = Time.get_ticks_msec()
	var scene := load("res://scenes/Main.tscn")
	if scene == null:
		push_error("Main.tscn did not load")
		quit(2)
		return
	main = scene.instantiate()
	main.agents = 3000
	main.grow_ticks = 500
	root.add_child(main)
	main.ticks_per_frame = 2
	print("[smoke] scene up, extension class RoseSim: ", ClassDB.class_exists("RoseSim"))

func _check(cond: bool, msg: String) -> void:
	if not cond:
		failures.append(msg)

func _process(_delta: float) -> bool:
	frames += 1
	if frames == 30:
		# every player verb once, on the hovered-tile path the UI uses
		var s0: Dictionary = main.sim.stats()
		main.sim.feed(96, 25, 20.0)
		main.sim.bait(110, 25, 0.5)
		main.sim.sever(97, 25)
		main.sim.salt(120, 25, true)
		main.sim.place_relay(96, 12)
		main.world.full_redraw()
		main._cycle_view()
		main._cycle_view()
		print("[smoke] verbs applied at tick ", s0.get("tick", -1))
	if frames < 240:
		return false
	var s: Dictionary = main.sim.stats()
	var elapsed := Time.get_ticks_msec() - t_start
	print("[smoke] ", JSON.stringify(s))
	_check(ClassDB.class_exists("RoseSim"), "RoseSim class missing (extension not loaded)")
	_check(int(s.get("tick", 0)) >= 970, "tick %d: sim did not advance ~480 ticks after 500 of growth" % int(s.get("tick", 0)))
	_check(int(s.get("cells", 0)) > 500, "cells %d: Mat did not grow" % int(s.get("cells", 0)))
	_check(int(s.get("agents", 0)) == 3000, "agents %d" % int(s.get("agents", 0)))
	_check(int(s.get("deliveries", 0)) > 1000, "deliveries %d: cargo loop not running" % int(s.get("deliveries", 0)))
	_check(int(s.get("step_us", 0)) < 20000, "step %d us: over budget" % int(s.get("step_us", 0)))
	_check(main.view_index == 2, "view cycling failed")
	print("[smoke] %d frames in %d ms, last step %d us" % [frames, elapsed, int(s.get("step_us", 0))])
	if failures.is_empty():
		print("[smoke] PASS")
		quit(0)
	else:
		for f in failures:
			printerr("[smoke] FAIL: ", f)
		quit(1)
	return true
