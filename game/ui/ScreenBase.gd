class_name ScreenBase
extends Control
## Every screen: a CanvasLayer of chrome (title, nav) over whatever the screen puts in the world.
## Subclasses override _build_content(content) and may add Node2D children for the world.

@export var screen_title: String = "SCREEN"

var ui: CanvasLayer
var content: Control
var title_label: Label
var nav: HBoxContainer

const NAV := ["colony", "route", "stash", "bench", "market", "strain", "charter", "codex", "settings"]

func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	ui = CanvasLayer.new()
	ui.name = "UI"
	add_child(ui)
	var root := Control.new()
	root.name = "Chrome"
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	root.theme = UiTheme.build()
	ui.add_child(root)
	# title bar
	var bar := PanelContainer.new()
	bar.position = Vector2(4, 4)
	bar.custom_minimum_size = Vector2(0, 16)
	root.add_child(bar)
	title_label = Label.new()
	title_label.text = screen_title
	title_label.add_theme_color_override("font_color", UiTheme.CYAN)
	bar.add_child(title_label)
	# nav bar
	nav = HBoxContainer.new()
	nav.position = Vector2(4, 360 - 22)
	nav.add_theme_constant_override("separation", 4)
	root.add_child(nav)
	for id in NAV:
		var b := Button.new()
		b.text = id.to_upper()
		b.focus_mode = Control.FOCUS_NONE
		b.pressed.connect(func(): Screens.show_screen(id))
		nav.add_child(b)
	var esc := Label.new()
	esc.text = "ESC MENU"
	esc.add_theme_color_override("font_color", UiTheme.DUST)
	nav.add_child(esc)
	# content region
	content = Control.new()
	content.name = "Content"
	content.position = Vector2(4, 26)
	content.size = Vector2(632, 306)
	content.mouse_filter = Control.MOUSE_FILTER_IGNORE
	root.add_child(content)
	_build_content(content)

func _build_content(_c: Control) -> void:
	pass

# ---------------------------------------------------------------- helpers every shell uses
func make_label(parent: Control, text: String, pos: Vector2, color: Color = UiTheme.CHALK) -> Label:
	var l := Label.new()
	l.text = text
	l.position = pos
	l.add_theme_color_override("font_color", color)
	parent.add_child(l)
	return l

func make_button(parent: Control, text: String, pos: Vector2, on_press: Callable = Callable()) -> Button:
	var b := Button.new()
	b.text = text
	b.position = pos
	b.focus_mode = Control.FOCUS_NONE
	if on_press.is_valid():
		b.pressed.connect(on_press)
	parent.add_child(b)
	return b

func make_frame(parent: Control, pos: Vector2, size_: Vector2) -> Panel:
	var p := Panel.new()
	p.position = pos
	p.size = size_
	parent.add_child(p)
	return p

func make_slot(parent: Control, slot_id: String, pos: Vector2, frame_w: int = 0, frame_h: int = 0, fps: float = 0.0, state: String = "", scale_: int = 1) -> AssetSlot:
	var s := AssetSlot.new()
	s.slot_id = slot_id
	s.state = state
	s.frame_w = frame_w
	s.frame_h = frame_h
	s.fps = fps
	s.scale_factor = scale_
	s.position = pos
	parent.add_child(s)
	return s

## A gauge from a strip slot: the fill is a cropped copy of the same texture.
func make_gauge(parent: Control, slot_id: String, pos: Vector2, fill: float, label: String = "") -> Control:
	var holder := Control.new()
	holder.position = pos
	parent.add_child(holder)
	var track := make_slot(holder, slot_id, Vector2.ZERO)
	track.modulate = Color(0.35, 0.35, 0.35)
	var fill_slot := make_slot(holder, slot_id, Vector2.ZERO)
	fill_slot.clip_contents = true
	fill_slot.size = Vector2(96 * clampf(fill, 0.0, 1.0), 8)
	fill_slot.custom_minimum_size = fill_slot.size
	if label != "":
		make_label(holder, label, Vector2(100, -2), UiTheme.DUST)
	return holder

func icon_for(item_id: String) -> String:
	return "icon.item." + item_id
