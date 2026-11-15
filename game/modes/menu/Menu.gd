extends ScreenBase
## Title and menu (Full Design v3.19, the Game shell): Continue, New Run, Endless Sump, Daily Seed, Codex, Settings,
## Quit; the profile's endings and Daily best under the title. Continue goes back to the city this session holds, or,
## in a fresh session, loads the newest save on disk (Full Design: Saves).

var note: Label

func _ready() -> void:
	screen_title = "CYBERPUNK: ROSE"
	super._ready()
	nav.visible = false

func _build_content(c: Control) -> void:
	make_slot(c, "ivan.portrait", Vector2(40, 40), 0, 0, 0.0, "a1", 2)
	make_label(c, "RISE OF THE SLIMECITY EMPIRE", Vector2(200, 40), UiTheme.OOZE_GLOW)
	make_label(c, "Sewer Noir 32  -  every picture is a placeholder until art lands", Vector2(200, 56), UiTheme.DUST)
	var held: bool = Sim.sim != null
	var newest: String = "" if held else Sim.newest_save()
	var today: int = Sim.today()
	var tried: int = Sim.ensure_profile().daily_of(today)
	var y := 84
	var cont := make_button(c, "CONTINUE", Vector2(200, y), func():
		if not held:
			var why: String = Sim.load_game(newest)
			if why != "":
				note.text = why
				return
		Screens.show_screen("colony"))
	cont.disabled = not held and newest == ""
	y += 20
	make_button(c, "NEW RUN", Vector2(200, y), func():
		Sim.newrun_preset = {"mode": "campaign"}
		Screens.show_screen("newrun"))
	y += 20
	make_button(c, "ENDLESS SUMP", Vector2(200, y), func():
		Sim.newrun_preset = {"mode": "endless"}
		Screens.show_screen("newrun"))
	y += 20
	var daily := make_button(c, "DAILY SEED %d" % today, Vector2(200, y), func():
		var why: String = Sim.start_daily(today)
		if why != "":
			note.text = why)
	daily.disabled = tried >= 0
	y += 20
	make_button(c, "CODEX", Vector2(200, y), func(): Screens.show_screen("codex"))
	y += 20
	make_button(c, "SETTINGS", Vector2(200, y), func(): Screens.show_screen("settings"))
	y += 20
	make_button(c, "QUIT", Vector2(200, y), func(): get_tree().quit())
	# what the city in hand is, and what the profile holds
	var line := "no city this session, no save on disk"
	if held:
		var camp: Dictionary = Sim.sim.campaign()
		line = "seed %d  -  %s  -  %s" % [Sim.seed_value, String(camp.get("card", "")), String(camp.get("difficulty", "")).to_upper()]
	elif newest != "":
		var h: Dictionary = Sim.save_slots()[newest]
		line = "saved: %s  -  %s  -  seed %d" % [String(h.label), _when(int(h.saved_unix)), int(h.seed)]
	make_label(c, line, Vector2(200, 230), UiTheme.DUST)
	var p: Dictionary = Sim.ensure_profile().summary()
	var endings: Array = p.get("endings", [])
	var names := []
	for e in endings:
		if not (String(e.name) in names):
			names.append(String(e.name))
	make_label(c, "ENDINGS %d/5  %s" % [names.size(), ", ".join(names).to_upper()], Vector2(200, 244), UiTheme.CYAN)
	var best: Dictionary = p.get("daily_best", {})
	var daily_line := "DAILY: no try yet"
	if not best.is_empty():
		daily_line = "DAILY BEST %d (%d)" % [int(best.score), int(best.date)]
	if tried >= 0:
		daily_line += "   TODAY %d" % tried
	make_label(c, daily_line, Vector2(200, 258), UiTheme.CYAN)
	make_label(c, "STRAINBOOK %d   SCRAPDEX %d   LAWS UNLOCKED %d" % [int(p.strainbook), int(p.scrapdex), int(p.laws)], Vector2(200, 272), UiTheme.DUST)
	note = make_label(c, "", Vector2(200, 288), UiTheme.HOT_PINK)

## A save's time as the player's clock reads it.
func _when(unix: int) -> String:
	var d := Time.get_datetime_dict_from_unix_time(unix + int(Time.get_time_zone_from_system().get("bias", 0)) * 60)
	return "%04d-%02d-%02d %02d:%02d" % [int(d.year), int(d.month), int(d.day), int(d.hour), int(d.minute)]
