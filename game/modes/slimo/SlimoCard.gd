extends ScreenBase
## Slimo card: portrait, stats, skills, traits, needs, gear, record. Phase A: layout on placeholder data.

func _ready() -> void:
	screen_title = "SLIMO"
	super._ready()

func _build_content(c: Control) -> void:
	make_slot(c, "slimo.d2.idle", Vector2(0, 0), 32, 48, 4.0, "", 3)
	make_label(c, "MARA 'TIN HANDS' VOSS", Vector2(110, 0), UiTheme.CYAN)
	make_label(c, "D2 Limbed  -  Past: Gang  -  Runner  -  7 runs survived", Vector2(110, 12), UiTheme.DUST)
	var y := 30
	for stat in [["BODY", 0.6], ["NERVE", 0.5], ["HANDS", 0.8], ["EYE", 0.4], ["MAT-SENSE", 0.3], ["VOICE", 0.2]]:
		make_gauge(c, "ui.gauge.ooze", Vector2(110, y), stat[1], stat[0])
		y += 12
	make_label(c, "SKILLS", Vector2(320, 0), UiTheme.CYAN)
	y = 14
	for sk in [["SCAV", 6], ["FIGHT", 4], ["TWEAK", 1], ["SHEPHERD", 0], ["TRADE", 2], ["APPRAISE", 3], ["MEDIC", 1], ["GUARD", 2]]:
		make_label(c, "%-9s %s" % [sk[0], "#".repeat(sk[1]) + ".".repeat(10 - sk[1])], Vector2(320, y))
		y += 10
	make_label(c, "TRAITS  Hoarder - Night Eyes", Vector2(320, 100), UiTheme.OOZE_GLOW)
	make_label(c, "INJURIES  Fracture (left arm, 2 shifts)", Vector2(320, 112), UiTheme.HOT_PINK)
	make_label(c, "GEAR", Vector2(0, 160), UiTheme.CYAN)
	var x := 0
	for id in ["ger_plate_vest", "ger_nail_rifle", "ger_lantern", "ger_waders", "imp_night_eye"]:
		var f := make_frame(c, Vector2(x, 172), Vector2(40, 40))
		make_slot(f, icon_for(id), Vector2(4, 4))
		x += 44
	make_label(c, "NEEDS   Fed 72   Lit 40   Dry 90   Signal 100   Rest 55   Mood 61", Vector2(0, 220), UiTheme.DUST)
	make_label(c, "RECORD  14 hauls - 3 builds - 2 kills - survived the first Patrol", Vector2(0, 232), UiTheme.DUST)
	make_button(c, "JOB PRIORITIES", Vector2(0, 256), func(): Screens.show_screen("jobs"))
	make_button(c, "SEND ON A RUN", Vector2(110, 256), func(): Screens.show_screen("route"))
