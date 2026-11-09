extends ScreenBase
## The Charter (Full Design v3.10, The Charter; Pass 2): the eight laws as cards from data/laws.toml with their
## Authority, effect and cost, their state (open, spreading, signed, repealing, locked with the reason) and where
## the wave has reached; SIGN and REPEAL. Below, the districts with their foremen, Mood, strike and defection
## lines and petitions (ANSWER repeals the law asked or arms the Build tool with the structure; REFUSE closes it,
## twice dissolves the district), and the board's history.

const CARD_W := 154
const CARD_H := 90

var sim: RoseSim
var people: RosePeople
var laws_frame: Control
var districts_frame: Panel
var head: Label
var note := ""

func _ready() -> void:
	screen_title = "CHARTER"
	super._ready()

func _build_content(c: Control) -> void:
	sim = Sim.ensure()
	people = Sim.ensure_people()
	head = make_label(c, "", Vector2(0, 0), UiTheme.CYAN)
	laws_frame = Control.new()
	laws_frame.position = Vector2(0, 12)
	c.add_child(laws_frame)
	districts_frame = make_frame(c, Vector2(0, 200), Vector2(558, 106))
	# Ivan signs: his portrait in the state his earned Authority gives him
	var ivan := make_frame(c, Vector2(562, 200), Vector2(70, 106))
	make_slot(ivan, "ivan.portrait", Vector2(3, 4), 0, 0, 0.0, "a%d" % int(sim.ivan_state()))
	make_label(ivan, "IVAN", Vector2(5, 74), UiTheme.OOZE_GLOW)
	make_label(ivan, "STATE %d OF 4" % int(sim.ivan_state()), Vector2(5, 86), UiTheme.DUST)
	_redraw()

func _redraw() -> void:
	head.text = "AUTHORITY %d   a law leaves the Heart as a signal wave: %d cords a shift%s" % [int(sim.authority()), int(sim.wave_speed()), ("   -   " + note) if note != "" else ""]
	_draw_laws()
	_draw_districts()

## Lines of at most `width` characters, broken at spaces.
func _wrap(text: String, width: int) -> Array:
	var lines := []
	var line := ""
	for word in text.split(" ", false):
		if line != "" and line.length() + 1 + word.length() > width:
			lines.append(line)
			line = word
		else:
			line = word if line == "" else line + " " + word
	if line != "":
		lines.append(line)
	return lines

func _draw_laws() -> void:
	for ch in laws_frame.get_children():
		ch.queue_free()
	var laws: Array = sim.laws()
	for i in laws.size():
		var l: Dictionary = laws[i]
		var f := make_frame(laws_frame, Vector2((i % 4) * (CARD_W + 5), (i / 4) * (CARD_H + 4)), Vector2(CARD_W, CARD_H))
		var state := String(l.state)
		var state_col: Color = {"signed": UiTheme.OOZE_GLOW, "spreading": UiTheme.ACID, "repealing": UiTheme.HOT_PINK, "open": UiTheme.CYAN, "locked": UiTheme.DUST}[state]
		f.modulate = Color.WHITE if state != "locked" else Color(0.75, 0.75, 0.8)
		make_label(f, String(l.name).to_upper(), Vector2(4, 3), UiTheme.CYAN)
		make_label(f, state.to_upper(), Vector2(CARD_W - 4 - state.length() * 4, 3), state_col)
		# effect then cost, four lines between them
		var eff := _wrap(String(l.effect), 37)
		var cost := _wrap(String(l.cost), 37)
		var y := 14
		for line in eff.slice(0, 4 - mini(cost.size(), 2)):
			make_label(f, line, Vector2(4, y), UiTheme.CHALK)
			y += 9
		for line in cost.slice(0, 2):
			make_label(f, line, Vector2(4, y), UiTheme.HOT_PINK)
			y += 9
		var price := "A %d" % int(l.authority)
		if int(l.mood) != 0:
			price += "   MOOD %d" % int(l.mood)
		make_label(f, price, Vector2(4, 51), UiTheme.ACID)
		if String(l.live) != "":
			make_label(f, ("waits: " + String(l.live)).left(37), Vector2(4, 61), UiTheme.DUST)
		elif l.has("reached"):
			var r: PackedStringArray = l.reached
			make_label(f, ("in force: " + (", ".join(r) if r.size() > 0 else "the Heart")).left(37), Vector2(4, 61), UiTheme.OOZE_GLOW)
		var id := String(l.id)
		match state:
			"open":
				make_button(f, "SIGN", Vector2(4, 71), func():
					note = String(sim.sign_law(id))
					note = ("signed " + String(l.name)) if note == "" else note
					_redraw())
			"signed", "spreading":
				make_button(f, "REPEAL", Vector2(4, 71), func():
					var why := String(sim.repeal_law(id))
					note = ("repealed " + String(l.name)) if why == "" else why
					_redraw())
				make_label(f, "shift %d  for %d against %d" % [int(l.shift), int(l.votes_for), int(l.votes_against)], Vector2(52, 74), UiTheme.DUST)
			"repealing":
				make_label(f, "the repeal is travelling", Vector2(4, 74), UiTheme.DUST)
			_:
				make_label(f, String(l.why).left(37), Vector2(4, 74), UiTheme.DUST)

func _draw_districts() -> void:
	for ch in districts_frame.get_children():
		ch.queue_free()
	var ds: Array = sim.districts(people)
	make_label(districts_frame, "DISTRICTS   %d" % ds.size(), Vector2(6, 4), UiTheme.CYAN)
	var y := 16
	if ds.is_empty():
		make_label(districts_frame, "none yet: a Node with three structures within 12 cords forms one, and Authority with it", Vector2(6, y), UiTheme.DUST)
		y += 12
	for d in ds:
		if y > 66:
			break
		var mood := int(d.mood)
		var mood_col: Color = UiTheme.HOT_PINK if mood < 20 else (UiTheme.ACID if mood < 30 else UiTheme.CHALK)
		var line := "%s   foreman %s   Mood %d" % [String(d.name).to_upper(), String(d.foreman_name) if String(d.foreman_name) != "" else "none", mood]
		if bool(d.strike):
			line += "   ON STRIKE"
		if bool(d.ready_to_defect):
			line += "   READY TO DEFECT"
		if not bool(d.board):
			line += "   no Board: no vote, no petitions"
		if int(d.refusals) > 0:
			line += "   refused %d" % int(d.refusals)
		make_label(districts_frame, line, Vector2(6, y), mood_col)
		y += 11
		var pet: Dictionary = d.petition
		if not pet.is_empty():
			var did := int(d.id)
			var ask := ""
			if String(pet.repeal) != "":
				ask = "repeal %s" % String(pet.repeal_name)
			else:
				var asks: PackedStringArray = pet.asks
				ask = "%s: build a %s" % [String(pet.need).to_lower(), String(asks[0]).replace("_", " ")]
			make_label(districts_frame, "  petition: %s   %d shifts%s" % [ask, int(pet.shifts_left), "   MISSED" if bool(pet.missed) else ""], Vector2(6, y), UiTheme.HOT_PINK if bool(pet.missed) else UiTheme.ACID)
			make_button(districts_frame, "ANSWER", Vector2(420, y - 3), func():
				if String(pet.repeal) != "":
					var why := String(sim.repeal_law(String(pet.repeal)))
					note = "repealed " + String(pet.repeal_name) if why == "" else why
					_redraw()
				else:
					var asks2: PackedStringArray = pet.asks
					Sim.build_tool = String(asks2[0])
					Screens.show_screen("colony"))
			make_button(districts_frame, "REFUSE", Vector2(484, y - 3), func():
				note = String(sim.refuse_petition(did))
				_redraw())
			y += 13
	# the board at the Heart: the last lines of the history
	var hist: PackedStringArray = sim.charter_history()
	var hy := 80
	make_label(districts_frame, "THE BOARD", Vector2(6, hy), UiTheme.CYAN)
	if hist.is_empty():
		make_label(districts_frame, "no law signed yet", Vector2(70, hy), UiTheme.DUST)
	for k in mini(2, hist.size()):
		make_label(districts_frame, String(hist[hist.size() - 1 - k]), Vector2(70, hy + k * 10), UiTheme.DUST)
