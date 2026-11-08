extends SceneTree
## Screenshot every screen at native 640x360 into docs/shots/. Needs a display (Xvfb works):
##   xvfb-run -s "-screen 0 1920x1080x24" godot --path game --rendering-driver opengl3 --script res://tests/shots.gd

var screen_ids: Array = []
var screen_i := -1
var frames_on_screen := 0
var assets: Node
var screens: Node
var sim: Node
var out_dir := ""

func _autoload(name: String, path: String) -> Node:
	var n := root.get_node_or_null(name)
	if n == null:
		n = (load(path) as GDScript).new()
		n.name = name
		root.add_child(n)
	return n

func _initialize() -> void:
	assets = _autoload("Assets", "res://autoload/Assets.gd")
	screens = _autoload("Screens", "res://autoload/Screens.gd")
	sim = _autoload("Sim", "res://autoload/Sim.gd")
	root.add_child((load("res://Main.tscn") as PackedScene).instantiate())
	sim.ticks_per_frame = 2
	screen_ids = screens.ids()
	out_dir = ProjectSettings.globalize_path("res://../docs/shots")
	DirAccess.make_dir_recursive_absolute(out_dir)

func _process(_delta: float) -> bool:
	frames_on_screen += 1
	if frames_on_screen == 12 and screen_i >= 0:
		var id: String = screen_ids[screen_i]
		if id == "colony":
			screens.current.set_view(0)
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/" + id + ".png")
		if id == "colony":
			screens.current.set_view(2)
	if frames_on_screen == 16 and screen_i >= 0 and screen_ids[screen_i] == "colony":
		var img: Image = root.get_viewport().get_texture().get_image()
		img.save_png(out_dir + "/colony_stack.png")
	if frames_on_screen >= 18:
		screen_i += 1
		frames_on_screen = 0
		if screen_i < screen_ids.size():
			screens.show_screen(screen_ids[screen_i])
		else:
			print("[shots] wrote %d screens to %s" % [screen_ids.size() + 1, out_dir])
			quit(0)
			return true
	return false
