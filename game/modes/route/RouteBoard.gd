extends ScreenBase
## The Route Board (Full Design v3.3): one screen between the Heart and a run. Three route cards from the real
## assembled routes (run::board::RouteCard through RoseRun.card), the memory strip of the selected route, the crew,
## the rig packed from the Stash itself, DEPART. Every number on a card comes from Rust; this screen draws it.

const CARD_W := 206
const CARD_H := 96
const SLOT := 36
const DANGER_COLOR := {"low": UiTheme.DUST, "medium": UiTheme.ACID, "high": UiTheme.HOT_PINK}
var items: RoseItems
var stash: RoseStash
var people: RosePeople
var eligible: Array[int] = []       # roster ids who may run, Runners first
var runner_id := -1
var follower_ids: Array[int] = []    # up to two
var crew_frame: Panel
var carry := 18.0
var routes: Array = []            # [{stratum, seed, name, run: RoseRun, card: Dictionary, memory}]
var selected := 0
var card_frames: Array[Panel] = []
var strip_frame: Panel
var strip_tex: TextureRect
var strip_label: Label
var strip_note: Label
var rig_frame: Panel
var rig_row: Control
var hand_handle := -1
var quick_handles: Array[int] = [-1, -1, -1, -1]
var pack_handles: Array[int] = []
var weight_label: Label
var rig_title: Label
var depart_note: Label
var depart_button: Button
var hand_label: Label

func _ready() -> void:
	screen_title = "ROUTE BOARD"
	super._ready()

func _build_content(c: Control) -> void:
	items = RoseItems.new()
	stash = Sim.ensure_stash()
	people = Sim.ensure_people()
	for id in people.eligible():
		eligible.append(int(id))
	if not eligible.is_empty():
		runner_id = eligible[0]
		# SEND ON A RUN from the Slimo card names the Runner
		var sent := int(Sim.launch.get("runner_id", -1))
		if eligible.has(sent):
			runner_id = sent
		for id in eligible:
			if id != runner_id and follower_ids.size() < 2:
				follower_ids.append(id)
	Sim.launch = {}
	_assemble_routes()
	# ---- the cards
	for i in routes.size():
		var r: Dictionary = routes[i]
		var f := make_frame(c, Vector2(i * (CARD_W + 7), 0), Vector2(CARD_W, CARD_H))
		card_frames.append(f)
		_fill_card(f, r, i)
	# ---- the memory strip
	strip_frame = make_frame(c, Vector2(0, CARD_H + 6), Vector2(632, 32))
	strip_tex = TextureRect.new()
	strip_tex.position = Vector2(6, 4)
	strip_tex.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	strip_tex.stretch_mode = TextureRect.STRETCH_KEEP
	strip_frame.add_child(strip_tex)
	strip_label = make_label(strip_frame, "", Vector2(462, 4), UiTheme.CYAN)
	strip_note = make_label(strip_frame, "the route as remembered", Vector2(462, 16), UiTheme.DUST)
	# ---- the crew: the roster's eligible slimos (D1-D3, up, not out); the Runner and up to two followers
	crew_frame = make_frame(c, Vector2(0, CARD_H + 44), Vector2(300, 166))
	_draw_crew()
	# ---- the rig
	rig_frame = make_frame(c, Vector2(306, CARD_H + 44), Vector2(326, 166))
	rig_title = make_label(rig_frame, "RIG", Vector2(6, 4), UiTheme.CYAN)
	weight_label = make_label(rig_frame, "", Vector2(226, 4), UiTheme.CYAN)
	rig_row = Control.new()
	rig_frame.add_child(rig_row)
	depart_note = make_label(rig_frame, "", Vector2(6, 150), UiTheme.DUST)
	depart_button = make_button(rig_frame, "DEPART", Vector2(258, 134), _depart)
	_redraw_rig()
	_select(0)

## One RoseRun per open stratum — the three from one below the city's to one above (Full Design v3.19) —
## assembled from the save's route seed: the cards read these.
func _assemble_routes() -> void:
	routes.clear()
	for stratum in Sim.board_strata():
		var seed_v: int = Sim.route_seed(stratum)
		var run := RoseRun.new()
		if run.assemble(seed_v, stratum, 0) == 0:
			push_error("RouteBoard: S%d route %d did not assemble" % [stratum, seed_v])
			continue
		var mem: PackedByteArray = Sim.memory_of(stratum, seed_v)
		run.set_shift(Sim.day, Sim.flood_roll(), Sim.next_flood_roll())
		routes.append({"stratum": stratum, "seed": seed_v, "name": Sim.region_name(stratum), "run": run, "card": run.card(mem), "memory": mem})

func _fill_card(f: Panel, r: Dictionary, i: int) -> void:
	var cd: Dictionary = r.card
	make_label(f, String(r.name), Vector2(6, 4), UiTheme.CYAN)
	make_label(f, "S%d" % int(r.stratum), Vector2(180, 4), UiTheme.CHALK)
	make_label(f, str(i + 1), Vector2(196, 4), UiTheme.DUST)
	var shift_s := int(cd.shift_s)
	make_label(f, "%d CHUNKS   SHIFT %d:%02d" % [int(cd.chunks), shift_s / 60, shift_s % 60], Vector2(6, 16), UiTheme.CHALK)
	var band := String(cd.danger_band)
	var dc: Color = DANGER_COLOR.get(band, UiTheme.DUST)
	make_label(f, "DANGER %s" % band.to_upper(), Vector2(6, 28), dc)
	make_label(f, "%d" % int(round(float(cd.danger))), Vector2(72, 28), UiTheme.CYAN)
	var pips := clampi(int(ceil(float(cd.density) / 0.7)), 1, 5)
	for k in 5:
		var pip := ColorRect.new()
		pip.position = Vector2(90 + k * 8, 30)
		pip.size = Vector2(6, 5)
		pip.color = dc if k < pips else UiTheme.SUMP
		f.add_child(pip)
	var known := float(cd.known)
	if known <= 0.0:
		make_label(f, "UNEXPLORED", Vector2(6, 40), UiTheme.DUST)
	else:
		make_label(f, "KNOWN %d%%   LOCKERS %d   CRATES %d" % [int(round(known * 100.0)), int(cd.known_lockers), int(cd.known_crates)], Vector2(6, 40), UiTheme.CHALK)
	var water_line := "WATER %d%%   GRATES %d" % [int(round(float(cd.water) * 100.0)), int(cd.grates)]
	make_label(f, water_line, Vector2(6, 52), UiTheme.DUST)
	# a Flood shift: how far the water rises and when (Full Design, the Threats table)
	var rise := int(cd.get("flood_rise", -1))
	if rise > 0:
		var fs := int(cd.get("flood_s", 0))
		var when := "IN OVERTIME" if fs >= shift_s else "AT %d:%02d" % [fs / 60, fs % 60]
		make_label(f, "FLOOD +%d %s" % [rise, when], Vector2(6 + (water_line.length() + 3) * 4, 52), UiTheme.CYAN)
	elif rise == 0:
		make_label(f, "FLOOD HOLDS", Vector2(6 + (water_line.length() + 3) * 4, 52), UiTheme.DUST)
	var bodies: Array = Sim.corpses_on(int(r.stratum), int(r.seed))
	if bodies.size() > 0:
		var names := []
		for b in bodies:
			names.append("%s d%d%s" % [String(b.runner), int(b.day), " raided" if bool(b.raided) else ""])
		make_label(f, ("BODY: " if bodies.size() == 1 else "BODIES: ") + ", ".join(names), Vector2(6, 64), UiTheme.HOT_PINK)
	var stranded: Array = Sim.stranded.get(Sim.route_key(int(r.stratum), int(r.seed)), [])
	if stranded.size() > 0:
		var sn := []
		for id in stranded:
			sn.append(_crew_name(int(id)))
		make_label(f, "STRANDED: " + ", ".join(sn), Vector2(6, 76), UiTheme.CHALK)
	var pick := Button.new()
	pick.flat = true
	pick.focus_mode = Control.FOCUS_NONE
	pick.position = Vector2.ZERO
	pick.size = Vector2(CARD_W, CARD_H)
	pick.pressed.connect(func(): _select(i))
	f.add_child(pick)
	# route loops (Full Design v3.18, Route loops): a route come home from twice runs every shift with one slimo
	var lp: Dictionary = Sim.ensure().route_loops()
	if bool(lp.researched):
		var st := int(r.stratum)
		var homes := int((lp.homes as Dictionary).get(st, 0))
		var mine := {}
		for l in lp.loops:
			if int(l.stratum) == st:
				mine = l
		if not mine.is_empty():
			make_label(f, "LOOP: %s, %d REPEATS" % [_crew_name(int(mine.slimo)).to_upper(), int(mine.repeats)], Vector2(6, 84), UiTheme.OOZE_GLOW)
			_card_word(f, "END", Vector2(180, 84), UiTheme.HOT_PINK, func():
				Sim.ensure().set_route_loop(Sim.ensure_people(), st, -1)
				_refresh_cards())
		elif homes >= int(lp.after) and runner_id >= 0:
			_card_word(f, "LOOP IT WITH %s" % _crew_name(runner_id).to_upper(), Vector2(6, 84), UiTheme.ACID, func():
				var why: String = Sim.ensure().set_route_loop(Sim.ensure_people(), st, runner_id)
				depart_note.text = why
				_refresh_cards())
		else:
			make_label(f, "LOOP: %d OF %d RUNS HOME" % [homes, int(lp.after)], Vector2(6, 84), UiTheme.DUST)

## A clickable word on a card, over the card's own pick.
func _card_word(f: Control, text: String, pos: Vector2, color: Color, on_press: Callable) -> void:
	make_label(f, text, pos, color)
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.position = pos
	b.size = Vector2(text.length() * 4 + 4, 10)
	b.pressed.connect(on_press)
	f.add_child(b)

func _refresh_cards() -> void:
	for i in card_frames.size():
		for ch in card_frames[i].get_children():
			ch.queue_free()
		_fill_card(card_frames[i], routes[i], i)

func _crew_name(id: int) -> String:
	return Sim.slimo_name(id)

## The crew panel: every eligible slimo on a row with its run numbers; click a row to make it the Runner, click
## COMES ALONG / STAYS HOME to take or leave a follower (two at most).
func _draw_crew() -> void:
	for ch in crew_frame.get_children():
		ch.queue_free()
	make_label(crew_frame, "CREW   %d can run" % eligible.size(), Vector2(6, 4), UiTheme.CYAN)
	if eligible.is_empty():
		make_label(crew_frame, "nobody can run: everyone is out, down or deep", Vector2(6, 20), UiTheme.HOT_PINK)
		return
	var y := 16
	var shown := 0
	for id in eligible:
		if shown >= 4:
			make_label(crew_frame, "+%d more on the Slimo card" % (eligible.size() - 4), Vector2(6, y), UiTheme.DUST)
			break
		var rec: Dictionary = people.record(id)
		if rec.is_empty():
			continue
		var is_runner := id == runner_id
		var along := follower_ids.has(id)
		var row := Control.new()
		row.position = Vector2(0, y)
		crew_frame.add_child(row)
		make_slot(row, "slimo.%s.idle" % String(rec.depth).to_lower(), Vector2(6, 0), 20, 30, 3.0)
		var st: Dictionary = rec.stats
		var total := int((20 + 50 + 30 + 30) * (0.8 + 0.04 * int(st.body)))
		var name_col: Color = UiTheme.CYAN if is_runner else (UiTheme.CHALK if along else UiTheme.DUST)
		make_label(row, "%s   %s %s" % [String(rec.full_name).to_upper(), String(rec.depth), String(rec.past)], Vector2(36, 1), name_col)
		var line2 := "Body %d  Nerve %d  Hands %d  Eye %d   HP %d" % [int(st.body), int(st.nerve), int(st.hands), int(st.eye), total if is_runner else int(total * 0.6)]
		make_label(row, line2, Vector2(36, 11), UiTheme.CHALK if is_runner or along else UiTheme.DUST)
		var skills: Array = rec.skills
		var sk := "Scav %d  Fight %d" % [int(skills[0].level), int(skills[1].level)]
		var traits: Array = rec.traits
		if traits.size() == 2:
			sk += "   %s, %s" % [String(traits[0].name), String(traits[1].name)]
		make_label(row, sk, Vector2(36, 21), UiTheme.DUST)
		if is_runner:
			make_label(row, "RUNNER", Vector2(232, 1), UiTheme.CYAN)
			hand_label = make_label(row, "", Vector2(232, 11), UiTheme.DUST)
		else:
			var pick := Button.new()
			pick.flat = true
			pick.focus_mode = Control.FOCUS_NONE
			pick.size = Vector2(220, 30)
			var pid := id
			pick.pressed.connect(func(): _make_runner(pid))
			row.add_child(pick)
			var fid := id
			var b := make_button(row, "COMES ALONG" if along else "STAYS HOME", Vector2(216, 7), func(): _toggle_follower(fid))
			b.modulate = Color.WHITE if along else Color(0.6, 0.6, 0.65)
		y += 32
		shown += 1
	make_label(crew_frame, "click a name to make them the Runner", Vector2(6, 155), UiTheme.DUST)
	make_button(crew_frame, "CARDS", Vector2(246, 148), func(): Screens.show_screen("slimo"))

func _make_runner(id: int) -> void:
	if id == runner_id:
		return
	follower_ids.erase(id)
	if runner_id >= 0 and follower_ids.size() < 2:
		follower_ids.append(runner_id)
	runner_id = id
	_draw_crew()
	_redraw_rig()
	_refresh_cards()
	_select(selected)

func _select(i: int) -> void:
	if routes.is_empty():
		return
	selected = clampi(i, 0, routes.size() - 1)
	for k in card_frames.size():
		card_frames[k].modulate = Color.WHITE if k == selected else Color(0.72, 0.72, 0.78)
	_draw_strip()
	_redraw_rig()

## The selected route's remembered tiles at one pixel per tile: Mat edge Ooze Glow, grates Acid, known containers
## Cyan, corpses Hot Pink, unseen Void.
func _draw_strip() -> void:
	var r: Dictionary = routes[selected]
	var run: RoseRun = r.run
	var w: int = run.width()
	var h: int = run.height()
	var img := Image.create(w, h, false, Image.FORMAT_RGBA8)
	img.fill(UiTheme.VOID)
	# chunk seams as faint ticks, so the strip shows the route's length the card already states
	for cx in range(0, w, 32):
		for y in h:
			img.set_pixel(cx, y, Color8(18, 20, 30))
	var mem: PackedByteArray = r.memory
	var sub: PackedByteArray = run.substrate()
	var pass_: PackedByteArray = run.passable()
	var known := 0
	for i in w * h:
		var seen := i / 8 < mem.size() and (mem[i / 8] & (1 << (i % 8))) != 0
		if not seen:
			continue
		known += 1
		var x := i % w
		var y := i / w
		if pass_[i] != 0:
			# water reads blue-grey, passable sewer Dust; solids a shade above Void so walls shape the memory
			img.set_pixel(x, y, Color8(60, 84, 110) if sub[i] == 5 else Color8(90, 86, 96))
		else:
			img.set_pixel(x, y, Color8(40, 38, 48))
	for i in run.entry():
		img.set_pixel(i % w, i / w, UiTheme.OOZE_GLOW)
	for i in run.grates():
		if i / 8 < mem.size() and (mem[i / 8] & (1 << (i % 8))) != 0:
			img.set_pixel(i % w, i / w, UiTheme.ACID)
	for cn in run.containers():
		var idx := int(cn.y) * w + int(cn.x)
		if idx / 8 < mem.size() and (mem[idx / 8] & (1 << (idx % 8))) != 0:
			img.set_pixel(int(cn.x), int(cn.y), UiTheme.CYAN)
	for b in Sim.corpses_on(int(r.stratum), int(r.seed)):
		img.set_pixel(int(b.x), int(b.y), UiTheme.HOT_PINK)
	strip_tex.texture = ImageTexture.create_from_image(img)
	strip_label.text = "%s   MEMORY %d%%" % [String(r.name), int(round(float(r.card.known) * 100.0))]
	strip_note.text = "the route as remembered" if known > 0 else "never run: only the Mat edge is known"

func _toggle_follower(id: int) -> void:
	if follower_ids.has(id):
		follower_ids.erase(id)
	elif follower_ids.size() < 2:
		follower_ids.append(id)
	_draw_crew()

# ---------------------------------------------------------------- the rig, packed from the Stash
func _stash_item(handle: int) -> Dictionary:
	for it in stash.items():
		if int(it.handle) == handle:
			return it
	return {}

func _packed() -> Array[int]:
	var out: Array[int] = []
	if hand_handle >= 0:
		out.append(hand_handle)
	for h in quick_handles:
		if h >= 0:
			out.append(h)
	out.append_array(pack_handles)
	return out

func _weight() -> float:
	var w := 0.0
	for h in _packed():
		var it := _stash_item(h)
		if not it.is_empty():
			w += float(items.get(int(it.index)).weight)
	return w

## Click a Stash item: melee gear to the hand, one- or two-cell items to a free quick slot, the rest to the pack.
func _pack(handle: int) -> void:
	var it := _stash_item(handle)
	if it.is_empty():
		return
	var d: Dictionary = items.get(int(it.index))
	if String(d.get("gear_kind", "")) == "melee" or String(d.get("gear_kind", "")) == "ranged":
		if hand_handle >= 0:
			pack_handles.append(hand_handle)
		hand_handle = handle
	elif int(d.w) * int(d.h) <= 2 and quick_handles.has(-1):
		quick_handles[quick_handles.find(-1)] = handle
	else:
		pack_handles.append(handle)
	_redraw_rig()

func _unpack(handle: int) -> void:
	if hand_handle == handle:
		hand_handle = -1
	for i in 4:
		if quick_handles[i] == handle:
			quick_handles[i] = -1
	pack_handles.erase(handle)
	_redraw_rig()

func _slot_frame(parent: Control, pos: Vector2, handle: int, key: String, on_click: Callable) -> void:
	var f := make_frame(parent, pos, Vector2(SLOT, SLOT))
	if handle >= 0:
		var it := _stash_item(handle)
		if not it.is_empty():
			var icon := make_slot(f, icon_for(String(it.id)), Vector2(2, 2))
			icon.modulate = Color.WHITE
			var tier := ColorRect.new()
			tier.position = Vector2(2, SLOT - 5)
			tier.size = Vector2(SLOT - 4, 2)
			tier.color = TIER_COLOR.get(String(it.tier), UiTheme.DUST)
			f.add_child(tier)
	if key != "":
		make_label(f, key, Vector2(2, -1), UiTheme.DUST)
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.size = Vector2(SLOT, SLOT)
	b.pressed.connect(on_click)
	f.add_child(b)

func _runner_setup() -> Dictionary:
	var rs: Dictionary = people.runner_setup(runner_id) if runner_id >= 0 else RosePeople.plain_runner(5)
	# the city's flags (Full Design v3.18, Scav yield): the curio sets, Conscription's cost during an alarm
	var flags: PackedStringArray = rs.get("flags", PackedStringArray())
	flags.append_array(Sim.ensure().run_flags())
	rs["flags"] = flags
	return rs

func _redraw_rig() -> void:
	for ch in rig_row.get_children():
		ch.queue_free()
	var rs := _runner_setup()
	carry = float(rs.get("carry_kg", 18.0))
	rig_title.text = "RIG   8x%d backpack   4 quick" % int(rs.get("rig_rows", 5))
	# the hand, the quick slots, the pack
	make_label(rig_row, "HAND", Vector2(6, 16), UiTheme.DUST)
	var hh := hand_handle
	_slot_frame(rig_row, Vector2(6, 26), hh, "J", func(): _unpack(hh))
	make_label(rig_row, "QUICK 1-4", Vector2(50, 16), UiTheme.DUST)
	for i in 4:
		var qh: int = quick_handles[i]
		_slot_frame(rig_row, Vector2(50 + i * 40, 26), qh, str(i + 1), func(): _unpack(qh))
	make_label(rig_row, "PACK", Vector2(214, 16), UiTheme.DUST)
	for i in 2:
		var ph: int = pack_handles[i] if i < pack_handles.size() else -1
		_slot_frame(rig_row, Vector2(214 + i * 40, 26), ph, "", func(): _unpack(ph))
	if pack_handles.size() > 2:
		make_label(rig_row, "+%d" % (pack_handles.size() - 2), Vector2(296, 36), UiTheme.CHALK)
	# the Stash: what can still be packed
	make_label(rig_row, "FROM THE STASH   click to pack", Vector2(6, 62), UiTheme.DUST)
	var packed := _packed()
	var k := 0
	var more := 0
	for it in stash.items():
		var h := int(it.handle)
		if packed.has(h):
			continue
		if k >= 14:
			more += 1
			continue
		_slot_frame(rig_row, Vector2(6 + (k % 7) * 40, 71 + (k / 7) * 38), h, "", func(): _pack(h))
		k += 1
	if more > 0:
		make_label(rig_row, "+%d more in the Stash" % more, Vector2(300, 108), UiTheme.DUST)
	var w := _weight()
	if hand_label != null:
		var hit := _stash_item(hand_handle) if hand_handle >= 0 else {}
		hand_label.text = "in hand: " + (String(hit.name) if not hit.is_empty() else "fists")
	weight_label.text = "%.1f / %.0f kg" % [w, carry]
	weight_label.add_theme_color_override("font_color", UiTheme.HOT_PINK if w > carry else UiTheme.CYAN)
	var r: Dictionary = routes[selected] if not routes.is_empty() else {}
	var note := ""
	if runner_id < 0:
		note = "nobody can run"
	elif hand_handle < 0:
		note = "no weapon in hand: fists"
	elif w > carry:
		note = "overweight: half sprint, double noise"
	elif not r.is_empty():
		note = "%s  S%d  %s danger     1-3 pick  ENTER go" % [String(r.name), int(r.stratum), String(r.card.danger_band)]
	depart_note.text = note

## Pack the rig out of the Stash, hand the Run its route, crew and rig, go.
func _depart() -> void:
	if routes.is_empty() or runner_id < 0:
		return
	var r: Dictionary = routes[selected]
	var rs := _runner_setup()
	var rig: PackedByteArray = stash.pack_rig(hand_handle, PackedInt32Array(quick_handles), PackedInt32Array(pack_handles), int(rs.rig_rows), float(rs.carry_kg))
	var followers := []
	for id in follower_ids:
		followers.append(people.follower_setup(id, runner_id))
	var fids := PackedInt32Array()
	for id in follower_ids:
		fids.append(id)
	people.depart(runner_id, fids)
	Sim.refresh_movers()   # the crew leaves the map
	Sim.launch = {"stratum": int(r.stratum), "seed": int(r.seed), "name": String(r.name), "rig": rig, "followers": followers, "runner": rs, "runner_id": runner_id, "runner_name": String(rs.name)}
	Screens.show_screen("run")

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode >= KEY_1 and event.keycode <= KEY_3:
			_select(event.keycode - KEY_1)
		elif event.keycode == KEY_ENTER:
			_depart()
