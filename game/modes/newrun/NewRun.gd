extends ScreenBase
## New Run (Full Design v3.19, the Game shell): the seed (random or typed), the strain code (blank, the last run's or
## typed: New Game+), the difficulty, the campaign or Endless Sump; BEGIN founds the city and chapter 1 opens.

const DIFFICULTY := {
	"Sump": "Attention decays 3 a shift; raiders never loot a Runner's corpse",
	"Culvert": "the design's values: Attention decays 2 a shift",
	"Undercroft": "Attention decays 1; one Spored item regrows a death; every day's first shift floods too",
}

var mode := "campaign"
var difficulty := "Culvert"
var seed_edit: LineEdit
var code_edit: LineEdit
var code_note: Label
var diff_note: Label
var why: Label
var mode_buttons := {}
var diff_buttons := {}

func _ready() -> void:
	mode = String(Sim.newrun_preset.get("mode", "campaign"))
	screen_title = "NEW RUN"
	super._ready()
	nav.visible = false

## A one-line field (UiTheme's LineEdit).
func _edit(c: Control, pos: Vector2, width: float, text: String) -> LineEdit:
	var e := LineEdit.new()
	e.text = text
	# sized once in the tree: out of it the field measures by the engine's theme
	c.add_child(e)
	e.position = pos
	e.size = Vector2(width, 12)
	return e

func _build_content(c: Control) -> void:
	var x := 120
	make_label(c, "MODE", Vector2(16, 20), UiTheme.DUST)
	mode_buttons["campaign"] = make_button(c, "CAMPAIGN", Vector2(x, 16), func(): _set_mode("campaign"))
	mode_buttons["endless"] = make_button(c, "ENDLESS SUMP", Vector2(x + 80, 16), func(): _set_mode("endless"))
	make_label(c, "SEED", Vector2(16, 50), UiTheme.DUST)
	seed_edit = _edit(c, Vector2(x, 46), 120, str(_random_seed()))
	make_button(c, "RANDOM", Vector2(x + 128, 46), func(): seed_edit.text = str(_random_seed()))
	make_label(c, "STRAIN CODE", Vector2(16, 80), UiTheme.DUST)
	code_edit = _edit(c, Vector2(x, 76), 220, String(Sim.newrun_preset.get("code", "")))
	code_edit.text_changed.connect(func(_t): _read_code())
	var last: String = String(Sim.ensure_profile().summary().get("last_code", ""))
	var lb := make_button(c, "LAST RUN'S", Vector2(x + 228, 76), func():
		code_edit.text = last
		_read_code())
	lb.disabled = last == ""
	make_button(c, "CLEAR", Vector2(x + 300, 76), func():
		code_edit.text = ""
		_read_code())
	code_note = make_label(c, "", Vector2(x, 96), UiTheme.DUST)
	make_label(c, "DIFFICULTY", Vector2(16, 126), UiTheme.DUST)
	var dx := x
	for d in DIFFICULTY.keys():
		var name: String = d
		diff_buttons[name] = make_button(c, name.to_upper(), Vector2(dx, 122), func(): _set_difficulty(name))
		dx += 84
	diff_note = make_label(c, "", Vector2(x, 142), UiTheme.DUST)
	make_button(c, "BEGIN", Vector2(x, 180), _begin)
	make_button(c, "BACK", Vector2(x + 60, 180), func(): Screens.show_screen("menu"))
	why = make_label(c, "", Vector2(x, 204), UiTheme.HOT_PINK)
	_set_mode(mode)
	_set_difficulty(difficulty)
	_read_code()

func _random_seed() -> int:
	return randi() % 100000000

func _set_mode(m: String) -> void:
	mode = m
	title_label.text = "NEW RUN  -  " + ("ENDLESS SUMP" if m == "endless" else "THE CAMPAIGN")
	for k in mode_buttons:
		mode_buttons[k].add_theme_color_override("font_color", UiTheme.OOZE_GLOW if k == m else UiTheme.DUST)

func _set_difficulty(d: String) -> void:
	difficulty = d
	diff_note.text = DIFFICULTY[d]
	for k in diff_buttons:
		diff_buttons[k].add_theme_color_override("font_color", UiTheme.OOZE_GLOW if k == d else UiTheme.DUST)

## The code read as it is typed: blank a new strain, a good code New Game+ with what it carries, else why not.
func _read_code() -> void:
	var t := code_edit.text.strip_edges()
	if t == "":
		code_note.text = "blank: the Heart is born at the founding's genome"
		code_note.add_theme_color_override("font_color", UiTheme.DUST)
		return
	var probe := RoseSim.new()
	var err: String = probe.configure_run(mode, difficulty, t, null)
	if err != "":
		code_note.text = err
		code_note.add_theme_color_override("font_color", UiTheme.HOT_PINK)
		return
	var p: Dictionary = Sim.ensure_profile().summary()
	code_note.text = "NEW GAME+: the Heart born with this genome; %d laws unlocked, the Scrapdex (%d) kept" % [int(p.laws), int(p.scrapdex)]
	code_note.add_theme_color_override("font_color", UiTheme.CYAN)

func _begin() -> void:
	if not seed_edit.text.strip_edges().is_valid_int():
		why.text = "the seed is a number"
		return
	var s := int(seed_edit.text.strip_edges())
	if s < 0 or s > 4294967295:
		why.text = "the seed is 0 to 4294967295 (the strain code carries eight hex digits of it)"
		return
	var err: String = Sim.new_run(s, mode, difficulty, code_edit.text.strip_edges())
	if err != "":
		why.text = err
		return
	Screens.show_screen("colony")
