class_name WorldView
extends Node2D
## The sewer and the Mat as TileMapLayers built from placeholder slots, 32 px per tile, plus the crowd.
## Shared by the Colony and Run screens.

const TILE := 32
const SUB := {0: "tile.concrete", 1: "", 2: "tile.pipe", 3: "tile.brick", 4: "tile.deposit", 5: "tile.water", 6: "tile.grate", 7: "tile.concrete"}
const HY := {1: ["mat.tip", ""], 2: ["mat.cord", "fed"], 3: ["mat.node", "fed"], 4: ["mat.necro", ""]}

var tileset: TileSet
var substrate: TileMapLayer
var matlayer: TileMapLayer
var crowd: Node2D
var sources := {}      # slot key -> [source_id, frames]
var w := 0
var h := 0
var draw_crowd := true

func _ready() -> void:
	tileset = TileSet.new()
	tileset.tile_size = Vector2i(TILE, TILE)
	substrate = TileMapLayer.new()
	substrate.tile_set = tileset
	add_child(substrate)
	matlayer = TileMapLayer.new()
	matlayer.tile_set = tileset
	add_child(matlayer)
	crowd = Node2D.new()
	crowd.draw.connect(_draw_crowd)
	add_child(crowd)
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

func _draw_crowd() -> void:
	if not draw_crowd:
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
