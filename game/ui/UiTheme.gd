class_name UiTheme
## Builds the one Theme every screen uses: 9-slice frames from ui.frame, buttons from ui.button, crisp text.

const CYAN := Color8(25, 230, 255)
const ACID := Color8(246, 255, 46)
const CHALK := Color8(181, 180, 176)
const DUST := Color8(142, 144, 151)
const SUMP := Color8(14, 16, 24)
const VOID := Color8(7, 7, 12)
const OOZE_GLOW := Color8(240, 166, 255)
const HOT_PINK := Color8(255, 46, 151)
const VESTMENT := Color8(242, 239, 228)
const DRAIN_LIGHT := Color8(53, 103, 156)
const MUZZLE := Color8(255, 106, 26)
const OXIDE := Color8(154, 112, 73)

static func nine(slot_id: String, state: String = "") -> StyleBoxTexture:
	var sb := StyleBoxTexture.new()
	sb.texture = Assets.texture(slot_id, state)
	var m := 4.0
	sb.texture_margin_left = m
	sb.texture_margin_right = m
	sb.texture_margin_top = m
	sb.texture_margin_bottom = m
	sb.content_margin_left = 6.0
	sb.content_margin_right = 6.0
	sb.content_margin_top = 4.0
	sb.content_margin_bottom = 4.0
	return sb

static var _pixel_font: FontFile

## The 3x5 pixel font from the ui.font.mono sheet (96 glyphs, ASCII 32..127, 4 px advance), as a bitmap FontFile.
## Provisional: the designed 5x7 font replaces the sheet in Phase F through the same slot.
static func pixel_font() -> FontFile:
	if _pixel_font != null:
		return _pixel_font
	var tex := Assets.texture("ui.font.mono")
	var ff := FontFile.new()
	if tex == null:
		return ff
	var img: Image = tex.get_image()
	var size := Vector2i(6, 0)
	ff.fixed_size = 6
	ff.set_texture_image(0, size, 0, img)
	ff.set_cache_ascent(0, 6, 5.0)
	ff.set_cache_descent(0, 6, 1.0)
	for code in range(32, 128):
		var x := (code - 32) * 4
		ff.set_glyph_texture_idx(0, size, code, 0)
		ff.set_glyph_uv_rect(0, size, code, Rect2(x, 0, 3, 5))
		ff.set_glyph_size(0, size, code, Vector2(3, 5))
		ff.set_glyph_offset(0, size, code, Vector2(0, -5))
		ff.set_glyph_advance(0, 6, code, Vector2(4, 0))
	_pixel_font = ff
	return ff

static func build() -> Theme:
	var t := Theme.new()
	t.default_font = pixel_font()
	t.default_font_size = 6
	t.set_stylebox("panel", "Panel", nine("ui.frame"))
	t.set_stylebox("panel", "PanelContainer", nine("ui.frame"))
	t.set_stylebox("normal", "Button", nine("ui.button", "idle"))
	t.set_stylebox("hover", "Button", nine("ui.button", "hover"))
	t.set_stylebox("pressed", "Button", nine("ui.button", "down"))
	t.set_stylebox("focus", "Button", nine("ui.button", "hover"))
	t.set_color("font_color", "Button", CHALK)
	t.set_color("font_hover_color", "Button", CYAN)
	t.set_color("font_pressed_color", "Button", ACID)
	t.set_color("font_color", "Label", CHALK)
	# a one-line field (New Run's seed and strain code): a 1 px Dust border, Cyan when focused, 2 px in
	t.set_font_size("font_size", "LineEdit", 6)
	t.set_color("font_color", "LineEdit", CHALK)
	t.set_color("caret_color", "LineEdit", CYAN)
	var field := StyleBoxFlat.new()
	field.bg_color = Color(0.04, 0.05, 0.07)
	field.border_color = DUST
	field.set_border_width_all(1)
	field.set_content_margin_all(2)
	field.content_margin_left = 3
	t.set_stylebox("normal", "LineEdit", field)
	var focused := field.duplicate() as StyleBoxFlat
	focused.border_color = CYAN
	t.set_stylebox("focus", "LineEdit", focused)
	return t
