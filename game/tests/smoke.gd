extends SceneTree
## Smoke: the extension loads, every screen instantiates, every asset slot resolves, the Colony runs the sim, the
## Stash round-trips through the save crate, the Route Board packs a rig from it, the Run plays through the input
## word to extraction and the Stash takes the haul. Headless.
## Run:  godot --headless --path game --script res://tests/smoke.gd

var main: Node
var frames := 0
var failures: Array[String] = []
var sim_sent_handle := -1
var screen_ids: Array = []
var smoke_runner := -1
var smoke_ghost := -1
var smoke_signed := false
var screen_i := -1
var frames_on_screen := 0
var resolved_per_screen := {}
var t_start := 0

var assets: Node
var screens: Node
var sim: Node

func _autoload(name: String, path: String) -> Node:
	# a SceneTree script does not get the project's autoloads; mount them under root by name so scripts resolve them
	var n := root.get_node_or_null(name)
	if n == null:
		n = (load(path) as GDScript).new()
		n.name = name
		root.add_child(n)
	return n

func _initialize() -> void:
	t_start = Time.get_ticks_msec()
	assets = _autoload("Assets", "res://autoload/Assets.gd")
	screens = _autoload("Screens", "res://autoload/Screens.gd")
	sim = _autoload("Sim", "res://autoload/Sim.gd")
	var scene := load("res://Main.tscn")
	main = scene.instantiate()
	root.add_child(main)
	sim.ticks_per_frame = 2
	screen_ids = screens.ids()
	print("[smoke] extension RoseSim: %s  RoseItems: %s  RoseStash: %s  RoseRun: %s" % [ClassDB.class_exists("RoseSim"), ClassDB.class_exists("RoseItems"), ClassDB.class_exists("RoseStash"), ClassDB.class_exists("RoseRun")])

func _check(cond: bool, msg: String) -> void:
	if not cond:
		failures.append(msg)

func _process(_delta: float) -> bool:
	frames += 1
	frames_on_screen += 1
	if frames_on_screen >= 8:
		if screen_i >= 0:
			_verify_current()
		screen_i += 1
		frames_on_screen = 0
		if screen_i < screen_ids.size():
			var id: String = screen_ids[screen_i]
			var before: int = assets.resolved_count()
			screens.show_screen(id)
			resolved_per_screen[id] = before
		else:
			return _finish()
	return false

## Step the city until the Stash's report shows `n` of `what` landed (or 1200 ticks); returns the ticks taken.
func _ride(st: RoseStash, what: String, n: int) -> int:
	for t in 1200:
		sim.ensure().step()
		if int(st.landed().get(what, 0)) >= n:
			return t + 1
	return 1200

func _verify_current() -> void:
	var id: String = screen_ids[screen_i]
	var scr: Node = screens.current
	_check(scr != null and scr.has_method("_build_content"), "%s: did not instantiate as a ScreenBase" % id)
	var slots := _count_slots(scr)
	_check(slots > 0, "%s: no AssetSlot resolved a texture" % id)
	match id:
		"colony":
			var s: Dictionary = sim.ensure().stats()
			_check(int(s.get("tick", 0)) > 500, "colony: sim did not advance (tick %d)" % int(s.get("tick", 0)))
			_check(int(s.get("cells", 0)) > 500, "colony: Mat did not grow")
			var wv: Node = null
			for ch in scr.get_children():
				if ch.has_method("full_paint"):
					wv = ch
			_check(wv != null and wv.substrate.get_used_cells().size() > 1000, "colony: world tiles not painted")
			_check(wv != null and wv.matlayer.get_used_cells().size() > 300, "colony: Mat tiles not painted")
			# the city founded on the Mat: the Heart, the Route Board, a Bench, a Pit per Relay; 24 defs
			var rs: RoseSim = sim.ensure()
			var defs: Array = rs.structure_defs()
			_check(defs.size() == 24, "colony: %d structure defs" % defs.size())
			var founded: Array = rs.structures()
			_check(founded.size() >= 4 and String(founded[0].def) == "heart" and String(founded[1].def) == "route_board" and String(founded[2].def) == "bench", "colony: founding %s" % str(founded.map(func(x): return x.def)))
			# the build tool: a Lantern Post ghost on the first tile near the Heart that takes it; two cheap Parts in
			# the Stash pay for it (the cheapest go first), so the Bench's housing stays
			var st0: RoseStash = sim.ensure_stash()
			st0.add(RoseItems.new().find("prt_wire"), 50, false)
			st0.add(RoseItems.new().find("prt_cloth"), 50, false)
			sim.build_tool = "lantern"
			scr.tool = "build"
			var hx := int(founded[0].x) + 1
			var hy := int(founded[0].y) + 1
			smoke_ghost = -1
			for r in range(2, 12):
				if smoke_ghost >= 0:
					break
				for dy in range(-r, r + 1):
					if smoke_ghost >= 0:
						break
					for dx in range(-r, r + 1):
						if absi(dx) != r and absi(dy) != r:
							continue
						smoke_ghost = scr.place_ghost(hx + dx, hy + dy)
						if smoke_ghost >= 0:
							break
			_check(smoke_ghost >= 0, "colony: no tile near the Heart takes a lantern (%s)" % scr.cursor_reason)
			_check(String(rs.can_place("vault", hx, hy)) != "", "colony: a Vault should refuse without the Heart at tier 2")
			_check(rs.structures().size() == founded.size() + 1 and not bool(rs.structures()[-1].built), "colony: the ghost is not a ghost")
			sim.build_tool = ""
			# the city has no phantoms (Full Design v3.11): the movers are the roster, one each
			var ids: PackedInt32Array = rs.agent_ids()
			_check(ids.size() == sim.ensure_people().count() and int(rs.stats().agents) == ids.size(), "colony: %d movers for a roster of %d" % [ids.size(), sim.ensure_people().count()])
			# SELECT on the Heart opens its quota card; a quota set stands as yours, DEFAULT restores the default
			scr.tool = "select"
			scr.apply_tool(hx, hy)
			_check(scr.quota_panel.visible and scr.quota_id == int(founded[0].id), "colony: selecting the Heart did not open its quota")
			_check(rs.set_quota(scr.quota_id, 1.5, 1) and is_equal_approx(float(rs.quota(scr.quota_id).target), 1.5) and String(rs.quota(scr.quota_id).from) == "yours", "colony: quota %s" % str(rs.quota(scr.quota_id)))
			_check(rs.clear_quota(scr.quota_id) and String(rs.quota(scr.quota_id).from) != "yours", "colony: the default did not come back")
			_check(int(rs.consumer_at(int(founded[1].x), int(founded[1].y))) == -1, "colony: the Route Board is not a consumer")
			# bait costs a Common Chrome from the Stash: the starting haul has none, so it says so and the Mat keeps
			scr.tool = "bait"
			scr.apply_tool(hx + 3, hy)
			_check(scr.cursor_reason.contains("Common Chrome"), "colony: bait without Common Chrome said '%s'" % scr.cursor_reason)
			# the Stack view is the core's circuit board, one pixel a tile
			scr.tool = "select"
			scr.set_view(2)
			_check(rs.stack_image().size() == rs.width() * rs.height() * 3 and scr.world.traffic_sprite.visible and not scr.world.matlayer.visible, "colony: the Stack view did not draw the core's image")
		"stash":
			var st: RoseStash = scr.stash
			_check(st.count() >= 14, "stash: starting haul not placed (%d)" % st.count())
			_check(st.save_round_trip(), "stash: save round trip failed")
			# the four verbs through the ledger
			var chrome := -1
			var chrome2 := -1
			var part := -1
			for it in st.items():
				if String(it.category) == "chrome" and chrome < 0:
					chrome = int(it.handle)
				elif String(it.category) == "chrome" and chrome2 < 0:
					chrome2 = int(it.handle)
				elif String(it.category) == "part" and part < 0 and int(it.sell) > 0:
					part = int(it.handle)
			_check(chrome >= 0 and chrome2 >= 0 and part >= 0, "stash: starting haul lacks chrome and parts")
			var markets: PackedInt32Array = sim.markets()
			_check(markets.size() > 0, "stash: the Mat reached no grate, nothing can be sold (markets %d)" % markets.size())
			var before: int = st.count()
			var quote: int = st.quote(part, 0)
			_check(quote > 0, "stash: a part quotes %d" % quote)
			_check(st.queue_sell(part, 0, sim.day) and st.count() == before - 1 and st.queue_count() == 1, "stash: sell did not queue")
			_check(not st.queue_feed(part, 0, sim.day), "stash: a part should not be feedable")
			_check(st.queue_feed(chrome, 0, sim.day) and st.queue_count() == 2, "stash: feed did not queue chrome")
			var back: int = st.unqueue(0)
			_check(back >= 0 and st.queue_count() == 1 and st.count() == before - 1, "stash: pull back failed (%d)" % back)
			# Keep pins; bulk skips pinned
			st.pin(chrome2, true)
			var extra: int = st.add(RoseItems.new().find("chr_optic_slag"), 80, true)
			var bulk: PackedInt32Array = st.bulk_handles("chrome", "", 101, false)
			_check(bulk.has(extra) and not bulk.has(chrome2), "stash: bulk view %s should hold %d and skip the pinned chrome %d" % [str(bulk), extra, chrome2])
			var unid: PackedInt32Array = st.bulk_handles("", "", 101, true)
			_check(unid.has(extra) and unid.size() == 1, "stash: unidentified filter %s" % str(unid))
			scr._select(chrome2)
			_check(scr.card.get_child_count() >= 8, "stash: item card not drawn")
			# sort keeps every item
			st.sort_by("value")
			_check(st.count() == before and not scr._item(chrome2).is_empty(), "stash: sort lost items or handles (%d)" % st.count())
			var vals := []
			for it in st.items():
				vals.append(int(it.sell))
			# shift end: the feed leaves the Heart as cargo (Full Design v3.11, Cargo) and lands when its packet does:
			# the Pit eats and the genome drifts on arrival, not at the shift end
			var genes0: PackedFloat32Array = sim.ensure().genes()
			sim.end_shift()
			var r: Dictionary = sim.shift_report
			_check(int(r.get("left", 0)) >= 1 and int(r.get("fed", 0)) == 0 and st.queue_count() == 0, "stash: the feed did not leave as cargo (%s)" % str(r))
			_check(int(sim.ensure().stats().get("riding", 0)) + int(sim.ensure().stats().get("waiting", 0)) >= 1, "stash: nothing on the Mat after the shift end")
			var ticks := _ride(st, "fed", 1)
			var landed: Dictionary = st.landed()
			_check(int(landed.fed) == 1 and landed.deliveries.size() == 1 and int(landed.deliveries[0].pit) == 0, "stash: the feed did not land at the Heart (%s after %d ticks)" % [str(landed), ticks])
			var genes1: PackedFloat32Array = sim.ensure().genes()
			var drifted := false
			for g in 10:
				if absf(genes1[g] - genes0[g]) > 1e-6:
					drifted = true
			_check(drifted, "stash: fed chrome did not drift the genome when it landed")
			# sell rides too and pays on arrival, with glut: two curios (the Route Board packs the tools later)
			var sold := 0
			for it in st.items():
				if String(it.category) in ["curio", "tech"] and not bool(it.pinned) and sold < 2:
					if st.queue_sell(int(it.handle), 0, sim.day):
						sold += 1
			_check(sold == 2, "stash: could not queue two sales (%d)" % sold)
			var e0: int = st.eddies()
			sim.end_shift()
			_check(st.eddies() == e0, "stash: a sale paid before it reached its market")
			ticks = _ride(st, "sold", 2)
			_check(st.eddies() > e0 and int(st.landed().sold) == 2, "stash: selling earned nothing on arrival (%d -> %d after %d ticks)" % [e0, st.eddies(), ticks])
			_check(st.save_round_trip(), "stash: save round trip with a ledger failed")
			_check(st.upgrade() and st.width() == 16, "stash: upgrade failed")
			# Craft hands the Bench an item
			sim.bench_slot = scr._item(chrome2)
			sim_sent_handle = chrome2
			_check(not sim.bench_slot.is_empty() and not st.best_recipe(chrome2).is_empty(), "stash: craft hand-off empty or no recipe takes a lamp core")
		"slimo":
			var pp: RosePeople = sim.ensure_people()
			_check(pp.count() == 6 and scr.ids.size() == 6, "slimo: founders %d, roster rows %d" % [pp.count(), scr.ids.size()])
			var el: PackedInt32Array = pp.eligible()
			_check(el.size() >= 3, "slimo: only %d founders can run" % el.size())
			var rec: Dictionary = pp.record(scr.selected)
			_check(not rec.is_empty() and rec.traits.size() == 2 and rec.skills.size() == 8 and int(rec.needs.mood) == 60, "slimo: record %s" % str(rec.keys()))
			var st_: Dictionary = rec.stats
			for k in ["body", "nerve", "hands", "eye", "matsense", "voice"]:
				_check(int(st_[k]) >= 1 and int(st_[k]) <= 10, "slimo: stat %s = %d" % [k, int(st_[k])])
			# a priority cell cycles and the utility follows it
			var jobs: PackedStringArray = pp.jobs()
			var scav := jobs.find("Scav")
			var u0 := float(rec.utility[scav])
			pp.set_priority(scr.selected, scav, 4)
			var u4 := float(pp.record(scr.selected).utility[scav])
			_check(u4 < 0.0 or u0 < 0.0 or u4 >= u0, "slimo: priority 4 did not raise Scav utility (%.2f -> %.2f)" % [u0, u4])
			pp.set_priority(scr.selected, scav, 0)
			_check(float(pp.record(scr.selected).utility[scav]) <= 0.0, "slimo: priority 0 should mean never")
			pp.set_priority(scr.selected, scav, 2)
			var sched: PackedStringArray = pp.record(scr.selected).schedule
			_check(Array(sched) == ["Any", "Any", "Any"], "slimo: default schedule %s" % str(sched))
			_check(String(pp.cycle_schedule(scr.selected, 1)) == "Work" and String(pp.cycle_schedule(scr.selected, 1)) == "Rest" and String(pp.cycle_schedule(scr.selected, 1)) == "Any", "slimo: schedule does not cycle")
			_check(scr.card_frame.get_child_count() > 40, "slimo: card drawn thin (%d nodes)" % scr.card_frame.get_child_count())
			smoke_runner = int(el[0])
		"charter":
			# the eight laws as cards; Authority comes from a district formed, and one signs Rationing
			var rs: RoseSim = sim.ensure()
			var laws: Array = rs.laws()
			_check(laws.size() == 8 and scr.laws_frame.get_child_count() == 8, "charter: %d laws, %d cards" % [laws.size(), scr.laws_frame.get_child_count()])
			var ds: Array = rs.districts(sim.ensure_people())
			if ds.is_empty():
				_check(int(rs.authority()) == 0 and String(rs.sign_law("rationing")) == "needs 1 Authority", "charter: signed with no Authority")
			else:
				_check(int(rs.authority()) >= 1, "charter: a district formed and no Authority (%d)" % int(rs.authority()))
				var why := String(rs.sign_law("rationing"))
				_check(why == "", "charter: Rationing would not sign (%s)" % why)
				_check(String(rs.sign_law("rationing")) == "already signed", "charter: Rationing signed twice")
				var st_ := ""
				for l in rs.laws():
					if String(l.id) == "rationing":
						st_ = String(l.state)
				_check(st_ in ["signed", "spreading"], "charter: Rationing is %s" % st_)
				_check(rs.charter_history().size() >= 1, "charter: the board has no history")
				smoke_signed = true
			_check(String(rs.sign_law("tithe")) == "only the accord offers it", "charter: Tithe outside the accord")
		"bench":
			var st: RoseStash = scr.stash
			_check(scr.recipes.size() == 60 and int(st.tier_open()) == 1, "bench: %d recipes, tier %d open" % [scr.recipes.size(), int(st.tier_open())])
			_check(scr.chosen != "" and scr.filled.has(int(sim_sent_handle)), "bench: the Stash's item was not pre-slotted (%s %s)" % [scr.chosen, str(scr.filled)])
			# a shiv: blade + housing, both common on a tier-1 recipe → base numbers; the parts leave the grid
			var blade: int = st.add(RoseItems.new().find("prt_blade_stock"), 90, false)
			var housing := -1
			for h in st.candidates("housing"):
				housing = int(h)
				break
			_check(blade >= 0 and housing >= 0, "bench: no blade or housing to craft with")
			var pv: Dictionary = st.preview("shiv", PackedInt32Array([blade, housing]), "the Bench", sim.day)
			_check(bool(pv.get("ok", false)) and String(pv.product_id) == "ger_shiv" and int(pv.stats.dmg) == int(pv.base.dmg), "bench: shiv preview %s" % str(pv))
			var wrong: Dictionary = st.preview("shiv", PackedInt32Array([housing, blade]), "the Bench", sim.day)
			_check(not bool(wrong.get("ok", true)), "bench: a housing in the blade slot should refuse")
			_check(not st.craft("monowire", PackedInt32Array([blade, housing, housing, housing]), "x", 0), "bench: a tier-3 recipe should be closed")
			var n0: int = st.count()
			_check(st.craft("shiv", PackedInt32Array([blade, housing]), "the Bench", sim.day) and st.count() == n0 - 2 and st.bench_jobs().size() == 1, "bench: craft did not queue")
			_check(st.cancel_job(0) and st.count() == n0 and st.bench_jobs().size() == 0, "bench: cancel did not return the parts")
			var blade2 := -1
			var housing2 := -1
			for h in st.candidates("blade"):
				blade2 = int(h)
				break
			for h in st.candidates("housing"):
				housing2 = int(h)
				break
			_check(st.craft("shiv", PackedInt32Array([blade2, housing2]), "Sable", sim.day), "bench: second craft failed")
			scr._redraw()
			_check(scr.queue_frame.get_child_count() >= 4, "bench: queue not drawn")
			sim.end_shift()
			var made := {}
			for it in st.items():
				if it.has("crafted_by") and String(it.id) == "ger_shiv":
					made = it
			_check(not made.is_empty() and String(made.crafted_by).begins_with("Sable") and int(made.stats.dmg) == 5, "bench: the shiv did not land in the Stash with its provenance (%s)" % str(made.keys()))
			_check(int(sim.shift_report.get("crafted", 0)) == 1, "bench: shift report crafted %s" % str(sim.shift_report))
			_check(st.save_round_trip(), "bench: save round trip with a queue failed")
		"route":
			_check(scr.routes.size() == sim.strata_open, "route: %d cards, expected %d" % [scr.routes.size(), sim.strata_open])
			for rt in scr.routes:
				var cd: Dictionary = rt.card
				_check(int(cd.chunks) >= 8 and int(cd.chunks) <= 14, "route: card S%d has %d chunks" % [int(rt.stratum), int(cd.chunks)])
				_check(int(cd.shift_s) == 480 + 70 * maxi(0, int(cd.chunks) - 8), "route: S%d shift %d s for %d chunks" % [int(rt.stratum), int(cd.shift_s), int(cd.chunks)])
				_check(float(cd.known) == 0.0 and int(cd.known_lockers) + int(cd.known_crates) == 0, "route: an unrun route claims knowledge")
			_check(scr.strip_tex.texture != null and scr.strip_tex.texture.get_width() == scr.routes[0].run.width(), "route: memory strip not drawn at one pixel per tile")
			# pack from the Stash: the pipe to the hand, the lantern and salt to quick slots, the pry bar to the pack
			var st: RoseStash = sim.ensure_stash()
			var before: int = st.count()
			for it in st.items():
				if String(it.id) in ["ger_pipe", "ger_lantern", "prt_salt", "ger_pry_bar", "org_antiseptic"]:
					scr._pack(int(it.handle))
			_check(scr.hand_handle >= 0, "route: the pipe did not go to the hand")
			_check(scr.quick_handles.count(-1) == 1, "route: quick slots %s" % str(scr.quick_handles))
			_check(scr.pack_handles.size() == 1, "route: the pry bar (1x3) should ride in the pack, pack %s" % str(scr.pack_handles))
			scr._select(0)
			# depart: the rig leaves the Stash and the launch carries it; the harness opens the run itself
			_check(scr.runner_id == smoke_runner and scr.follower_ids.size() == 2, "route: crew from the roster (runner %d, followers %s)" % [scr.runner_id, str(scr.follower_ids)])
			var pp: RosePeople = sim.ensure_people()
			var rs: Dictionary = pp.runner_setup(scr.runner_id)
			_check(int(rs.rig_rows) >= 3 and float(rs.carry_kg) >= 14.0 and scr.carry == float(rs.carry_kg), "route: runner setup %s" % str(rs))
			var rig: PackedByteArray = st.pack_rig(scr.hand_handle, PackedInt32Array(scr.quick_handles), PackedInt32Array(scr.pack_handles), int(rs.rig_rows), float(rs.carry_kg))
			_check(not rig.is_empty() and st.count() == before - 5, "route: pack_rig left the Stash at %d of %d" % [st.count(), before])
			var followers := []
			var fids := PackedInt32Array()
			for fid in scr.follower_ids:
				followers.append(pp.follower_setup(fid, scr.runner_id))
				fids.append(fid)
			pp.depart(scr.runner_id, fids)
			_check(String(pp.record(scr.runner_id).status) == "on a run" and not pp.eligible().has(scr.runner_id), "route: depart did not mark the crew out")
			sim.launch = {"stratum": 1, "seed": sim.route_seed(1), "name": sim.region_name(1), "rig": rig, "followers": followers, "runner": rs, "runner_id": scr.runner_id, "runner_name": String(rs.name)}
		"run":
			var r: RoseRun = scr.route
			_check(scr.seed_value == sim.route_seed(1) and String(r.runner().hand) == "ger_pipe", "run: the launch did not carry the route and the rig (seed %d, hand %s)" % [scr.seed_value, String(r.runner().hand)])
			_check(int(r.runner().rig_count) == 1, "run: the packed pry bar is not in the backpack (%d items)" % int(r.runner().rig_count))
			_check(int(r.shift().shift_ticks) >= 480 * 20 and scr.clock_label.text.begins_with("SHIFT"), "run: shift clock not live (%s)" % scr.clock_label.text)
			_check(scr.strip_cells.size() == r.chunk_count(), "run: route strip has %d cells for %d chunks" % [scr.strip_cells.size(), r.chunk_count()])
			_check(scr.hand_icon.texture != null and scr.quick_icons[0].texture != null, "run: hand or quick slot icon missing")
			var n: int = r.chunk_count()
			_check(n >= 8 and n <= 14, "run: route has %d chunks, expected 8..14" % n)
			_check(r.library_size() >= 14, "run: chunk library has %d chunks" % r.library_size())
			_check(r.connectivity() >= 0.95, "run: route connectivity %.3f" % r.connectivity())
			_check(scr.world.substrate.get_used_cells().size() > 2000, "run: route tiles not painted")
			_check(scr.world.props.get_used_cells().size() >= 8, "run: props not painted (%d)" % scr.world.props.get_used_cells().size())
			_check(scr.fog != null and scr.fog.texture != null, "run: fog overlay missing")
			var st0: Dictionary = r.runner()
			_check(bool(st0.get("on_extraction", false)), "run: runner did not start on the Mat edge")
			_check(String(st0.get("exposure_band", "")) == "lit", "run: lantern on should read lit (%s)" % String(st0.get("exposure_band", "")))
			# walk east two seconds, sprinting: continuous movement, footsteps, stamina spent
			var x0 := int(st0.x)
			for i in 40:
				scr.step_with(0, 1, 0)
			var st1: Dictionary = r.runner()
			_check(int(st1.x) > x0 + 256 * 4, "run: walked %d units in 2 s" % (int(st1.x) - x0))
			_check(String(st1.pose) == "crawl", "run: off the floor the Runner should crawl, pose %s" % String(st1.pose))
			# down to the floor, then sprint: faster, and it costs stamina
			for i in 40:
				if String(r.runner().pose) != "crawl":
					break
				scr.step_with(0, 0, 1)
			var floor_pose := String(r.runner().pose)
			var x1 := int(r.runner().x)
			for i in 20:
				scr.step_with(scr.B_SPRINT, 1, 0)
			var st1b: Dictionary = r.runner()
			if floor_pose == "walk":
				_check(int(st1b.x) - x1 > 256 * 5, "run: sprint moved %d units in 1 s" % (int(st1b.x) - x1))
				_check(float(st1b.stamina) < 100.0, "run: sprint cost no stamina")
			else:
				_check(floor_pose == "wade" or floor_pose == "swim", "run: unexpected pose %s under the Mat edge" % floor_pose)
			st1 = st1b
			_check(int(st1.noise_here) > 0 or r.sense_stats()[2] >= 0, "run: no footstep noise")
			# lantern off lowers exposure
			scr.lantern_pressed = true
			scr.step_with(scr.B_LANTERN, 0, 0)
			scr.lantern_pressed = false
			var st2: Dictionary = r.runner()
			_check(int(st2.exposure) < int(st1.exposure), "run: lantern off did not lower exposure (%d -> %d)" % [int(st1.exposure), int(st2.exposure)])
			scr.lantern_pressed = true
			scr.step_with(scr.B_LANTERN, 0, 0)
			scr.lantern_pressed = false
			# open the first closed container through the input word, take its contents
			var opened := false
			for src in r.sources():
				if src.state == "closed" and (src.kind == "locker" or src.kind == "crate"):
					r.debug_teleport(int(src.x), int(src.y))
					scr.step_with(0, 0, 0)
					scr.step_with(scr.B_INTERACT, 0, 0)
					opened = scr.loot_panel.visible and scr.loot_open == int(src.index)
					scr.loot_shown = scr.loot_items.size()
					scr.step_with(0, 0, 0)
					scr.step_with(scr.B_TAKE, 0, 0)
					break
			_check(opened, "run: interact did not open a container")
			_check(scr.loot_panel.visible and scr.loot_rig.get_child_count() >= 40, "run: loot panel rig grid not drawn")
			var st3: Dictionary = r.runner()
			_check(int(st3.rig_count) >= 1 or scr.loot_items.is_empty(), "run: take left the rig empty (%d items shown)" % scr.loot_items.size())
			for it in r.rig_items():
				if bool(it.found):
					_check(not bool(it.ident) and String(it.label).begins_with("unidentified"), "run: found item identified on pickup")
				else:
					_check(bool(it.ident), "run: a packed item lost its identity (%s)" % String(it.label))
			# a creature: stand next to the first one, let it come, swing the pipe until it dies
			var cs: Array = r.creatures()
			_check(cs.size() >= 1, "run: no creatures spawned")
			var crawler := {}
			for cc in cs:
				if String(cc.kind) == "crawler":
					crawler = cc
					break
			if not crawler.is_empty():
				var c: Dictionary = crawler
				var placed := false
				for d in [[1, 0], [-1, 0], [2, 0], [-2, 0]]:
					if r.debug_teleport(int(c.tx) + d[0], int(c.ty) + d[1]):
						placed = true
						break
				_check(placed, "run: could not stand beside creature %s" % str(c))
				var hits := 0
				var kills := 0
				var fled := false
				var hurt := false
				for t in 400:
					var cur := {}
					for cc in r.creatures():
						if int(cc.id) == int(c.id):
							cur = cc
					if cur.is_empty():
						break
					if String(cur.phase) == "flee":
						fled = true
						break
					var ddx := int(cur.tx) - int(r.runner().tx)
					var ddy := int(cur.ty) - int(r.runner().ty)
					var adjacent := absi(ddx) <= 1 and absi(ddy) <= 1
					var b: int = scr.B_ATTACK if adjacent and t % 12 == 0 else 0
					scr.step_with(b, signi(ddx) if (not adjacent or t % 12 == 0) else 0, signi(ddy) if not adjacent else 0)
					for e in scr._last_events:
						match String(e.kind):
							"creature_hit": hits += 1
							"creature_died": kills += 1
							"runner_hit": hurt = true
				_check(hits >= 1, "run: no swing landed on the crawler")
				_check(kills >= 1 or fled, "run: crawler neither died nor fled in 400 ticks (hits %d)" % hits)
				_check(String(r.runner().life) == "up", "run: a single crawler put a healthy Runner down (hurt %s)" % str(hurt))
			# extraction: back on the Mat edge, hold E ten seconds
			var fl: Array = r.followers()
			_check(fl.size() == 2, "run: expected two followers, got %d" % fl.size())
			scr.step_with(2 << scr.ORDER_SHIFT, 0, 0)
			_check(String(r.followers()[0].order) == "hold", "run: hold order not taken (%s)" % String(r.followers()[0].order))
			scr.step_with(0, 0, 0)
			scr.step_with(1 << scr.ORDER_SHIFT, 0, 0)
			_check(String(r.followers()[0].order) == "follow", "run: follow order not taken")
			var e0: PackedInt32Array = r.entry()
			r.debug_teleport(e0[1] % r.width(), e0[1] / r.width())
			for t in 40:
				scr.step_with(0, 0, 0)
			for t in 210:
				scr.step_with(scr.B_INTERACT, 0, 0)
			_check(r.result() == "extracted", "run: extraction did not complete (%s, action %s)" % [r.result(), String(r.runner().action)])
			var oc: Dictionary = r.outcome()
			_check(String(oc.get("result", "")) == "extracted" and int(oc.get("chunks_seen", 0)) >= 1, "run: outcome record missing or empty (%s)" % str(oc.keys()))
			_check(PackedInt32Array(oc.get("followers_extracted", PackedInt32Array())).size() + PackedInt32Array(oc.get("followers_stranded", PackedInt32Array())).size() == 2, "run: outcome lost a follower")
			_check(oc.has("report") and int(oc.report.pulls) >= 1, "run: outcome carries no cadence report")
			_check(sim.day >= 2 and not sim.last_outcome.is_empty(), "run: the colony did not take the outcome")
			# the roster took the run: the Runner is home with a run on the record and Scav XP from the pulls
			var pp: RosePeople = sim.ensure_people()
			var rr: Dictionary = pp.record(smoke_runner)
			_check(not rr.is_empty() and int(rr.runs) == 1 and int(rr.survived) == 1 and String(rr.status) == "idle", "run: the Runner's record did not take the run (%s)" % str(rr.get("status", "")))
			_check(int(rr.hauls) == int(oc.pulls) and int(rr.skills[0].xp) + int(rr.skills[0].level) > 0, "run: hauls %d vs pulls %d, Scav xp %d" % [int(rr.hauls), int(oc.pulls), int(rr.skills[0].xp)])
			# four shifts have ended; the Mat under the Heart feeds whoever stands there (nutrient at the Heart is
			# never short in this seed), Rest falls 12 a shift worked and rises 15 asleep on the Mat
			_check(int(rr.needs.fed) == 100, "run: the Mat did not feed the Runner (fed %d)" % int(rr.needs.fed))
			# Rest: the three shifts before the run were Dim (day 1, the default row works it) with more slots
			# than slimos, so the Runner worked each (-12), then the run's -12
			_check(int(rr.needs.rest) == 52, "run: Rest %d after three worked shifts and a run" % int(rr.needs.rest))
			for a in sim.last_shift.get("assignments", []):
				_check(int(a.slimo) != smoke_runner, "run: the Runner was given a job on the shift they ran")
			# the colony's shift: the founding Bench's slot was worked, the lantern ghost paid for (2 Parts) and built
			var rs: RoseSim = sim.ensure()
			var last: Dictionary = sim.last_shift
			_check(not last.is_empty() and last.has("assignments") and last.assignments.size() > 0, "run: the colony handed out no jobs")
			var lantern := {}
			for st in rs.structures():
				if int(st.id) == smoke_ghost:
					lantern = st
			_check(not lantern.is_empty() and bool(lantern.built) and bool(lantern.paid), "run: the lantern ghost did not get built (%s)" % str(lantern))
			# the first district: the Heart's node with the Board, the Bench and the lantern around it, a foreman elected
			var ds: Array = rs.districts(pp)
			_check(ds.size() == 1 and int(ds[0].foreman) >= 0 and ds[0].structures.size() >= 3, "run: the Heart's district did not form (%s)" % str(ds))
			var fm: Dictionary = pp.record(int(ds[0].foreman))
			_check(not fm.is_empty() and ds[0].members.has(int(ds[0].foreman)), "run: the foreman is not a member")
			# the Charter: the law signed before the run is in force in the Heart's district by now
			if smoke_signed:
				var arrived := false
				for l in rs.laws():
					if String(l.id) == "rationing":
						var reached: PackedStringArray = l.get("reached", PackedStringArray())
						arrived = reached.has(String(ds[0].name))
				_check(arrived, "run: Rationing never reached %s" % String(ds[0].name))
			else:
				_check(int(rs.authority()) >= 1, "run: the district formed and paid no Authority")
			for b in rr.bonds:
				_check(int(b.runs) == 1, "run: bond runs %d after one run together" % int(b.runs))
			_check(rr.bonds.size() == 2, "run: %d bonds counted for two followers" % rr.bonds.size())
			_check(scr.cadence_path != "" and FileAccess.file_exists(scr.cadence_path), "run: cadence log not written (%s)" % scr.cadence_path)
			var log_text := FileAccess.get_file_as_string(scr.cadence_path)
			_check(log_text.contains("LAND") and log_text.contains("budget"), "run: cadence log lacks entries or the report")
			_check(not sim.pending_haul.is_empty(), "run: rig not handed to the Stash")
			var probe := RoseStash.new()
			var left: int = probe.import_rig(sim.pending_haul)
			var kit_items := 0
			for q in r.quick_slots():
				if q != "":
					kit_items += 1
			var hand_items := 1 if String(r.runner().hand) != "" else 0
			var expected: int = int(r.runner().rig_count) + kit_items + hand_items - left
			_check(left >= 0 and probe.count() == expected, "run: stash import %d, expected %d (rig %d + kit %d + hand %d - left %d)" % [probe.count(), expected, int(r.runner().rig_count), kit_items, hand_items, left])
			scr.over = ""
	print("[smoke] %-9s ok  slots %d" % [id, slots])

func _has_kind(arr: Array, kind: String) -> bool:
	for e in arr:
		if e.kind == kind:
			return true
	return false

func _count_slots(n: Node) -> int:
	var c := 0
	if n is TextureRect and n.get("slot_id") != null and (n as TextureRect).texture != null:
		c += 1
	for ch in n.get_children():
		c += _count_slots(ch)
	return c

func _finish() -> bool:
	var missing: Array = assets.missing()
	_check(missing.is_empty(), "missing assets: %s" % str(missing))
	_check(screen_ids.size() == 14, "expected 14 screens, got %d" % screen_ids.size())
	var elapsed := Time.get_ticks_msec() - t_start
	print("[smoke] %d screens, %d textures resolved, %d frames in %d ms" % [screen_ids.size(), assets.resolved_count(), frames, elapsed])
	if failures.is_empty():
		print("[smoke] PASS")
		quit(0)
	else:
		for f in failures:
			printerr("[smoke] FAIL: ", f)
		quit(1)
	return true
