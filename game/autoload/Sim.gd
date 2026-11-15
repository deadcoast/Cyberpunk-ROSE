extends Node
## The one simulation, shared by Colony and Run. Started on first use; the clock lives here.

var sim: RoseSim
var paused := false
var tick_hz := 2.0
var accum := 0.0
var ticks_per_frame := 0   # headless tests: fixed ticks per frame instead of the clock
var seed_value := 11
signal ticked

## The city (Full Design, The city's clock; The founding): one `game::World`, founded as `rose` founds it — the Mat
## steered to the first grate, the Heart, a Route Board, a Bench, the founders, the starting haul, the surface's
## factions — and stepped, shift-ended and read through RoseSim. The Stash and the roster are views onto it.
func ensure() -> RoseSim:
	if sim == null:
		sim = RoseSim.new()
		# New Run's choices (mode, difficulty, strain code): the founding reads them
		if not run_config.is_empty():
			sim.configure_run(String(run_config.mode), String(run_config.difficulty), String(run_config.code), ensure_profile())
		sim.found(seed_value)
	return sim

## The picture's clock in milliseconds: the frames' own time, advanced by each frame's delta, paused or not. Every
## animation phase (pulses, blinks, a strip's frame, a fresh tile's glow) reads this instead of the wall clock, so a
## frame drawn under --fixed-fps repeats byte for byte (Phase G: the shots and play.gd as regression evidence).
var anim_ms := 0.0

func anim_msec() -> int:
	return int(anim_ms)

func _process(delta: float) -> void:
	anim_ms += delta * 1000.0
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
	# Ivan's ooze-time (Full Design v3.19): the clock at a fifth for six of the player's seconds
	var scale := 1.0
	if ooze_left > 0.0:
		ooze_left -= delta
		scale = 0.2
	accum += delta * scale
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
## The city's events as they happen, for the screens' kinetic lines and the sound (Full Design v3.24, Feel): each
## {t, class "landing" | "loss" | "decision", what}.
signal city_event(e: Dictionary)
var _sound_at := -1000.0
## a loaded city goes on writing the log it had rather than starting one
var _city_log_resume := false

func _write_city_log() -> void:
	var events: Array = sim.city_log_new()
	if events.is_empty():
		return
	_reward_sounds(events)
	for e in events:
		city_event.emit(e)
	if _city_log == null:
		DirAccess.make_dir_recursive_absolute("user://telemetry")
		if _city_log_resume and FileAccess.file_exists(city_log_path):
			_city_log = FileAccess.open(city_log_path, FileAccess.READ_WRITE)
			if _city_log != null:
				_city_log.seek_end()
				_city_log.store_line("city  seed %d  loaded at day %d" % [seed_value, day])
		else:
			_city_log = FileAccess.open(city_log_path, FileAccess.WRITE)
			if _city_log != null:
				_city_log.store_line("city  seed %d  founded day %d" % [seed_value, day])
		if _city_log == null:
			return
	for e in events:
		var t := float(e.t)
		_city_log.store_line("%3d:%04.1f  %-8s %s" % [int(t / 60.0), fmod(t, 60.0), String(e["class"]), String(e.what)])
	_city_log.flush()

## Every reward fires a sound with its line (Full Design v3.24, Feel): a landing the reward's blip, the big one for a
## chapter, a trophy, a stratum reached or an ending; a loss its fall. One sound a quarter second at most, the biggest
## of the moment, so a busy minute does not chatter.
func _reward_sounds(events: Array) -> void:
	var now := float(anim_ms)
	if now - _sound_at < 250.0:
		return
	var best := ""
	for e in events:
		var what := String(e.what)
		match String(e["class"]):
			"landing":
				if what.begins_with("chapter") or what.begins_with("Trophy") or what.contains("reached the") or what.begins_with("the ending"):
					best = "sfx.reward.big"
				elif best == "":
					best = "sfx.reward"
			"loss":
				if best != "sfx.reward.big":
					best = "sfx.loss"
	if best != "" and get_node_or_null("/root/Audio") != null:
		get_node("/root/Audio").play(best)
		_sound_at = now

func set_speed(hz: float) -> void:
	tick_hz = hz

## Seconds of ooze-time left on the player's clock.
var ooze_left := 0.0

## Ivan's ooze-time: "" when it holds, else why not.
func ooze() -> String:
	var why: String = ensure().ivan_ooze()
	if why == "":
		ooze_left = 6.0
	return why

# ---------------------------------------------------------------- the colony's clock and the run hand-off
## The shift the city is in (the World's own count: RoseSim.day after a load); provenance and the raider roll read it.
var day := 1
## The last run's haul as it went into the Stash — the rig, the Mat's haul, Spored regrowth: {placed, held, from};
## the Stash screen shows haul_note once
var last_haul := {}
var haul_note := ""
## The last run's outcome, for the Route Board and the Heart. The routes' runs, fog memory, bodies and stranded
## followers are the World's (game::routes, RoseSim.route_runs and the rest), so a save carries them.
var last_outcome := {}
## The city's one Stash (inventory crate through RoseStash); every screen is a view of it.
var stash: RoseStash
## The campaign's panels (Full Design v3.19): where the Panels screen goes back to, a set to replay (the Codex's
## Lore), and a hold that keeps a waiting set from playing (the shots and the play script set it).
var panel_return := "colony"
var panel_replay := ""
var panel_hold := false
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
var bench_tab := ""
var codex_tab := ""
var bench_slot := {}

## Market sites: the grates the Mat has reached (a grate once reached keeps its number), as tile indices.
func markets() -> PackedInt32Array:
	return ensure().sites().get("markets", PackedInt32Array())

## Pits: the Heart first, then every built Pit (the founding put one on every Relay).
func pits() -> PackedInt32Array:
	return ensure().sites().get("pits", PackedInt32Array())

## A market by its grate's number and the faction whose street it is ("GRATE 5 NOMADS").
func market_name(i: int) -> String:
	var names: PackedStringArray = ensure().sites().get("market_names", PackedStringArray())
	return names[i] if i >= 0 and i < names.size() else "GRATE ?"


func pit_name(i: int) -> String:
	return "HEART" if i == 0 else "PIT %d" % i

## Who is on the map changed (a Runner left): the movers follow the roster.
func refresh_movers() -> void:
	ensure().refresh_movers()

## Shift end, whole (RoseSim.colony_end_shift): the colony hands out the jobs and pays the ghosts from the Stash,
## the Stash resolves its queues and the Bench's slots, the Pits take the nutrient and the genome drifts, the
## colony's products land, and every record takes its work and its context (home, meal, light, water, sleep).
func end_shift() -> void:
	_shift_ended(ensure().colony_end_shift(day))

## A shift played at home (RoseSim.colony_play_shift, Full Design v3.21): the city's clock for a whole shift — the
## builders raise the ghosts, the researchers put their points in, the cargo rides — then its end, whole; the next day.
func play_shift() -> void:
	_shift_ended(ensure().colony_play_shift(day))
	day += 1

func _shift_ended(shift: Dictionary) -> void:
	var city := ensure()
	last_shift = shift
	# the Strainbook and the Scrapdex outlive the run (Full Design v3.19, Meta)
	if ensure_profile().absorb(city) > 0:
		save_profile()
	shift_report = last_shift.get("report", {})
	people_lines = last_shift.get("lines", PackedStringArray())
	# recover contracts' bodies were laid on their routes by the World (game::routes); the shift's end is a save
	autosave()

## The structure def the Colony's build tool places next ("" none); set by the Build menu.
var build_tool := ""

## The roster (Full Design, Creation): a view onto the World's, founded with the city.
func ensure_people() -> RosePeople:
	if people == null:
		people = RosePeople.new()
		people.attach(ensure())
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

## The Stash (game::world::STARTING_HAUL, founded with the city as `rose city` founds it): a view onto the World's.
func ensure_stash() -> RoseStash:
	if stash == null:
		stash = RoseStash.new()
		stash.attach(ensure())
	return stash

## A route's seed is fixed by the save and the stratum, so a route can be re-run and learned.
func route_seed(stratum: int) -> int:
	return ensure().route_seed(seed_value, stratum)

## The Route Board's strata (Full Design v3.19): the city's stratum − 1 to + 1, held to the stack.
func board_strata() -> PackedInt32Array:
	return ensure().board_strata()

## A stratum's route name (data/strata.toml).
func region_name(stratum: int) -> String:
	for st in ensure().strata():
		if int(st.n) == stratum:
			return String(st.region)
	return "S%d DRAIN" % stratum

func memory_of(stratum: int, seed_v: int) -> PackedByteArray:
	return ensure().route_memory(stratum, seed_v)

## Our dead on a route and the recover contracts' bodies laid there: [{seed, stratum, x, y, chunk, day, runner, rig,
## raided, contract}].
func corpses_on(stratum: int, seed_v: int) -> Array:
	return Array(ensure().bodies_on(stratum, seed_v))

## Followers left at a route's extraction tile: [ids].
func stranded_on(stratum: int, seed_v: int) -> Array:
	return Array(ensure().stranded_on(stratum, seed_v))

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
	# Ivan's line on the run, and the campaign's note of it (chapter 5 reads a Vault run's best item)
	ensure().campaign_run_home(stratum, runner_name, String(o.get("result", "")) == "extracted", String(o.get("result", "")) == "died", o.get("rig", PackedByteArray()))
	if String(o.get("result", "")) == "extracted":
		# a run by hand home: it counts toward a loop on the route (Full Design v3.18, Route loops)
		ensure().note_route_home(stratum)
		var rig: PackedByteArray = o.get("rig", PackedByteArray())
		held += maxi(0, st.import_rig(rig)) if not rig.is_empty() else 0
	for key in ["mat_haul", "regrown"]:
		var b: PackedByteArray = o.get(key, PackedByteArray())
		if int(o.get(key + "_count", 0)) > 0 and not b.is_empty():
			held += maxi(0, st.import_items(b))
	var from := String(crew.get("route_name", ""))
	last_haul = {"placed": st.count() - before, "held": held, "from": ("S%d %s" % [stratum, from]) if from != "" else ("S%d route %d" % [stratum, seed_value])}
	if int(last_haul.placed) + held > 0:
		haul_note = "haul from %s: %d in%s" % [last_haul.from, int(last_haul.placed), ("   %d waiting at the Heart for room" % held) if held > 0 else ""]
	if o.has("corpse"):
		ensure().add_body(stratum, seed_value, o.corpse, runner_name)
	ensure().strand(stratum, seed_value, o.get("followers_stranded", PackedInt32Array()))
	ensure().remember_route(stratum, seed_value, o.get("route_memory", PackedByteArray()))
	note_run(stratum, seed_value)
	# Oathkeepers the run killed: their Attention lands in the Heart's zone at the shift's end; raiders count toward
	# the clear-raiders contracts
	ensure().add_run_kills(int(o.get("oath_kills", 0)), int(o.get("raider_kills", 0)))
	ensure().add_run_trophies(o.get("killed", PackedStringArray()), day)
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

func runs_of(stratum: int, seed_v: int) -> int:
	return ensure().route_runs(stratum, seed_v)

func note_run(stratum: int, seed_v: int) -> void:
	ensure().note_route_run(stratum, seed_v)

## A nonce for this run's loot: the colony tick at departure, so re-running a route rolls fresh contents.
func run_nonce() -> int:
	var s: Dictionary = ensure().stats()
	return int(s.get("tick", 0)) * 131 + day

# ---------------------------------------------------------------- the profile, New Run, the Daily (Full Design v3.19)
## What outlives a run (the save crate's profile): the menu's endings and Daily best, New Run's last code, the Codex.
## The tests keep their own (profile_path), so a test's ending or Daily never lands in the player's.
var profile_path := "user://profile"
var profile: RoseProfile
## New Run's choices for the next founding: {mode, difficulty, code}; empty: the campaign at Culvert, no code.
var run_config := {}
## The ending has been written to the profile (the Ending screen writes it once).
var ending_recorded := false
## What the New Run screen opens with: {mode, code} (the menu's Endless Sump, the Ending's NEW GAME+).
var newrun_preset := {}
## Every variable's starting value, so a New Run founds from a clean slate; the tests' settings are kept.
var _defaults := {}
const _KEEP := ["_defaults", "ticks_per_frame", "panel_hold", "profile", "profile_path", "save_dir", "kinetic"]
## Labels that land slam in (ScreenBase.slam); the screenshot scripts turn it off so a shot shows the text, not a glitch
## frame of it (their city ticks every frame, so a line lands every frame).
var kinetic := true

func _init() -> void:
	for p in get_script().get_script_property_list():
		if p.usage & PROPERTY_USAGE_SCRIPT_VARIABLE and not (p.name in _KEEP):
			var v = get(p.name)
			_defaults[p.name] = v.duplicate(true) if (v is Array or v is Dictionary) else v

func ensure_profile() -> RoseProfile:
	if profile == null:
		profile = RoseProfile.new()
		if FileAccess.file_exists(profile_path) and not profile.load_bytes(FileAccess.get_file_as_bytes(profile_path)):
			# a profile that does not read is set aside, never overwritten in place
			DirAccess.rename_absolute(profile_path, profile_path + ".bad")
	return profile

func save_profile() -> void:
	var f := FileAccess.open(profile_path, FileAccess.WRITE)
	if f != null:
		f.store_buffer(ensure_profile().to_bytes())

# ---------------------------------------------------------------- saves (Full Design: Saves)
## One autosave a run, written at every shift's end and as a run leaves the Board, and three slots the player writes
## from Settings. A save is the World's snapshot (RoseSim.save_bytes) with the view state below beside it. Written to
## a temporary file and renamed over the old; a save that does not read is renamed .bad and never overwritten. The
## tests keep their own folder (save_dir), so a test never writes over the player's saves.
var save_dir := "user://saves"
const SLOTS := ["auto", "slot1", "slot2", "slot3"]
## When the run was founded (seconds since 1970), kept in its saves; the last save written or loaded: {slot, label,
## saved_unix, day}; why the last save failed ("" none).
var created_unix := 0
var last_save := {}
var save_error := ""
## The view state a save carries beside the World: where the player left the camera, the bookmarks and the overlay,
## the alerts already shown, the last run's card and haul, New Run's choices.
const _UI_KEYS := ["bookmarks", "colony_cam", "overlay", "alerts_shown", "focus_slimo", "last_outcome", "last_haul", "run_config", "ending_recorded", "created_unix"]

func save_path(slot: String) -> String:
	return "%s/%s.rose" % [save_dir, slot]

## Write the city to a slot ("auto", "slot1".."slot3"): "" or why not.
func save_game(slot: String) -> String:
	if sim == null or sim.campaign().is_empty():
		return "no city to save"
	DirAccess.make_dir_recursive_absolute(save_dir)
	var now := int(Time.get_unix_time_from_system())
	if created_unix == 0:
		created_unix = now
	var ui := {}
	for k in _UI_KEYS:
		ui[k] = get(k)
	var bytes: PackedByteArray = sim.save_bytes(var_to_bytes(ui), created_unix, now)
	if bytes.is_empty():
		return "no city to save"
	var path := save_path(slot)
	var tmp := path + ".tmp"
	var f := FileAccess.open(tmp, FileAccess.WRITE)
	if f == null:
		return "cannot write %s" % tmp
	f.store_buffer(bytes)
	f.close()
	if DirAccess.rename_absolute(tmp, path) != OK:
		return "cannot write %s" % path
	var h: Dictionary = RoseSim.save_header(bytes)
	last_save = {"slot": slot, "label": String(h.get("label", "")), "saved_unix": now, "day": int(h.get("day", day))}
	return ""

## The autosave (every shift's end, a run's departure). A failure is kept for Settings to show; play goes on.
func autosave() -> void:
	if ticks_per_frame > 0 and save_dir == "user://saves":
		return   # a headless test that kept the player's folder writes nothing there
	save_error = save_game("auto")

## What each slot holds: {slot: {why, label, day, saved_unix, ...} or {} when empty}.
func save_slots() -> Dictionary:
	var out := {}
	for slot in SLOTS:
		var path := save_path(slot)
		if not FileAccess.file_exists(path):
			out[slot] = {}
			continue
		out[slot] = RoseSim.save_header(FileAccess.get_file_as_bytes(path))
	return out

## The newest save that reads ("" none): what CONTINUE loads when this session holds no city.
func newest_save() -> String:
	var best := ""
	var best_t := []
	var slots := save_slots()
	for slot in slots:
		var h: Dictionary = slots[slot]
		# the latest written; within one second, the later shift
		var t := [int(h.get("saved_unix", 0)), int(h.get("day", 0)), int(h.get("tick", 0))]
		if not h.is_empty() and String(h.get("why", "")) == "" and (best_t.is_empty() or t > best_t):
			best = slot
			best_t = t
	return best

## Load a slot: the city so far is let go and the saved one takes its place, with its view state. "" or why not; a
## file that does not read is set aside as .bad and the city in hand is kept.
func load_game(slot: String) -> String:
	var path := save_path(slot)
	if not FileAccess.file_exists(path):
		return "no save in %s" % slot
	var bytes := FileAccess.get_file_as_bytes(path)
	var h: Dictionary = RoseSim.save_header(bytes)
	if String(h.get("why", "")) != "":
		DirAccess.rename_absolute(path, path + ".bad")
		return "%s: %s (set aside as %s.bad)" % [slot, String(h.why), path.get_file()]
	var next := RoseSim.new()
	var r: Dictionary = next.load_bytes(bytes)
	if String(r.get("why", "")) != "":
		DirAccess.rename_absolute(path, path + ".bad")
		return "%s: %s (set aside as %s.bad)" % [slot, String(r.why), path.get_file()]
	if _city_log != null:
		_city_log.close()
		_city_log = null
	for k in _defaults:
		var v = _defaults[k]
		set(k, v.duplicate(true) if (v is Array or v is Dictionary) else v)
	sim = next
	day = sim.day()
	seed_value = int(sim.campaign().get("seed", seed_value))
	var ui = bytes_to_var(r.get("ui", PackedByteArray()))
	if ui is Dictionary:
		for k in _UI_KEYS:
			if ui.has(k):
				set(k, ui[k])
	last_save = {"slot": slot, "label": String(r.get("label", "")), "saved_unix": int(r.get("saved_unix", 0)), "day": day}
	_city_log_resume = true
	return ""

## New Run: the city so far is let go and the next is founded on the seed, mode ("campaign" or "endless"), difficulty
## and strain code ("" none; New Game+ otherwise). "" or why not — a code that does not read keeps the city.
func new_run(seed_v: int, mode: String, difficulty: String, code: String) -> String:
	var why: String = RoseSim.new().configure_run(mode, difficulty, code, ensure_profile())
	if why != "":
		return why
	if _city_log != null:
		_city_log.close()
		_city_log = null
	for k in _defaults:
		var v = _defaults[k]
		set(k, v.duplicate(true) if (v is Array or v is Dictionary) else v)
	seed_value = seed_v
	run_config = {"mode": mode, "difficulty": difficulty, "code": code}
	ensure()
	return ""

## A fixed date in place of the system's (YYYYMMDD; 0 none): the screenshot scripts set it so the Menu's Daily
## Seed reads the same every day.
var today_fixed := 0

## Today as the Daily's seed (YYYYMMDD, the system's date).
func today() -> int:
	if today_fixed > 0:
		return today_fixed
	var d := Time.get_date_dict_from_system()
	return int(d.year) * 10000 + int(d.month) * 100 + int(d.day)

## The Daily Seed: the day's one try at its route with its Runner and the fixed rig (Full Design v3.19). "" and the
## Run is on screen, or why not.
func start_daily(date: int) -> String:
	var tried: int = ensure_profile().daily_of(date)
	if tried >= 0:
		return "the Daily of %d is tried: score %d" % [date, tried]
	var d: Dictionary = RoseRun.daily(date)
	var r: Dictionary = RosePeople.daily_runner(date)
	launch = {"daily": date, "stratum": int(d.stratum), "seed": int(d.seed), "name": String(d.name), "hand": String(d.hand), "kit": d.kit, "runner": r, "runner_name": String(r.full_name), "followers": []}
	Screens.show_screen("run")
	return ""
