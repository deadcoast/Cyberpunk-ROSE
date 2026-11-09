class_name WorldView
extends Node2D
## The sewer and the Mat as TileMapLayers built from placeholder slots, 32 px per tile, plus the crowd.
## Shared by the Colony (bound to the live Sim) and the Run (painted from a RoseRun route map).

const TILE := 32
const SUB := {0: "tile.concrete", 1: "", 2: "tile.pipe", 3: "tile.brick", 4: "tile.deposit", 5: "tile.water", 6: "tile.grate", 7: "tile.concrete", 8: "tile.collapse"}
const HY := {1: ["mat.tip", ""], 2: ["mat.cord", "fed"], 3: ["mat.node", "fed"], 4: ["mat.necro", ""]}

var tileset: TileSet
var substrate: TileMapLayer
var matlayer: TileMapLayer
var props: TileMapLayer
var crowd: Node2D
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
	crowd = Node2D.new()
	crowd.draw.connect(_draw_crowd)
	add_child(crowd)
	buildings = Node2D.new()
	buildings.draw.connect(_draw_buildings)
	add_child(buildings)
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

## District view: a ring of the district's reach around its hub, its name and foreman.
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
		var r := 8 * TILE + TILE / 2
		districts.draw_rect(Rect2(hx - r, hy - r, r * 2, r * 2), Color(UiTheme.CYAN, 0.7), false, 2.0 * k)
		districts.draw_circle(Vector2(hx, hy), 3.0 * k, UiTheme.CYAN)
		var name := String(d.name).to_upper()
		if String(d.foreman_name) != "":
			name += "   FOREMAN " + String(d.foreman_name).to_upper()
		# the name inside the top-left corner of the reach on a dark backing, the petition inside the bottom-left
		districts.draw_set_transform(Vector2(hx - r + 3 * k, hy - r + 8 * k), 0.0, Vector2(k, k))
		districts.draw_rect(Rect2(-2, -6, name.length() * 4 + 3, 8), Color(UiTheme.SUMP, 0.85))
		districts.draw_string(font, Vector2.ZERO, name, HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.CYAN)
		var pet: Dictionary = d.petition
		if not pet.is_empty():
			var ptext := "PETITION: %s   %d SHIFTS" % [String(pet.need).to_upper(), int(pet.shifts_left)]
			districts.draw_set_transform(Vector2(hx - r + 3 * k, hy + r - 3 * k), 0.0, Vector2(k, k))
			districts.draw_rect(Rect2(-2, -6, ptext.length() * 4 + 3, 8), Color(UiTheme.SUMP, 0.85))
			districts.draw_string(font, Vector2.ZERO, ptext, HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.HOT_PINK if bool(pet.missed) else UiTheme.ACID)
		districts.draw_set_transform(Vector2.ZERO)

func redraw_city() -> void:
	buildings.queue_redraw()
	districts.queue_redraw()

func _draw_crowd() -> void:
	if not draw_crowd or not bind_sim:
		return
	var cam := get_viewport().get_camera_2d()
	if cam == null:
		return
	var tex := Assets.texture("slimo.d2.walk")
	if tex == null:
		return
	var sim: RoseSim = Sim.ensure()
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
		if s >= 2:
			continue
		slot[p] = s + 1
		var dst := Rect2(px * TILE + s * 12 - 6, py * TILE - 16, 32, 48)
		var src := Rect2(frame * 32, 0, 32, 48)
		crowd.draw_texture_rect_region(tex, dst, src, Color.WHITE if st[k] == 2 else Color(0.7, 0.7, 0.75))
