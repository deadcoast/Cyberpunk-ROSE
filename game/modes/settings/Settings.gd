extends ScreenBase
## Settings: video, audio, controls, gameplay, accessibility, and the saves. Phase A: the controls that exist are wired;
## the rest are listed.

func _ready() -> void:
	screen_title = "SETTINGS"
	super._ready()

func _build_content(c: Control) -> void:
	# each row says what works now; what the row will hold and does not yet is named "not yet" (PAC-108)
	make_label(c, "VIDEO", Vector2(0, 0), UiTheme.CYAN)
	var fs := CheckButton.new()
	fs.text = "FULLSCREEN"
	fs.position = Vector2(110, -4)
	fs.focus_mode = Control.FOCUS_NONE
	fs.button_pressed = DisplayServer.window_get_mode() == DisplayServer.WINDOW_MODE_FULLSCREEN
	fs.toggled.connect(func(on: bool): DisplayServer.window_set_mode(DisplayServer.WINDOW_MODE_FULLSCREEN if on else DisplayServer.WINDOW_MODE_WINDOWED))
	c.add_child(fs)
	make_label(c, "integer scale always   not yet: vsync, CRT, shake", Vector2(230, 0), UiTheme.DUST)
	# the sound channel's buses (Audio), each in steps of ten, kept in user://settings.cfg
	make_label(c, "AUDIO", Vector2(0, 20), UiTheme.CYAN)
	var ax := 110
	for bus in ["Master"] + Array(Audio.BUSES):
		var lb := make_label(c, "%s %d" % [String(bus).to_upper(), Audio.volume(bus)], Vector2(ax, 20), UiTheme.CHALK)
		var minus := make_button(c, "-", Vector2(ax, 30), func():
			Audio.set_volume(bus, Audio.volume(bus) - 10)
			lb.text = "%s %d" % [String(bus).to_upper(), Audio.volume(bus)]
			Audio.play("sfx.click"))
		minus.custom_minimum_size = Vector2(14, 0)
		var plus := make_button(c, "+", Vector2(ax + 18, 30), func():
			Audio.set_volume(bus, Audio.volume(bus) + 10)
			lb.text = "%s %d" % [String(bus).to_upper(), Audio.volume(bus)]
			Audio.play("sfx.click"))
		plus.custom_minimum_size = Vector2(14, 0)
		ax += 84
	make_label(c, "GAMEPLAY", Vector2(0, 56), UiTheme.CYAN)
	make_label(c, "tick speed", Vector2(110, 56), UiTheme.CHALK)
	var hz := HBoxContainer.new()
	hz.position = Vector2(170, 52)
	c.add_child(hz)
	for v in [1.0, 2.0, 4.0]:
		var b := Button.new()
		b.text = "%.0f HZ" % v
		b.focus_mode = Control.FOCUS_NONE
		b.pressed.connect(func(): Sim.set_speed(v))
		hz.add_child(b)
	make_label(c, "not yet: auto-pause on alerts, confirm sells over 200", Vector2(300, 56), UiTheme.DUST)
	make_label(c, "CONTROLS", Vector2(0, 76), UiTheme.CYAN)
	make_label(c, "not yet: rebind, gamepad, mouse sensitivity", Vector2(110, 76), UiTheme.DUST)
	make_label(c, "ACCESSIBILITY", Vector2(0, 92), UiTheme.CYAN)
	make_label(c, "not yet: colour-blind ooze ramp, text scale, reduced flash, hold-to-toggle", Vector2(110, 92), UiTheme.DUST)
	make_label(c, "CURSOR", Vector2(0, 116), UiTheme.CYAN)
	var cx := 110
	for st in ["point", "grab", "dig", "sever"]:
		make_slot(c, "ui.cursor", Vector2(cx, 108), 0, 0, 0.0, st, 2)
		cx += 40
	# the city in hand and its saves (Full Design: Saves): the seed and strain code it runs on, the last save written,
	# and the three slots, each to write the city into or to load it from
	var held: bool = Sim.sim != null and not Sim.sim.campaign().is_empty()
	var line := "no city this session"
	if held:
		line = "seed %d   strain code %s" % [Sim.seed_value, String(Sim.sim.strain_code())]
	make_label(c, line, Vector2(0, 168), UiTheme.DUST)
	var last := "autosave: every shift's end and each run's departure"
	if not Sim.last_save.is_empty():
		last += "   last: %s, %s" % [String(Sim.last_save.slot), String(Sim.last_save.label)]
	if Sim.save_error != "":
		last = "autosave FAILED: %s" % Sim.save_error
	save_note = make_label(c, last, Vector2(0, 180), UiTheme.HOT_PINK if Sim.save_error != "" else UiTheme.DUST)
	var slots: Dictionary = Sim.save_slots()
	var sy := 196
	for slot in Sim.SLOTS:
		var h: Dictionary = slots[slot]
		var what := "empty"
		if not h.is_empty():
			what = String(h.label) if String(h.get("why", "")) == "" else "does not read: %s" % String(h.why)
		make_label(c, "%-5s %s" % [String(slot).to_upper(), what], Vector2(0, sy + 4), UiTheme.CHALK if not h.is_empty() else UiTheme.DUST)
		if slot != "auto":
			var sb := make_button(c, "SAVE", Vector2(330, sy), func(): _save(slot))
			sb.disabled = not held
		var lb := make_button(c, "LOAD", Vector2(380, sy), func(): _load(slot))
		lb.disabled = h.is_empty() or String(h.get("why", "")) != ""
		sy += 20

var save_note: Label

## Write a slot, then the screen again with the slot's new line (or why it failed).
func _save(slot: String) -> void:
	var why: String = Sim.save_game(slot)
	var again = Screens.show_screen("settings")
	if why != "":
		again.save_note.text = "save FAILED: %s" % why

func _load(slot: String) -> void:
	var why: String = Sim.load_game(slot)
	if why != "":
		var again = Screens.show_screen("settings")
		again.save_note.text = "load FAILED: %s" % why
		return
	Screens.show_screen("colony")
