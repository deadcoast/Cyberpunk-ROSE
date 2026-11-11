extends Node
## Asset slots. A drawable is a slot id; this resolves art/ first, then placeholders/, never a path in a scene.
## Assets live under res://assets with a .gdignore, so they are read as files at runtime (no import churn).

const ART := "res://assets/art/"
const PLACEHOLDERS := "res://assets/placeholders/"

var _cache := {}
var _missing := []
var _resolved := 0

func _file_name(slot_id: String, state: String) -> String:
	return slot_id + ".png" if state == "" else slot_id + "." + state + ".png"

func texture(slot_id: String, state: String = "") -> Texture2D:
	var key := _file_name(slot_id, state)
	if _cache.has(key):
		return _cache[key]
	var tex: Texture2D = null
	for base in [ART, PLACEHOLDERS]:
		var path: String = ProjectSettings.globalize_path(base + key)
		if FileAccess.file_exists(path):
			var img := Image.load_from_file(path)
			if img != null:
				tex = ImageTexture.create_from_image(img)
				break
	if tex == null:
		_missing.append(key)
	else:
		_resolved += 1
	_cache[key] = tex
	return tex

## One frame of a horizontal strip.
func frame(slot_id: String, frame_w: int, frame_h: int, index: int, state: String = "") -> Texture2D:
	var base := texture(slot_id, state)
	if base == null:
		return null
	var at := AtlasTexture.new()
	at.atlas = base
	var frames := maxi(1, int(base.get_width() / frame_w))
	at.region = Rect2((index % frames) * frame_w, 0, frame_w, frame_h)
	return at

## The state an organ's sprite is drawn in where none is asked: the Sphincter's open ring (its slot has two states).
func organ_state(slot_id: String, shut: bool = false) -> String:
	if slot_id == "mat.organ.sphincter":
		return "closed" if shut else "open"
	return ""

func missing() -> Array:
	return _missing.duplicate()

func resolved_count() -> int:
	return _resolved
