extends ScreenBase
## Settings: video, audio, controls, gameplay, accessibility. Phase A: the controls that exist are wired; the rest are listed.

func _ready() -> void:
	screen_title = "SETTINGS"
	super._ready()

func _build_content(c: Control) -> void:
	var y := 0
	for row in [["VIDEO", "integer scale  fullscreen  vsync  CRT off  shake"], ["AUDIO", "master  music  world  ui  voice"], ["CONTROLS", "rebind  gamepad  mouse sensitivity"], ["GAMEPLAY", "tick speed default 2 Hz  auto-pause on alerts  confirm sells over 200"], ["ACCESSIBILITY", "colour-blind ooze ramp  text scale  reduced flash  hold-to-toggle"]]:
		make_label(c, row[0], Vector2(0, y), UiTheme.CYAN)
		make_label(c, row[1], Vector2(110, y), UiTheme.DUST)
		y += 16
	var fs := CheckButton.new()
	fs.text = "FULLSCREEN"
	fs.position = Vector2(0, 100)
	fs.focus_mode = Control.FOCUS_NONE
	fs.toggled.connect(func(on: bool): DisplayServer.window_set_mode(DisplayServer.WINDOW_MODE_FULLSCREEN if on else DisplayServer.WINDOW_MODE_WINDOWED))
	c.add_child(fs)
	var hz := HBoxContainer.new()
	hz.position = Vector2(0, 130)
	c.add_child(hz)
	for v in [1.0, 2.0, 4.0]:
		var b := Button.new()
		b.text = "%.0f HZ" % v
		b.focus_mode = Control.FOCUS_NONE
		b.pressed.connect(func(): Sim.set_speed(v))
		hz.add_child(b)
	make_label(c, "CURSOR", Vector2(0, 200), UiTheme.CYAN)
	var cx := 60
	for st in ["point", "grab", "dig", "sever"]:
		make_slot(c, "ui.cursor", Vector2(cx, 196), 0, 0, 0.0, st, 2)
		cx += 40
	make_label(c, "seed 11   strain code -   save: autosave each shift", Vector2(0, 240), UiTheme.DUST)
