extends ScreenBase
## The ending (Full Design v3.19, Endings): after its panels, the final Stack-view shot of the city the player built,
## held for as long as they look at it, with the ending's name, the warchest and the strain code; a key: the menu.

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
	var ending := String(camp.get("ending", ""))
	var name := ending.replace("_", "-").to_upper()
	make_label(c, "THE %s ENDING" % name, Vector2(0, 0), UiTheme.OOZE_GLOW)
	var f := make_frame(c, Vector2(0, 264), Vector2(632, 40))
	make_label(f, "WARCHEST %d" % int(camp.get("warchest", 0)), Vector2(8, 6), UiTheme.CYAN)
	make_label(f, "%s   DIFFICULTY %s" % [String(camp.get("card", "")), String(camp.get("difficulty", "")).to_upper()], Vector2(8, 20), UiTheme.DUST)
	make_label(f, "ANY KEY: THE MENU", Vector2(500, 20), UiTheme.DUST)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		get_viewport().set_input_as_handled()
		Screens.show_screen("menu")
