extends ScreenBase
## The Slimo card (Full Design, UI table): the roster down the left (name, depth, status, Mood), the selected record
## on the right from RosePeople.record: portrait by depth, stats as gauges, skills as ten-step bars with XP to next,
## traits with their effects in words, needs as six bars with the Mood target marked, injuries, the record line,
## bonds; job priorities 0-4 in a row of cells; SEND ON A RUN opens the Route Board with them as the Runner.

var people: RosePeople
var ids: Array[int] = []
var selected := -1
var roster_frame: Panel
var card_frame: Panel
var memorial_shown := false

func _ready() -> void:
	screen_title = "SLIMO"
	super._ready()

func _build_content(c: Control) -> void:
	people = Sim.ensure_people()
	roster_frame = make_frame(c, Vector2(0, 0), Vector2(170, 306))
	card_frame = make_frame(c, Vector2(176, 0), Vector2(456, 306))
	_reload()
	if selected < 0 and not ids.is_empty():
		selected = ids[0]
	_redraw()

func _reload() -> void:
	ids.clear()
	for id in people.ids():
		ids.append(int(id))
	if Sim.launch.has("runner_id"):
		selected = int(Sim.launch.runner_id)
	if Sim.focus_slimo >= 0:
		selected = Sim.focus_slimo
		Sim.focus_slimo = -1

func _redraw() -> void:
	_draw_roster()
	_draw_card()

func _draw_roster() -> void:
	for ch in roster_frame.get_children():
		ch.queue_free()
	make_label(roster_frame, "ROSTER   %d" % ids.size(), Vector2(6, 4), UiTheme.CYAN)
	var y := 18
	for id in ids:
		var rec: Dictionary = people.record(id)
		if rec.is_empty():
			continue
		var is_sel := id == selected
		var row := Control.new()
		row.position = Vector2(0, y)
		roster_frame.add_child(row)
		var mood := int(rec.needs.mood)
		var mood_col: Color = UiTheme.HOT_PINK if mood < 20 else (UiTheme.ACID if mood < 30 else UiTheme.CHALK)
		make_label(row, String(rec.full_name).left(22).to_upper(), Vector2(6, 0), UiTheme.CYAN if is_sel else UiTheme.CHALK)
		make_label(row, "%s  %s" % [String(rec.depth), String(rec.status)], Vector2(6, 10), UiTheme.DUST)
		make_label(row, "M%d" % mood, Vector2(140, 10), mood_col)
		var b := Button.new()
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.size = Vector2(166, 22)
		var pid := id
		b.pressed.connect(func():
			selected = pid
			memorial_shown = false
			_redraw())
		row.add_child(b)
		y += 24
		if y > 250:
			make_label(roster_frame, "+%d more" % (ids.size() - ids.find(id) - 1), Vector2(6, y), UiTheme.DUST)
			break
	var dead: Array = people.memorial()
	if dead.size() > 0:
		make_button(roster_frame, "MEMORIAL  %d" % dead.size(), Vector2(6, 282), func():
			memorial_shown = true
			_redraw())

func _bar(parent: Control, pos: Vector2, w: int, fill: float, col: Color, mark: float = -1.0) -> void:
	var back := ColorRect.new()
	back.position = pos
	back.size = Vector2(w, 5)
	back.color = UiTheme.SUMP
	parent.add_child(back)
	var f := ColorRect.new()
	f.position = pos
	f.size = Vector2(maxf(0.0, w * clampf(fill, 0.0, 1.0)), 5)
	f.color = col
	parent.add_child(f)
	if mark >= 0.0:
		var m := ColorRect.new()
		m.position = pos + Vector2(w * clampf(mark, 0.0, 1.0) - 1, -1)
		m.size = Vector2(2, 7)
		m.color = UiTheme.CHALK
		parent.add_child(m)

func _draw_card() -> void:
	for ch in card_frame.get_children():
		ch.queue_free()
	if memorial_shown:
		_draw_memorial()
		return
	var rec: Dictionary = people.record(selected)
	if rec.is_empty():
		make_label(card_frame, "nobody here", Vector2(6, 4), UiTheme.DUST)
		return
	# ---- head: portrait by depth, name, the line
	var depth := String(rec.depth).to_lower()
	make_slot(card_frame, "slimo.d5.pile" if depth == "d5" else "slimo.%s.idle" % depth, Vector2(8, 6), 32, 48, 4.0, "", 2)
	make_label(card_frame, String(rec.full_name).to_upper(), Vector2(80, 4), UiTheme.CYAN)
	var runs := int(rec.runs)
	make_label(card_frame, "%s %s   Past %s   %s   %d run%s, %d survived" % [String(rec.depth), String(rec.depth_name), String(rec.past), String(rec.status), runs, "" if runs == 1 else "s", int(rec.survived)], Vector2(80, 16), UiTheme.DUST)
	# the city's hold on them: bed, last shift's job, district and the foreman's chair (Full Design v3.9)
	var csim: RoseSim = Sim.ensure()
	var city_line := "sleeps on the Mat"
	var homes: Dictionary = csim.homes()
	if homes.has(selected):
		for st in csim.structures():
			if int(st.id) == int(homes[selected]):
				city_line = "sleeps in the %s at %d,%d" % [String(st.name), int(st.x), int(st.y)]
	for a in Sim.last_shift.get("assignments", []):
		if int(a.slimo) == selected:
			city_line += "   last shift %s at the %s" % [String(a.job).to_lower(), String(a.structure_name)]
	for d in csim.districts(people):
		var members: PackedInt32Array = d.members
		if members.has(selected):
			city_line += "   %s" % String(d.name)
			if int(d.foreman) == selected:
				city_line += " (FOREMAN)"
	make_label(card_frame, city_line.left(70), Vector2(80, 26), UiTheme.OOZE_GLOW if city_line.contains("FOREMAN") else UiTheme.DUST)
	# ---- stats as gauges
	var st: Dictionary = rec.stats
	var y := 38
	for key in ["body", "nerve", "hands", "eye", "matsense", "voice"]:
		var v := int(st[key])
		make_label(card_frame, ("MAT-SENSE" if key == "matsense" else key.to_upper()), Vector2(80, y), UiTheme.DUST)
		_bar(card_frame, Vector2(140, y + 3), 80, v / 10.0, UiTheme.OOZE_GLOW)
		make_label(card_frame, str(v), Vector2(224, y), UiTheme.CHALK)
		y += 11
	# ---- skills as ten-step bars with XP to next
	make_label(card_frame, "SKILLS", Vector2(250, 38), UiTheme.CYAN)
	y = 50
	for sk in rec.skills:
		var lv := int(sk.level)
		make_label(card_frame, String(sk.name).to_upper(), Vector2(250, y), UiTheme.DUST)
		for k in 10:
			var cell := ColorRect.new()
			cell.position = Vector2(312 + k * 8, y + 3)
			cell.size = Vector2(6, 5)
			cell.color = UiTheme.CYAN if k < lv else UiTheme.SUMP
			card_frame.add_child(cell)
		var xp_text := "%d/%d" % [int(sk.xp), int(sk.next)] if lv < 10 else String(sk.mastery).left(20)
		make_label(card_frame, xp_text, Vector2(396, y), UiTheme.CHALK if lv < 10 else UiTheme.OOZE_GLOW)
		y += 10
	# ---- traits with their effects in words
	y = 108
	make_label(card_frame, "TRAITS", Vector2(8, y), UiTheme.CYAN)
	y += 12
	for t in rec.traits:
		make_label(card_frame, String(t.get("name", t.id)).to_upper(), Vector2(8, y), UiTheme.OOZE_GLOW)
		make_label(card_frame, String(t.get("text", "")).left(58), Vector2(8, y + 10), UiTheme.DUST)
		y += 22
	# ---- needs as six bars with the Mood target marked
	make_label(card_frame, "NEEDS", Vector2(250, 134), UiTheme.CYAN)
	y = 146
	var needs: Dictionary = rec.needs
	var target := int(rec.mood_target)
	for key in ["fed", "lit", "dry", "signal", "rest", "mood"]:
		var v := int(needs[key])
		var col: Color = UiTheme.HOT_PINK if v < 20 else (UiTheme.ACID if v < 40 else UiTheme.OOZE_GLOW)
		make_label(card_frame, key.to_upper(), Vector2(250, y), UiTheme.DUST)
		_bar(card_frame, Vector2(300, y + 3), 100, v / 100.0, col, target / 100.0 if key == "mood" else -1.0)
		make_label(card_frame, str(v) + (" to %d" % target if key == "mood" else ""), Vector2(406, y), UiTheme.CHALK)
		y += 11
	var line := int(rec.unrest_line)
	if line > 0:
		make_label(card_frame, ["", "below the strike line", "below the defection line", "below the petition line"][line], Vector2(250, y), UiTheme.HOT_PINK)
	# ---- injuries, the record line, bonds (under the two traits, which end at 164)
	y = 170
	var inj: Array = rec.injuries
	if inj.size() > 0:
		var parts := []
		for i in inj:
			parts.append("%s %d" % [String(i.name), int(i.shifts)])
		make_label(card_frame, "INJURIES  " + ", ".join(parts), Vector2(8, y), UiTheme.HOT_PINK)
	else:
		make_label(card_frame, "INJURIES  none", Vector2(8, y), UiTheme.DUST)
	y += 12
	make_label(card_frame, "RECORD  %d hauls - %d kills - %d crafts - joined day %d" % [int(rec.hauls), int(rec.kills), int(rec.crafts), int(rec.joined_day)], Vector2(8, y), UiTheme.DUST)
	y += 12
	var bonds: Array = rec.bonds
	var bl := []
	for b in bonds:
		if bool(b.bonded):
			bl.append("%s (bonded)" % String(b.with_name))
		elif int(b.runs) > 0:
			bl.append("%s %d/%d" % [String(b.with_name), int(b.runs), int(b.need)])
	make_label(card_frame, "BONDS  " + (", ".join(bl) if bl.size() > 0 else "none yet: three runs together"), Vector2(8, y), UiTheme.DUST)
	# ---- job priorities 0-4 in a row of cells
	y = 222
	make_label(card_frame, "JOBS   click a cell: 0 never, 1-4 priority", Vector2(8, y), UiTheme.CYAN)
	y += 12
	var jobs: PackedStringArray = people.jobs()
	var pri: PackedInt32Array = rec.priorities
	var util: PackedFloat32Array = rec.utility
	for j in jobs.size():
		var x := 8 + j * 49
		var allowed := float(util[j]) >= 0.0
		make_label(card_frame, String(jobs[j]).to_upper().left(6), Vector2(x, y), UiTheme.CHALK if allowed else UiTheme.DUST)
		var cell := make_frame(card_frame, Vector2(x, y + 11), Vector2(40, 16))
		var p := int(pri[j])
		var ptext := "-" if not allowed else ("never" if p == 0 else "#".repeat(p))
		make_label(cell, ptext, Vector2(3, 2), UiTheme.CYAN if p > 0 and allowed else UiTheme.DUST)
		var jj := j
		var b := Button.new()
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.size = cell.size
		b.pressed.connect(func():
			people.set_priority(selected, jj, (p + 1) % 5)
			_draw_card())
		cell.add_child(b)
		if allowed:
			make_label(card_frame, "%.1f" % float(util[j]), Vector2(x + 2, y + 29), UiTheme.DUST)
	# ---- the schedule row: Dim, Dark, Flood; click to cycle Work / Rest / Any
	make_label(card_frame, "SCHEDULE", Vector2(8, 286), UiTheme.CYAN)
	var row: PackedStringArray = rec.schedule
	for k in 3:
		var x := 60 + k * 76
		make_label(card_frame, ["DIM", "DARK", "FLOOD"][k], Vector2(x, 286), UiTheme.DUST)
		var kk := k
		var b := make_button(card_frame, String(row[k]).to_upper(), Vector2(x + 26, 282), func():
			people.cycle_schedule(selected, kk)
			_draw_card())
		b.modulate = Color.WHITE if String(row[k]) != "Rest" else Color(0.6, 0.6, 0.65)
	# ---- send on a run
	if bool(rec.can_run):
		make_button(card_frame, "SEND ON A RUN", Vector2(340, 282), func():
			Sim.launch = {"runner_id": selected}
			Screens.show_screen("route"))
	else:
		make_label(card_frame, "cannot run: " + ("out already" if String(rec.status) != "idle" and String(rec.status) != "resting" and not String(rec.status).begins_with("working") else "the depth keeps them on the Mat"), Vector2(300, 270), UiTheme.DUST)
	# a D5 cannot act: the card shows its feed value and the Dissolution (Pass 2)
	if String(rec.depth) == "D5":
		var rite := false
		for l in Sim.ensure().laws():
			if String(l.id) == "dissolution_rite" and String(l.state) in ["signed", "spreading"]:
				rite = true
		make_button(card_frame, "DISSOLVE: 0.30 NUTRIENT", Vector2(300, 282), func():
			Sim.people_lines = PackedStringArray([String(Sim.ensure().dissolve(people, selected))])
			_reload()
			selected = ids[0] if not ids.is_empty() else -1
			_redraw())
		make_label(card_frame, "under the Rite" if rite else "Mood -10 city-wide", Vector2(420, 286), UiTheme.OOZE_GLOW if rite else UiTheme.HOT_PINK)

func _draw_memorial() -> void:
	make_label(card_frame, "THE MEMORIAL", Vector2(8, 4), UiTheme.CYAN)
	var y := 18
	for m in people.memorial():
		make_label(card_frame, "%s   %s %s   %d runs, %d hauls, %d kills" % [String(m.full_name).to_upper(), String(m.depth), String(m.past), int(m.runs), int(m.hauls), int(m.kills)], Vector2(8, y), UiTheme.CHALK)
		y += 12
	if y == 18:
		make_label(card_frame, "nobody yet", Vector2(8, y), UiTheme.DUST)
	make_button(card_frame, "BACK", Vector2(400, 282), func():
		memorial_shown = false
		_redraw())
