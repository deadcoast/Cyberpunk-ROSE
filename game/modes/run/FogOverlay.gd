class_name FogOverlay
extends Sprite2D
## The run's fog, light and noise fields drawn over the world by fog.gdshader. One texel per tile, scaled to the
## tile size; the shader dithers at art pixels. Fed each senses tick from RoseRun.overlay_rgb().

var w := 0
var h := 0
var image: Image
var tex: ImageTexture

func setup(width: int, height: int) -> void:
	w = width
	h = height
	image = Image.create_empty(w, h, false, Image.FORMAT_RGB8)
	tex = ImageTexture.create_from_image(image)
	texture = tex
	centered = false
	scale = Vector2(WorldView.TILE, WorldView.TILE)
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	z_index = 10
	var mat := ShaderMaterial.new()
	mat.shader = load("res://modes/run/fog.gdshader")
	mat.set_shader_parameter("data", tex)
	mat.set_shader_parameter("void_col", UiTheme.VOID)
	mat.set_shader_parameter("noise_col", UiTheme.CYAN)
	material = mat

func feed(rgb: PackedByteArray) -> void:
	if rgb.size() != w * h * 3:
		return
	image.set_data(w, h, false, Image.FORMAT_RGB8, rgb)
	tex.update(image)
