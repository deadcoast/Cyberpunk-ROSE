class_name AssetSlot
extends TextureRect
## A drawable by slot id. Resolves through Assets (art, then placeholder). Strips show one frame; `fps` animates.

@export var slot_id: String = ""
@export var state: String = ""
@export var frame_w: int = 0
@export var frame_h: int = 0
@export var frame_index: int = 0
@export var fps: float = 0.0
@export var scale_factor: int = 1

var _frames := 1
var _t := 0.0

func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	stretch_mode = TextureRect.STRETCH_KEEP
	refresh()

func refresh() -> void:
	if slot_id == "":
		return
	if frame_w > 0 and frame_h > 0:
		var base := Assets.texture(slot_id, state)
		if base == null:
			return
		_frames = maxi(1, int(base.get_width() / frame_w))
		texture = Assets.frame(slot_id, frame_w, frame_h, frame_index, state)
		custom_minimum_size = Vector2(frame_w, frame_h) * scale_factor
	else:
		texture = Assets.texture(slot_id, state)
		if texture != null:
			custom_minimum_size = texture.get_size() * scale_factor
	if scale_factor != 1:
		stretch_mode = TextureRect.STRETCH_SCALE
		size = custom_minimum_size

func _process(delta: float) -> void:
	if fps <= 0.0 or _frames <= 1:
		return
	_t += delta
	var step := 1.0 / fps
	if _t >= step:
		_t -= step
		frame_index = (frame_index + 1) % _frames
		texture = Assets.frame(slot_id, frame_w, frame_h, frame_index, state)
