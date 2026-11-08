extends Node
## The one simulation, shared by Colony and Run. Started on first use; the clock lives here.

var sim: RoseSim
var paused := false
var tick_hz := 2.0
var accum := 0.0
var ticks_per_frame := 0   # headless tests: fixed ticks per frame instead of the clock
var seed_value := 11
var agents := 3000
var grow_ticks := 500
signal ticked

func ensure() -> RoseSim:
	if sim == null:
		sim = RoseSim.new()
		sim.start_scale(seed_value, agents, grow_ticks)
	return sim

func _process(delta: float) -> void:
	if sim == null:
		return
	if ticks_per_frame > 0:
		for i in ticks_per_frame:
			sim.step()
		ticked.emit()
		return
	if paused:
		return
	accum += delta
	var period := 1.0 / tick_hz
	var n := 0
	while accum >= period and n < 8:
		accum -= period
		sim.step()
		n += 1
	if n > 0:
		ticked.emit()

func set_speed(hz: float) -> void:
	tick_hz = hz

# ---------------------------------------------------------------- the colony's clock and the run hand-off
## Day counter for provenance and the raider roll; a shift is a sim day for now.
var day := 1
## How many runs each route (key "s<stratum>:<seed>") has had: first-run generosity reads 0.
var route_runs := {}
## A Runner's rig handed from the Run to the Stash on extraction (RoseRun.rig_export bytes), consumed once.
var pending_haul := PackedByteArray()
var pending_haul_from := ""
## Loose items coming home with or without the Runner: the Mat haul and Spored regrowth (postcard Vec<ItemInstance>).
var pending_items: Array[PackedByteArray] = []
## Our dead on the routes: [{seed, stratum, x, y, chunk, day, runner, rig: PackedByteArray, raided}] (save v3 corpses).
var corpses: Array = []
## Followers stranded at an extraction tile, by route key -> [ids].
var stranded := {}
## The last run's outcome, for the Route Board and the Heart.
var last_outcome := {}

func corpses_on(stratum: int, seed_value: int) -> Array:
	var out := []
	for c in corpses:
		if int(c.stratum) == stratum and int(c.seed) == seed_value:
			out.append(c)
	return out

## Record what a run gave back (RoseRun.outcome()) and advance the shift.
func take_outcome(stratum: int, seed_value: int, runner_name: String, o: Dictionary) -> void:
	last_outcome = o.duplicate()
	last_outcome["stratum"] = stratum
	last_outcome["seed"] = seed_value
	if String(o.get("result", "")) == "extracted":
		pending_haul = o.get("rig", PackedByteArray())
		pending_haul_from = "S%d route %d" % [stratum, seed_value]
	elif o.has("corpse"):
		var c: Dictionary = o.corpse
		corpses.append({"seed": seed_value, "stratum": stratum, "x": int(c.x), "y": int(c.y), "chunk": String(c.chunk), "day": int(c.day), "runner": runner_name, "rig": c.rig, "raided": bool(c.raided)})
	for key in ["mat_haul", "regrown"]:
		var b: PackedByteArray = o.get(key, PackedByteArray())
		if int(o.get(key + "_count", 0)) > 0 and not b.is_empty():
			pending_items.append(b)
	var stranded_ids: PackedInt32Array = o.get("followers_stranded", PackedInt32Array())
	if stranded_ids.size() > 0:
		stranded[route_key(stratum, seed_value)] = Array(stranded_ids)
	note_run(stratum, seed_value)
	day += 1

func route_key(stratum: int, seed_value: int) -> String:
	return "s%d:%d" % [stratum, seed_value]

func runs_of(stratum: int, seed_value: int) -> int:
	return int(route_runs.get(route_key(stratum, seed_value), 0))

func note_run(stratum: int, seed_value: int) -> void:
	var k := route_key(stratum, seed_value)
	route_runs[k] = int(route_runs.get(k, 0)) + 1

## A nonce for this run's loot: the colony tick at departure, so re-running a route rolls fresh contents.
func run_nonce() -> int:
	var s: Dictionary = ensure().stats()
	return int(s.get("tick", 0)) * 131 + day
