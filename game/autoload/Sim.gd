extends Node
## The one simulation, shared by Colony and Run. Started on first use; the clock lives here.

var sim: RoseSim
var paused := false
var tick_hz := 2.0
var accum := 0.0
var ticks_per_frame := 0   # headless tests: fixed ticks per frame instead of the clock
var seed_value := 11
signal ticked

## The city (Full Design, The city's clock; The founding): the Mat steered to the first grate, on its own clock,
## the roster's slimos and the Stash's cargo — no phantoms. The Stash is bound so cargo lands into it as it arrives.
func ensure() -> RoseSim:
	if sim == null:
		sim = RoseSim.new()
		sim.start_city(seed_value)
		# the city on the founding Mat: the Heart, a Route Board, a Bench
		sim.found_city()
		sim.bind_stash(ensure_stash())
		sim.refresh_movers(ensure_people(), day)
	return sim

func _process(delta: float) -> void:
	if sim == null:
		return
	if ticks_per_frame > 0:
		for i in ticks_per_frame:
			sim.step()
		_write_city_log()
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
		_write_city_log()
		ticked.emit()

# ---------------------------------------------------------------- what the Colony HUD keeps between screens
## Bookmarks 1–9: {n: {pos, view}}; the camera and the overlay as the player left them; the alerts already shown
## (so coming back does not replay the queue); the slimo the Slimo card opens on.
var bookmarks := {}
var colony_cam := {}
var overlay := ""
var alerts_shown: Array = []
var focus_slimo := -1

## The city's log (Full Design v3.12, The city's log) goes to the save folder beside the runs' cadence logs: one
## line an event, the city's clock in seconds, its class, what happened.
var city_log_path := "user://telemetry/city.log"
var _city_log: FileAccess

func _write_city_log() -> void:
	var events: Array = sim.city_log_new()
	if events.is_empty():
		return
	if _city_log == null:
		DirAccess.make_dir_recursive_absolute("user://telemetry")
		_city_log = FileAccess.open(city_log_path, FileAccess.WRITE)
		if _city_log == null:
			return
		_city_log.store_line("city  seed %d  founded day %d" % [seed_value, day])
	for e in events:
		var t := float(e.t)
		_city_log.store_line("%3d:%04.1f  %-8s %s" % [int(t / 60.0), fmod(t, 60.0), String(e["class"]), String(e.what)])
	_city_log.flush()

func set_speed(hz: float) -> void:
	tick_hz = hz

# ---------------------------------------------------------------- the colony's clock and the run hand-off
## Day counter for provenance and the raider roll; a shift is a sim day for now.
var day := 1
## How many runs each route (key "s<stratum>:<seed>") has had: first-run generosity reads 0.
var route_runs := {}
## The last run's haul as it went into the Stash — the rig, the Mat's haul, Spored regrowth: {placed, held, from};
## the Stash screen shows haul_note once
var last_haul := {}
var haul_note := ""
## Our dead on the routes: [{seed, stratum, x, y, chunk, day, runner, rig: PackedByteArray, raided}] (save v3 corpses).
var corpses: Array = []
## Followers stranded at an extraction tile, by route key -> [ids].
var stranded := {}
## The last run's outcome, for the Route Board and the Heart.
var last_outcome := {}
## Route memory bits per route key, from each run's outcome (save v3 route_memory).
var route_memory := {}
## The city's one Stash (inventory crate through RoseStash); every screen is a view of it.
var stash: RoseStash
## Strata whose route cards are open; Phase E's Relay sets this, the three launch cards are open until then.
var strata_open := 3
## Region names per stratum at launch (Full Design, The Route Board).
const REGIONS := {1: "SUMP EAST", 2: "CULVERT NORTH", 3: "STORM DRAIN"}
## The roster (people crate through RosePeople): every slimo's record, founded from the save's seed on first use.
var people: RosePeople
## What the Route Board hands the Run: {stratum, seed, name, rig: PackedByteArray, runner: RosePeople.runner_setup,
## runner_id, followers: [RosePeople.follower_setup...]}. Empty: the Run's own defaults.
var launch := {}
## The last shift's record lines (XP, bonds, injuries, unrest) from the roster, shown on the Stash with the report.
var people_lines: PackedStringArray = PackedStringArray()
## The whole last shift (RoseSim.colony_end_shift): report, lines, assignments, waiting ghosts, elections, petitions.
var last_shift := {}
## The last shift's report from the ledger ({sold, earned, fed, eddies, deliveries}), shown once on the Stash.
var shift_report := {}
## The item the Stash sent to the Bench with Craft (a RoseStash.items() entry), consumed by the Bench screen.
var bench_slot := {}

## Market sites: the grates the Mat has reached (a grate once reached keeps its number), as tile indices.
func markets() -> PackedInt32Array:
	return ensure().sites().get("markets", PackedInt32Array())

## Pits: the Heart first, then every built Pit (the founding put one on every Relay).
func pits() -> PackedInt32Array:
	return ensure().sites().get("pits", PackedInt32Array())

func market_name(i: int) -> String:
	return "GRATE %d" % (i + 1)

func pit_name(i: int) -> String:
	return "HEART" if i == 0 else "PIT %d" % i

## Who is on the map changed (a Runner left): the movers follow the roster.
func refresh_movers() -> void:
	ensure().refresh_movers(ensure_people(), day)

## Shift end, whole (RoseSim.colony_end_shift): the colony hands out the jobs and pays the ghosts from the Stash,
## the Stash resolves its queues and the Bench's slots, the Pits take the nutrient and the genome drifts, the
## colony's products land, and every record takes its work and its context (home, meal, light, water, sleep).
func end_shift() -> void:
	var st := ensure_stash()
	var city := ensure()
	last_shift = city.colony_end_shift(ensure_people(), st, day)
	shift_report = last_shift.get("report", {})
	people_lines = last_shift.get("lines", PackedStringArray())

## The structure def the Colony's build tool places next ("" none); set by the Build menu.
var build_tool := ""

## The roster, founded once per session from the save's seed (Full Design, Creation).
func ensure_people() -> RosePeople:
	if people == null:
		people = RosePeople.new()
		people.found(seed_value, day)
	return people

## The crew's names, for the Route Board's stranded line and the Run's chips.
func slimo_name(id: int) -> String:
	var rec: Dictionary = ensure_people().record(id)
	if rec.is_empty():
		for m in ensure_people().memorial():
			if int(m.id) == id:
				return String(m.full_name)
		return "#%d" % id
	return String(rec.full_name)

## The Stash, created once for the session with its starting haul (game::world::STARTING_HAUL, the same one
## `rose city` founds with).
func ensure_stash() -> RoseStash:
	if stash == null:
		stash = RoseStash.new()
		stash.found()
	return stash

## A route's seed is fixed by the save and the stratum, so a route can be re-run and learned.
func route_seed(stratum: int) -> int:
	return seed_value * 7 + stratum

func region_name(stratum: int) -> String:
	return String(REGIONS.get(stratum, "S%d DRAIN" % stratum))

func memory_of(stratum: int, seed_v: int) -> PackedByteArray:
	return route_memory.get(route_key(stratum, seed_v), PackedByteArray())

func corpses_on(stratum: int, seed_value: int) -> Array:
	var out := []
	for c in corpses:
		if int(c.stratum) == stratum and int(c.seed) == seed_value:
			out.append(c)
	return out

## Record what a run gave back (RoseRun.outcome()) and advance the shift. `crew` is the launch's {runner_id,
## followers} (empty for a run without a record: tests).
func take_outcome(stratum: int, seed_value: int, runner_name: String, o: Dictionary, crew: Dictionary = {}) -> void:
	last_outcome = o.duplicate()
	last_outcome["stratum"] = stratum
	last_outcome["seed"] = seed_value
	# the roster takes the run: XP, bonds, injuries, the dead to the memorial
	if crew.has("runner_id"):
		var fids := PackedInt32Array()
		for f in crew.get("followers", []):
			fids.append(int(f.id))
		var rep := {
			"runner": int(crew.runner_id), "followers": fids,
			"extracted": String(o.get("result", "")) == "extracted", "runner_died": String(o.get("result", "")) == "died",
			"followers_extracted": o.get("followers_extracted", PackedInt32Array()),
			"followers_stranded": o.get("followers_stranded", PackedInt32Array()),
			"followers_dead": o.get("followers_dead", PackedInt32Array()),
			"pulls": int(o.get("pulls", 0)), "items_taken": int(o.get("items_taken", 0)), "kills": int(o.get("kills", 0)),
			"hits_survived": int(o.get("hits_survived", 0)), "follower_strikes": int(o.get("follower_strikes", 0)),
			"injuries": o.get("injuries", PackedStringArray()),
		}
		last_outcome["record_lines"] = ensure_people().apply_run(rep)
	# the haul into the Stash before the shift ends (game::store::take_haul): the rig on extraction, and either way
	# what the Mat hauled home and what regrew at the Heart; what the grid cannot hold waits at the Heart for a
	# stockpile, and the shift's quotas and settling see it
	var st: RoseStash = ensure_stash()
	var before: int = st.count()
	var held := 0
	if String(o.get("result", "")) == "extracted":
		var rig: PackedByteArray = o.get("rig", PackedByteArray())
		held += maxi(0, st.import_rig(rig)) if not rig.is_empty() else 0
	for key in ["mat_haul", "regrown"]:
		var b: PackedByteArray = o.get(key, PackedByteArray())
		if int(o.get(key + "_count", 0)) > 0 and not b.is_empty():
			held += maxi(0, st.import_items(b))
	last_haul = {"placed": st.count() - before, "held": held, "from": "S%d route %d" % [stratum, seed_value]}
	if int(last_haul.placed) + held > 0:
		haul_note = "haul from %s: %d in%s" % [last_haul.from, int(last_haul.placed), ("   %d waiting at the Heart for room" % held) if held > 0 else ""]
	if o.has("corpse"):
		var c: Dictionary = o.corpse
		corpses.append({"seed": seed_value, "stratum": stratum, "x": int(c.x), "y": int(c.y), "chunk": String(c.chunk), "day": int(c.day), "runner": runner_name, "rig": c.rig, "raided": bool(c.raided)})
	var stranded_ids: PackedInt32Array = o.get("followers_stranded", PackedInt32Array())
	if stranded_ids.size() > 0:
		stranded[route_key(stratum, seed_value)] = Array(stranded_ids)
	var mem: PackedByteArray = o.get("route_memory", PackedByteArray())
	if not mem.is_empty():
		route_memory[route_key(stratum, seed_value)] = mem
	note_run(stratum, seed_value)
	# Oathkeepers the run killed: their Attention lands in the Heart's zone at the shift's end
	ensure().add_run_kills(int(o.get("oath_kills", 0)))
	launch = {}
	end_shift()
	day += 1

## This shift's Flood roll for a run on it (0-3), -1 when it is not a Flood shift; the next shift's for overtime.
func flood_roll() -> int:
	return ensure().flood_roll()

func next_flood_roll() -> int:
	return ensure().next_flood_roll()

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
