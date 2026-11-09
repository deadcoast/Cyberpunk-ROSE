class_name WorldView
extends Node2D
## The sewer and the Mat as TileMapLayers built from placeholder slots, 32 px per tile, the roster on it and the
## cargo riding it (Full Design v3.11, Cargo). Shared by the Colony (bound to the live Sim) and the Run (painted from
## a RoseRun route map). The Stack view draws traffic per tile as light and a jam Chalk white (render::stack_view).

const TILE := 32
const SUB := {0: "tile.concrete", 1: "", 2: "tile.pipe", 3: "tile.brick", 4: "tile.deposit", 5: "tile.water", 6: "tile.grate", 7: "tile.concrete", 8: "tile.collapse"}
const HY := {1: ["mat.tip", ""], 2: ["mat.cord", "fed"], 3: ["mat.node", "fed"], 4: ["mat.necro", ""]}

var tileset: TileSet
var substrate: TileMapLayer
var matlayer: TileMapLayer
var props: TileMapLayer
var crowd: Node2D
var cargo: Node2D      # packets as pulses in their tier's colour (Street 8 px, District 4 px)
var traffic_sprite: Sprite2D   # the Stack view: the core's circuit board, one texel a tile
var traffic_img: Image
var view_level := 0    # 0 Street, 1 District, 2 Stack (set by the Colony)
var buildings: Node2D  # the colony's structures and ghosts, drawn from RoseSim.structures()
var districts: Node2D  # hub rings and names, District view
var show_districts := false
var sources := {}      # slot key -> [source_id, frames]
var w := 0
var h := 0
var draw_crowd := true
var bind_sim := true   # false: a Run paints its own grid and nothing ticks

func _ready() -> void:
	tileset = TileSet.new()
	tileset.tile_size = Vector2i(TILE, TILE)
	substrate = TileMapLayer.new()
	substrate.tile_set = tileset
	add_child(substrate)
	matlayer = TileMapLayer.new()
	matlayer.tile_set = tileset
	add_child(matlayer)
	props = TileMapLayer.new()
	props.tile_set = tileset
	add_child(props)
	buildings = Node2D.new()
	buildings.draw.connect(_draw_buildings)
	add_child(buildings)
	cargo = Node2D.new()
	cargo.draw.connect(_draw_cargo)
	add_child(cargo)
	crowd = Node2D.new()
	crowd.draw.connect(_draw_crowd)
	add_child(crowd)
	traffic_sprite = Sprite2D.new()
	traffic_sprite.centered = false
	traffic_sprite.scale = Vector2(TILE, TILE)
	traffic_sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	traffic_sprite.visible = false
	add_child(traffic_sprite)
	districts = Node2D.new()
	districts.draw.connect(_draw_districts)
	add_child(districts)
	if bind_sim:
		var sim: RoseSim = Sim.ensure()
		w = sim.width()
		h = sim.height()
		full_paint()
		Sim.ticked.connect(_on_tick)

func _source(slot_id: String, state: String) -> Array:
	var key := slot_id + "|" + state
	if sources.has(key):
		return sources[key]
	var tex := Assets.texture(slot_id, state)
	if tex == null:
		return [-1, 1]
	var src := TileSetAtlasSource.new()
	src.texture = tex
	src.texture_region_size = Vector2i(TILE, TILE)
	var frames := maxi(1, int(tex.get_width() / TILE))
	for i in frames:
		src.create_tile(Vector2i(i, 0))
	var sid := tileset.add_source(src)
	sources[key] = [sid, frames]
	return sources[key]

func _put_cell(layer: TileMapLayer, x: int, y: int, slot_id: String, state: String, variant: int) -> void:
	if slot_id == "":
		layer.erase_cell(Vector2i(x, y))
		return
	var s: Array = _source(slot_id, state)
	if s[0] < 0:
		return
	layer.set_cell(Vector2i(x, y), s[0], Vector2i(variant % int(s[1]), 0))

## Paint a substrate grid (mat::sim::sub codes) of width x height; clears the Mat and prop layers.
func paint_substrate(sub: PackedByteArray, width: int, height: int) -> void:
	w = width
	h = height
	substrate.clear()
	matlayer.clear()
	props.clear()
	for y in h:
		for x in w:
			_put_cell(substrate, x, y, SUB.get(int(sub[y * w + x]), ""), "", (x * 7 + y * 13) % 4)

## Repaint one substrate tile (after a dig).
func repaint_tile(x: int, y: int, code: int) -> void:
	_put_cell(substrate, x, y, SUB.get(code, ""), "", (x * 7 + y * 13) % 4)

## A prop on a floor tile: lockers, crates, bodies, lamps, gas, burrows, waymarks.
func put_prop(x: int, y: int, slot_id: String, state: String = "", variant: int = 0) -> void:
	_put_cell(props, x, y, slot_id, state, variant)

## Mat tiles painted by hand (the Run paints the Mat edge it starts from).
func put_mat(x: int, y: int, hv: int, state: String) -> void:
	var spec: Array = HY.get(hv, ["", ""])
	_put_cell(matlayer, x, y, spec[0], state if hv == 2 or hv == 3 else "", (x + y) % 4)

func full_paint() -> void:
	var sim: RoseSim = Sim.ensure()
	var sub: PackedByteArray = sim.substrate()
	var hy: PackedByteArray = sim.hyphae()
	var sig: PackedFloat32Array = sim.signal()
	for y in h:
		for x in w:
			var i := y * w + x
			_put_cell(substrate, x, y, SUB.get(int(sub[i]), ""), "", (x * 7 + y * 13) % 4)
			_paint_mat(x, y, int(hy[i]), sig[i])

func _paint_mat(x: int, y: int, hv: int, sg: float) -> void:
	if hv == 0:
		matlayer.erase_cell(Vector2i(x, y))
		return
	var spec: Array = HY.get(hv, ["", ""])
	var state: String = spec[1]
	if hv == 2 or hv == 3:
		state = "fed" if sg > 0.3 else ("lean" if sg > 0.0 else "starving")
	_put_cell(matlayer, x, y, spec[0], state, (x + y) % 4)

func _on_tick() -> void:
	var sim: RoseSim = Sim.ensure()
	var changed: PackedInt32Array = sim.changed_cells()
	if not changed.is_empty():
		var hy: PackedByteArray = sim.hyphae()
		var sig: PackedFloat32Array = sim.signal()
		for i in changed:
			_paint_mat(i % w, i / w, int(hy[i]), sig[i])
	crowd.queue_redraw()
	cargo.queue_redraw()
	if view_level == 2:
		paint_traffic()

## The Stack view (Full Design v3.11): the stratum as a circuit board at one pixel a tile, drawn by the core
## (render::stack_view) — the Mat dim, traffic as light, a jam (a full tile with cargo waiting to enter it) Chalk
## white, consumers Acid, markets Hot Pink — in place of the tiles, which at a thirty-second zoom would only sample
## their own texels.
func paint_traffic() -> void:
	var sim: RoseSim = Sim.ensure()
	var rgb: PackedByteArray = sim.stack_image()
	if rgb.size() != w * h * 3:
		return
	traffic_img = Image.create_from_data(w, h, false, Image.FORMAT_RGB8, rgb)
	if traffic_sprite.texture == null or traffic_sprite.texture.get_width() != w:
		traffic_sprite.texture = ImageTexture.create_from_image(traffic_img)
	else:
		(traffic_sprite.texture as ImageTexture).update(traffic_img)

func set_view_level(i: int) -> void:
	view_level = i
	var stack := i == 2
	traffic_sprite.visible = stack
	for layer in [substrate, matlayer, props, buildings, cargo, crowd]:
		layer.visible = not stack
	if stack:
		paint_traffic()
	cargo.queue_redraw()
	crowd.queue_redraw()

## Packets riding the Mat as pulses in their item's tier colour: at Street a quarter of a tile, up to four side by
## side on a tile; at District one larger pulse a tile, so it reads at a quarter zoom. The Stack view counts them into
## traffic instead.
func _draw_cargo() -> void:
	if not bind_sim or view_level == 2:
		return
	var sim: RoseSim = Sim.ensure()
	var pos: PackedInt32Array = sim.packet_positions()
	if pos.is_empty():
		return
	var tiers: PackedByteArray = sim.packet_tiers()
	var size := 8 if view_level == 0 else 16
	var cols := [UiTheme.DUST, UiTheme.CYAN, UiTheme.HOT_PINK, UiTheme.OOZE_GLOW, UiTheme.VESTMENT]
	var seen := {}
	for k in pos.size():
		var p: int = pos[k]
		var n: int = seen.get(p, 0)
		seen[p] = n + 1
		var col: Color = cols[mini(int(tiers[k]) if k < tiers.size() else 0, 4)]
		# up to four to a tile, side by side along the tile's middle
		var x := (p % w) * TILE + 2 + (n % 4) * (TILE - 4 - size) / 3 if view_level == 0 else (p % w) * TILE + (TILE - size) / 2
		var y := (p / w) * TILE + (TILE - size) / 2
		cargo.draw_rect(Rect2(x - 1, y - 1, size + 2, size + 2), UiTheme.VOID)
		cargo.draw_rect(Rect2(x, y, size, size), col)

## Structures stand on their footprint: the sprite's bottom edge on the footprint's bottom row, its width the
## footprint's. A ghost is half-transparent with its build progress as a bar; a damaged one shows its HP.
func _draw_buildings() -> void:
	if not bind_sim:
		return
	var sim: RoseSim = Sim.ensure()
	for st in sim.structures():
		var tex := Assets.texture(String(st.art), "t%d" % int(st.tier))
		if tex == null:
			continue
		var fw := int(st.w) * TILE
		var fh := int(st.h) * TILE
		var dst := Rect2(int(st.x) * TILE, int(st.y) * TILE + fh - tex.get_height(), tex.get_width(), tex.get_height())
		var built := bool(st.built)
		buildings.draw_texture_rect(tex, dst, false, Color(1, 1, 1, 1.0 if built else 0.45))
		# the footprint's outline on the floor, Cyan for a ghost
		buildings.draw_rect(Rect2(int(st.x) * TILE, int(st.y) * TILE, fw, fh), UiTheme.CYAN if not built else Color(UiTheme.OOZE_GLOW, 0.5), false, 1.0)
		if not built or int(st.upgrading_to) > 0:
			var total := maxi(1, int(st.build_total))
			var frac := clampf(float(st.progress) / total, 0.0, 1.0)
			buildings.draw_rect(Rect2(int(st.x) * TILE, int(st.y) * TILE + fh + 2, fw, 3), UiTheme.SUMP)
			buildings.draw_rect(Rect2(int(st.x) * TILE, int(st.y) * TILE + fh + 2, fw * frac, 3), UiTheme.ACID if bool(st.paid) else UiTheme.HOT_PINK)
		elif int(st.hp) < int(st.hp_max):
			var frac := float(st.hp) / maxf(1.0, float(st.hp_max))
			buildings.draw_rect(Rect2(int(st.x) * TILE, int(st.y) * TILE + fh + 2, fw * frac, 2), UiTheme.HOT_PINK)

## District view: the district's reach (the box of the tiles within 12 cords of its hub), the hub, its name and
## foreman, the Foreman's Cut's stock when it holds any, and a standing petition.
func _draw_districts() -> void:
	if not bind_sim or not show_districts:
		return
	var sim: RoseSim = Sim.ensure()
	var cam := get_viewport().get_camera_2d()
	var k := 1.0 / (cam.zoom.x if cam != null else 1.0)   # the pixel font is 6 px on screen whatever the zoom
	var font := UiTheme.pixel_font()
	for d in sim.districts(Sim.ensure_people()):
		var hx := int(d.hub_x) * TILE + TILE / 2
		var hy := int(d.hub_y) * TILE + TILE / 2
		var rc: PackedInt32Array = d.get("reach", PackedInt32Array([int(d.hub_x) - 12, int(d.hub_y) - 12, int(d.hub_x) + 12, int(d.hub_y) + 12]))
		var box := Rect2(rc[0] * TILE, rc[1] * TILE, (rc[2] - rc[0] + 1) * TILE, (rc[3] - rc[1] + 1) * TILE)
		districts.draw_rect(box, Color(UiTheme.CYAN, 0.7), false, 2.0 * k)
		districts.draw_circle(Vector2(hx, hy), 3.0 * k, UiTheme.CYAN)
		var name := String(d.name).to_upper()
		if String(d.foreman_name) != "":
			name += "   FOREMAN " + String(d.foreman_name).to_upper()
		if int(d.get("cut", 0)) > 0:
			name += "   CUT %d" % int(d.cut)
		# the name inside the top-left corner of the reach on a dark backing, the petition inside the bottom-left
		districts.draw_set_transform(box.position + Vector2(3 * k, 8 * k), 0.0, Vector2(k, k))
		districts.draw_rect(Rect2(-2, -6, name.length() * 4 + 3, 8), Color(UiTheme.SUMP, 0.85))
		districts.draw_string(font, Vector2.ZERO, name, HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.CYAN)
		var pet: Dictionary = d.petition
		if not pet.is_empty():
			var ptext := "PETITION: %s   %d SHIFTS" % [String(pet.need).to_upper(), int(pet.shifts_left)]
			districts.draw_set_transform(Vector2(box.position.x + 3 * k, box.end.y - 3 * k), 0.0, Vector2(k, k))
			districts.draw_rect(Rect2(-2, -6, ptext.length() * 4 + 3, 8), Color(UiTheme.SUMP, 0.85))
			districts.draw_string(font, Vector2.ZERO, ptext, HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.HOT_PINK if bool(pet.missed) else UiTheme.ACID)
		districts.draw_set_transform(Vector2.ZERO)

func redraw_city() -> void:
	buildings.queue_redraw()
	districts.queue_redraw()

## The roster on the map: each slimo by its depth, walking or standing at work; up to four drawn to a tile.
func _draw_crowd() -> void:
	if not draw_crowd or not bind_sim:
		return
	var cam := get_viewport().get_camera_2d()
	if cam == null:
		return
	var sim: RoseSim = Sim.ensure()
	var ids: PackedInt32Array = sim.agent_ids()
	var people: RosePeople = Sim.ensure_people()
	var half: Vector2 = get_viewport_rect().size * 0.5 / cam.zoom.x
	var x0 := int(floor((cam.position.x - half.x) / TILE)) - 1
	var y0 := int(floor((cam.position.y - half.y) / TILE)) - 2
	var x1 := int(ceil((cam.position.x + half.x) / TILE)) + 1
	var y1 := int(ceil((cam.position.y + half.y) / TILE)) + 1
	var pos: PackedInt32Array = sim.agent_positions()
	var st: PackedByteArray = sim.agent_states()
	var frame := int(Time.get_ticks_msec() / 125) % 8
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
		var rec: Dictionary = people.record(ids[k]) if k < ids.size() else {}
		var depth := String(rec.get("depth", "D2")).to_lower()
		var moving: bool = st[k] == 3 or st[k] == 4
		var tex := Assets.texture("slimo.%s.%s" % [depth, "walk" if moving else "idle"])
		if depth == "d5":
			tex = Assets.texture("slimo.d5.pile")
		if tex == null:
			continue
		var fw := 32
		var frames := maxi(1, tex.get_width() / fw)
		var dst := Rect2(px * TILE + s * 8 - 12, py * TILE + TILE - tex.get_height(), fw, tex.get_height())
		var src := Rect2((frame % frames) * fw, 0, fw, tex.get_height())
		crowd.draw_texture_rect_region(tex, dst, src, Color.WHITE)
