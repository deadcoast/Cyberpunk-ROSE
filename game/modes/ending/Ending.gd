extends ScreenBase
## The ending (Full Design v3.19, Endings): after its panels, the final Stack-view shot of the city the player built,
## held for as long as they look at it, with the ending's name, the warchest, the strain code and NEW GAME+; an
## Endless city's burning with the shift its Empire came. The profile keeps either.

func _ready() -> void:
	screen_title = "THE END"
	super._ready()
	nav.visible = false

func _build_content(c: Control) -> void:
	var sim: RoseSim = Sim.ensure()
	var camp: Dictionary = sim.campaign()
	# the city as a circuit board: the stack's picture, one texel a tile, the reached strata
	var rgb: PackedByteArray = sim.stack_image()
	var w: int = sim.width()
	var h: int = sim.height()
	if rgb.size() == w * h * 3:
		var img := Image.create_from_data(w, h, false, Image.FORMAT_RGB8, rgb)
		var top := 0
		# from the slab row over the highest stratum reached down to the Sump's floor
		for st in sim.strata():
			if bool(st.reached):
				top = maxi(0, int(st.ceiling) - 1)
		var crop := img.get_region(Rect2i(0, top, w, h - top))
		var tex := TextureRect.new()
		tex.texture = ImageTexture.create_from_image(crop)
		tex.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		tex.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		tex.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		tex.position = Vector2(0, 14)
		tex.size = Vector2(632, 246)
		c.add_child(tex)
	# the profile keeps the ending (or the Endless city's rank), once
	if not Sim.ending_recorded and Sim.ensure_profile().record_end(sim):
		Sim.ending_recorded = true
		Sim.save_profile()
	var code: String = sim.strain_code()
	var f := make_frame(c, Vector2(0, 264), Vector2(632, 40))
	if String(camp.get("mode", "")) == "endless":
		var empire := int(camp.get("empire_at", -1))
		make_label(c, "THE HEART BURNED ON SHIFT %d" % int(camp.get("ended_at", 0)), Vector2(0, 0), UiTheme.HOT_PINK)
		make_label(f, ("THE EMPIRE AT SHIFT %d" % empire) if empire >= 0 else "THE EMPIRE NEVER CAME", Vector2(8, 6), UiTheme.CYAN)
		make_label(f, "SEED %d   WARCHEST %d   DIFFICULTY %s" % [int(camp.get("seed", 0)), int(camp.get("warchest", 0)), String(camp.get("difficulty", "")).to_upper()], Vector2(8, 20), UiTheme.DUST)
	else:
		var ending := String(camp.get("ending", ""))
		make_label(c, "THE %s ENDING" % ending.replace("_", "-").to_upper(), Vector2(0, 0), UiTheme.OOZE_GLOW)
		make_label(f, "WARCHEST %d   %s" % [int(camp.get("warchest", 0)), code], Vector2(8, 6), UiTheme.CYAN)
		make_label(f, "SHIFT %d   DIFFICULTY %s" % [int(camp.get("ended_at", 0)), String(camp.get("difficulty", "")).to_upper()], Vector2(8, 20), UiTheme.DUST)
		make_button(f, "NEW GAME+", Vector2(420, 16), func():
			Sim.newrun_preset = {"mode": "campaign", "code": code}
			Screens.show_screen("newrun"))
	make_button(f, "THE MENU", Vector2(520, 16), func(): Screens.show_screen("menu"))
