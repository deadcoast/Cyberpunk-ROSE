extends ScreenBase
## Expedition: direct control of a Runner on the world at Street zoom. Phase A: movement along passable tiles,
## the HUD (ooze bars, lume and noise arcs, run clock), extraction on reaching the Mat. Loot and threats land in Phase B.

var world: WorldView
var cam: Camera2D
var runner: Sprite2D
var tile := Vector2i(10, 12)
var move_cd := 0.0
var clock := 0.0
var hp := 1.0
var stamina := 1.0
var hud: Label
var frame := 0
var anim_t := 0.0
var passable: PackedByteArray
var hyphae: PackedByteArray

func _ready() -> void:
	screen_title = "RUN  -  SUMP EAST"
	super._ready()
	world = WorldView.new()
	world.draw_crowd = false
	add_child(world)
	passable = Sim.ensure().substrate()
	hyphae = Sim.ensure().hyphae()
	# spawn on the first open tile of the middle culvert
	for x in world.w:
		if _open(Vector2i(x, 25)):
			tile = Vector2i(x, 25)
			break
	runner = Sprite2D.new()
	runner.texture = Assets.texture("slimo.d2.walk")
	runner.region_enabled = true
	runner.region_rect = Rect2(0, 0, 32, 48)
	runner.centered = false
	runner.offset = Vector2(0, -16)
	runner.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	add_child(runner)
	cam = Camera2D.new()
	add_child(cam)
	cam.make_current()
	_place()

func _build_content(c: Control) -> void:
	make_gauge(c, "ui.gauge.ooze", Vector2(0, 270), 1.0, "HP")
	make_gauge(c, "ui.gauge.ooze", Vector2(0, 284), 1.0, "STAMINA")
	hud = make_label(c, "", Vector2(0, 0), UiTheme.CYAN)
	make_label(c, "WASD move   SHIFT sprint   E extract on Mat   TAB stash", Vector2(300, 284), UiTheme.DUST)
	var quick := HBoxContainer.new()
	quick.position = Vector2(480, 250)
	c.add_child(quick)
	for id in ["ger_lantern", "org_antiseptic", "org_stim", "prt_salt"]:
		var f := Panel.new()
		f.custom_minimum_size = Vector2(36, 36)
		quick.add_child(f)
		make_slot(f, icon_for(id), Vector2(2, 2))

func _open(t: Vector2i) -> bool:
	if t.x < 0 or t.y < 0 or t.x >= world.w or t.y >= world.h:
		return false
	var i := t.y * world.w + t.x
	return passable[i] != 0

func on_mat() -> bool:
	var i := tile.y * world.w + tile.x
	var hv := int(hyphae[i])
	return hv == 1 or hv == 2 or hv == 3

func _place() -> void:
	runner.position = Vector2(tile.x * WorldView.TILE, tile.y * WorldView.TILE)
	cam.position = runner.position + Vector2(16, 16)

func step(dir: Vector2i) -> bool:
	if _open(tile + dir):
		tile += dir
		_place()
		return true
	return false

func _process(delta: float) -> void:
	clock += delta
	move_cd -= delta
	var sprint := Input.is_key_pressed(KEY_SHIFT) and stamina > 0.05
	if move_cd <= 0.0:
		var d := Vector2i.ZERO
		if Input.is_key_pressed(KEY_W): d = Vector2i(0, -1)
		elif Input.is_key_pressed(KEY_S): d = Vector2i(0, 1)
		elif Input.is_key_pressed(KEY_A): d = Vector2i(-1, 0)
		elif Input.is_key_pressed(KEY_D): d = Vector2i(1, 0)
		if d != Vector2i.ZERO:
			if step(d):
				move_cd = 0.09 if sprint else 0.16
				if sprint:
					stamina = maxf(0.0, stamina - 0.03)
			runner.flip_h = d.x < 0
	stamina = minf(1.0, stamina + delta * 0.08)
	anim_t += delta
	if anim_t > 0.12:
		anim_t = 0.0
		frame = (frame + 1) % 8
		runner.region_rect = Rect2(frame * 32, 0, 32, 48)
	hud.text = "run %02d:%02d   tile %d,%d   %s   lume 6  noise %s" % [int(clock) / 60, int(clock) % 60, tile.x, tile.y, "ON THE MAT: extract with E" if on_mat() else "off the Mat", "sprint" if sprint else "quiet"]

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_E and on_mat():
			Screens.show_screen("stash")
		elif event.keycode == KEY_TAB:
			Screens.show_screen("stash")
