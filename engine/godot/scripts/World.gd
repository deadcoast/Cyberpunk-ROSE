extends Node2D
## Greybox renderer: substrate + Mat as an image texture, traffic as a second image, agents as glyphs at Street view.
## Colours are the SEEPAGE palette; shapes are placeholders.

const TILE := 16

# SEEPAGE
const C_SUMP := Color8(14, 16, 24)
const C_CULVERT := Color8(23, 26, 38)
const C_RUST_GRAY := Color8(74, 79, 94)
const C_REBAR := Color8(107, 111, 123)
const C_CHALK := Color8(181, 180, 176)
const C_SILT := Color8(42, 31, 25)
const C_RUST := Color8(110, 78, 52)
const C_OXIDE := Color8(154, 112, 73)
const C_DRAIN := Color8(18, 34, 56)
const C_CISTERN := Color8(31, 61, 99)
const C_BRUISE := Color8(91, 58, 74)
const C_FLESH_DIM := Color8(154, 96, 112)
const C_FLESH := Color8(217, 154, 166)
const C_MAT_DARK := Color8(15, 46, 28)
const C_MAT := Color8(31, 110, 58)
const C_MAT_LIT := Color8(63, 214, 110)
const C_MAT_GLOW := Color8(182, 255, 200)
const C_PINK := Color8(255, 46, 151)
const C_CYAN := Color8(25, 230, 255)
const C_ACID := Color8(246, 255, 46)
const C_VOID := Color8(7, 7, 12)

# substrate / hyphal codes (matsim::sim::sub / hy)
const SUB_ROCK := 0
const SUB_OPEN := 1
const SUB_PIPE := 2
const SUB_BRICK := 3
const SUB_DEPOSIT := 4
const SUB_WATER := 5
const SUB_GRATE := 6
const HY_EMPTY := 0
const HY_TIP := 1
const HY_CORD := 2
const HY_NODE := 3
const HY_NECRO := 4

var sim: RoseSim
var w := 0
var h := 0
var ground_img: Image
var ground_tex: ImageTexture
var ground: Sprite2D
var traffic_img: Image
var traffic_tex: ImageTexture
var traffic: Sprite2D
var view_index := 0
var hover := Vector2i(-1, -1)
var sites := {}
var ticks := 0

func bind(s: RoseSim) -> void:
	sim = s
	w = sim.width()
	h = sim.height()
	ground_img = Image.create_empty(w, h, false, Image.FORMAT_RGB8)
	ground_tex = ImageTexture.create_from_image(ground_img)
	ground = Sprite2D.new()
	ground.centered = false
	ground.texture = ground_tex
	ground.scale = Vector2(TILE, TILE)
	ground.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	add_child(ground)
	traffic_img = Image.create_empty(w, h, false, Image.FORMAT_RGBA8)
	traffic_tex = ImageTexture.create_from_image(traffic_img)
	traffic = Sprite2D.new()
	traffic.centered = false
	traffic.texture = traffic_tex
	traffic.scale = Vector2(TILE, TILE)
	traffic.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	add_child(traffic)
	sites = sim.sites() as Dictionary

func set_view(i: int) -> void:
	view_index = i
	traffic.visible = view_index > 0
	queue_redraw()

func set_hover(t: Vector2i) -> void:
	hover = t
	queue_redraw()

func on_tick() -> void:
	ticks += 1
	if ticks % 20 == 0:
		full_redraw()
	else:
		_paint_changed()
	_paint_traffic()
	queue_redraw()

func full_redraw() -> void:
	var sub: PackedByteArray = sim.substrate()
	var hy: PackedByteArray = sim.hyphae()
	var sig: PackedFloat32Array = sim.signal()
	for y in h:
		for x in w:
			var i := y * w + x
			ground_img.set_pixel(x, y, _cell_colour(sub[i], hy[i], sig[i]))
	for p in sites.get("pods", []):
		ground_img.set_pixel(p % w, p / w, C_OXIDE)
	for p in sites.get("pits", []):
		ground_img.set_pixel(p % w, p / w, C_ACID)
	for p in sites.get("markets", []):
		ground_img.set_pixel(p % w, p / w, C_PINK)
	ground_tex.update(ground_img)
	_paint_traffic()
	queue_redraw()

func _paint_changed() -> void:
	var changed: PackedInt32Array = sim.changed_cells()
	if changed.is_empty():
		return
	var sub: PackedByteArray = sim.substrate()
	var hy: PackedByteArray = sim.hyphae()
	var sig: PackedFloat32Array = sim.signal()
	for i in changed:
		ground_img.set_pixel(i % w, i / w, _cell_colour(sub[i], hy[i], sig[i]))
	ground_tex.update(ground_img)

func _cell_colour(s: int, hv: int, sg: float) -> Color:
	match hv:
		HY_TIP: return C_MAT_LIT
		HY_CORD: return C_MAT if sg > 0.0 else C_MAT_DARK
		HY_NODE: return C_MAT_GLOW
		HY_NECRO: return C_BRUISE
	match s:
		SUB_ROCK: return C_CULVERT
		SUB_BRICK: return C_RUST
		SUB_PIPE: return C_RUST_GRAY
		SUB_WATER: return C_CISTERN
		SUB_DEPOSIT: return C_REBAR
		SUB_GRATE: return C_CHALK
	return C_SUMP

func _paint_traffic() -> void:
	# traffic as light: 1-2 Mat Lit, 3-6 Mat Glow, 7+ Chalk; transparent elsewhere
	traffic_img.fill(Color(0, 0, 0, 0))
	var t: PackedInt32Array = sim.traffic()
	for i in t.size():
		var v: int = t[i]
		if v <= 0:
			continue
		var c := C_MAT_LIT if v < 3 else (C_MAT_GLOW if v < 7 else C_CHALK)
		traffic_img.set_pixel(i % w, i / w, c)
	traffic_tex.update(traffic_img)

func _draw() -> void:
	if sim == null:
		return
	if view_index == 0:
		_draw_street()
	if hover.x >= 0 and hover.y >= 0 and hover.x < w and hover.y < h:
		draw_rect(Rect2(hover.x * TILE, hover.y * TILE, TILE, TILE), C_ACID, false, 1.0)

func _draw_street() -> void:
	# only the tiles the camera can see
	var cam: Camera2D = get_viewport().get_camera_2d()
	if cam == null:
		return
	var half: Vector2 = get_viewport_rect().size * 0.5 / cam.zoom.x
	var x0 := int(floor((cam.position.x - half.x) / TILE)) - 1
	var y0 := int(floor((cam.position.y - half.y) / TILE)) - 1
	var x1 := int(ceil((cam.position.x + half.x) / TILE)) + 1
	var y1 := int(ceil((cam.position.y + half.y) / TILE)) + 1
	var pk: PackedInt32Array = sim.packet_positions()
	for p in pk:
		var px: int = p % w
		var py: int = p / w
		if px < x0 or px > x1 or py < y0 or py > y1:
			continue
		draw_rect(Rect2(px * TILE + 6, py * TILE + 6, 4, 4), C_CYAN)
	var pos: PackedInt32Array = sim.agent_positions()
	var st: PackedByteArray = sim.agent_states()
	var slot := {}
	for k in pos.size():
		var p: int = pos[k]
		var px: int = p % w
		var py: int = p / w
		if px < x0 or px > x1 or py < y0 or py > y1:
			continue
		var s: int = slot.get(p, 0)
		if s >= 4:
			continue
		slot[p] = s + 1
		var gx := px * TILE + 2 + (s % 2) * 7
		var gy := py * TILE + 1 + (s / 2) * 7
		var carrying: bool = st[k] == 2
		draw_rect(Rect2(gx - 1, gy - 1, 7, 9), C_VOID)
		draw_rect(Rect2(gx, gy, 5, 7), C_FLESH if carrying else C_FLESH_DIM)
		draw_rect(Rect2(gx + 2, gy + 3, 1, 1), C_MAT_GLOW)
		if carrying:
			draw_rect(Rect2(gx + 4, gy - 1, 2, 2), C_ACID)
