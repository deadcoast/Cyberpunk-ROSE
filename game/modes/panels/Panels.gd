extends ScreenBase
## Panels (Full Design v3.19, Narrative and Ivan: Panels): a chapter's or an ending's set full-screen — the panel at
## half scale in the 640 x 360 frame, Ivan's caption in a box under it — a key or a click a panel, ESC skips the set.
## The set is the campaign's (RoseSim.take_panels); the Codex's Lore replays one (Sim.panel_replay). When the set is
## done the screen goes back where it came from, or to the ending once the ending's set has played.

var set_data: Dictionary
var index := 0
var art: TextureRect
var caption: Label
var count_label: Label
var heading: Label

func _ready() -> void:
	screen_title = "PANELS"
	super._ready()
	nav.visible = false
	title_label.get_parent().visible = false

func _build_content(c: Control) -> void:
	var sim: RoseSim = Sim.ensure()
	if Sim.panel_replay != "":
		set_data = sim.panel_set(Sim.panel_replay)
		Sim.panel_replay = ""
	else:
		set_data = sim.take_panels()
	var back := ColorRect.new()
	back.color = UiTheme.VOID
	back.position = Vector2(-4, -26)
	back.size = Vector2(640, 360)
	c.add_child(back)
	art = TextureRect.new()
	art.position = Vector2(-4, -26)
	art.size = Vector2(640, 360)
	art.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	art.stretch_mode = TextureRect.STRETCH_SCALE
	art.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	c.add_child(art)
	heading = make_label(c, String(set_data.get("title", "")), Vector2(8, -18), UiTheme.CYAN)
	var box := make_frame(c, Vector2(40, 262), Vector2(552, 40))
	caption = make_label(box, "", Vector2(8, 6), UiTheme.CHALK)
	caption.autowrap_mode = TextServer.AUTOWRAP_WORD
	caption.size = Vector2(536, 30)
	count_label = make_label(c, "", Vector2(520, -18), UiTheme.DUST)
	make_label(c, "CLICK OR ANY KEY: ON   ESC: SKIP", Vector2(8, 312), UiTheme.DUST)
	_show()

func _panels() -> Array:
	return set_data.get("panels", [])

func _show() -> void:
	var ps := _panels()
	if index >= ps.size():
		_done()
		return
	var p: Dictionary = ps[index]
	art.texture = Assets.texture(String(p.slot))
	caption.text = String(p.caption)
	count_label.text = "%d / %d" % [index + 1, ps.size()]

func _done() -> void:
	var sim: RoseSim = Sim.ensure()
	# another set waiting plays next (a chapter's close, then the next's open)
	if bool(sim.campaign().get("waiting", false)) and not Sim.panel_hold:
		Screens.show_screen("panels")
		return
	var key := String(set_data.get("key", ""))
	if key.begins_with("end."):
		Screens.show_screen("ending")
		return
	Screens.show_screen(Sim.panel_return if Sim.panel_return != "" else "colony")

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		get_viewport().set_input_as_handled()
		if event.keycode == KEY_ESCAPE:
			index = _panels().size()
		else:
			index += 1
		_show()
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		get_viewport().set_input_as_handled()
		index += 1
		_show()
