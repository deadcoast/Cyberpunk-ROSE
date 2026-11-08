extends ScreenBase
## Title and menu. Phase A: the entry points; the live Stack view behind the menu arrives with the colony's traffic texture.

func _ready() -> void:
	screen_title = "CYBERPUNK: ROSE"
	super._ready()
	nav.visible = false

func _build_content(c: Control) -> void:
	make_slot(c, "ivan.portrait", Vector2(40, 40), 0, 0, 0.0, "a1", 2)
	make_label(c, "RISE OF THE SLIMECITY EMPIRE", Vector2(200, 40), UiTheme.OOZE_GLOW)
	make_label(c, "Sewer Noir 32  -  Phase A shell  -  every picture is a placeholder until art lands", Vector2(200, 56), UiTheme.DUST)
	var y := 90
	for entry in [["CONTINUE", "colony"], ["NEW RUN", "route"], ["ENDLESS SUMP", "colony"], ["DAILY SEED", "route"], ["CODEX", "codex"], ["SETTINGS", "settings"]]:
		make_button(c, entry[0], Vector2(200, y), func(): Screens.show_screen(entry[1]))
		y += 22
	make_label(c, "seed 11  -  strain -", Vector2(200, 230), UiTheme.DUST)
