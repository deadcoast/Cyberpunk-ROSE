class_name WorldView
extends Node2D
## The sewer and the Mat as TileMapLayers built from placeholder slots, 32 px per tile, the roster on it and the
## cargo riding it (Full Design v3.11, Cargo). Shared by the Colony (bound to the live Sim) and the Run (painted from
## a RoseRun route map). The Stack view draws traffic per tile as light and a jam Chalk white (render::stack_view).
## The Colony's layers (v3.12): salt on the prop layer, the overlay (one number a tile, RoseSim.overlay), zones
## outlined by kind, the items a stockpile holds on its cells, and the cuts marked for Build. Pressure (v3.14): the
## Flood's water repainted as it rises and falls, bleach as a pale stain, a Bloom's tissue in Pond Scum, the white
## light as hard pools round the lamps, and the bodies on the map over everything.

const TILE := 32
const SUB := {0: "tile.concrete", 1: "", 2: "tile.pipe", 3: "tile.brick", 4: "tile.deposit", 5: "tile.water", 6: "tile.grate", 7: "tile.concrete", 8: "tile.collapse", 9: "tile.slab"}
const HY := {1: ["mat.tip", ""], 2: ["mat.cord", "fed"], 3: ["mat.node", "fed"], 4: ["mat.necro", ""]}

var tileset: TileSet
var substrate: TileMapLayer
var matlayer: TileMapLayer
var props: TileMapLayer
var crowd: Node2D
var cargo: Node2D      # packets as pulses in their tier's colour (Street 8 px, District 4 px)
var traffic_sprite: Sprite2D   # the Stack view: the core's circuit board, one texel a tile
var traffic_img: Image
var overlay_sprite: Sprite2D   # an overlay at one texel a tile over the Street and District views
var overlay_kind := ""
var view_level := 0    # 0 Street, 1 District, 2 Stack (set by the Colony)
var buildings: Node2D  # the colony's structures and ghosts, drawn from RoseSim.structures()
var threat_fx: Node2D  # bleach, Bloom tissue and the white light, under the structures
var threat_bodies: Node2D  # Oathkeepers, crawlers and tamed crawlers, over the crowd
var flooded := {}      # tile -> true: what the Flood has under water now (repainted when it changes)
var gauges: Node2D     # Attention at each reached grate, in every view (Full Design: the Stack view keeps them)
var districts: Node2D  # hub rings and names, District view
var show_districts := false
var sources := {}      # slot key -> [source_id, frames]
var w := 0
var h := 0
var draw_crowd := true
var bind_sim := true   # false: a Run paints its own grid and nothing ticks
## The stack (Full Design v3.19): the strata from the Sump up; rows above `top_row` (the slab row over the city's
## stratum) belong to strata not yet reached and draw as Void with their names
var strata_info: Array = []
var top_row := 0
var strata_names: Node2D
## Ivan on the map (Full Design v3.19): his sprite by his Authority state, his goal dashed, his HP when hurt
var ivan_node: Node2D

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
	overlay_sprite = Sprite2D.new()
	overlay_sprite.centered = false
	overlay_sprite.scale = Vector2(TILE, TILE)
	overlay_sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	overlay_sprite.visible = false
	add_child(overlay_sprite)
	threat_fx = Node2D.new()
	threat_fx.draw.connect(_draw_threat_fx)
	add_child(threat_fx)
	buildings = Node2D.new()
	buildings.draw.connect(_draw_buildings)
	add_child(buildings)
	cargo = Node2D.new()
	cargo.draw.connect(_draw_cargo)
	add_child(cargo)
	crowd = Node2D.new()
	crowd.draw.connect(_draw_crowd)
	add_child(crowd)
	threat_bodies = Node2D.new()
	threat_bodies.draw.connect(_draw_threat_bodies)
	add_child(threat_bodies)
	traffic_sprite = Sprite2D.new()
	traffic_sprite.centered = false
	traffic_sprite.scale = Vector2(TILE, TILE)
	traffic_sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	traffic_sprite.visible = false
	add_child(traffic_sprite)
	districts = Node2D.new()
	districts.draw.connect(_draw_districts)
	add_child(districts)
	gauges = Node2D.new()
	gauges.z_index = 10
	gauges.draw.connect(_draw_gauges)
	add_child(gauges)
	ivan_node = Node2D.new()
	ivan_node.z_index = 5
	ivan_node.draw.connect(_draw_ivan)
	add_child(ivan_node)
	strata_names = Node2D.new()
	strata_names.z_index = 9
	strata_names.draw.connect(_draw_strata_names)
	add_child(strata_names)
	if bind_sim:
		var sim: RoseSim = Sim.ensure()
		w = sim.width()
		h = sim.height()
		refresh_strata()
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

## Read the city's stratum; true when it rose (a stratum newly reached: repaint).
func refresh_strata() -> bool:
	strata_info = Sim.ensure().strata()
	var row := 0
	for st in strata_info:
		if bool(st.reached):
			row = maxi(0, int(st.ceiling) - 1)
	if strata_info.is_empty() or row == top_row:
		return false
	top_row = row
	return true

func full_paint() -> void:
	var sim: RoseSim = Sim.ensure()
	var sub: PackedByteArray = sim.substrate()
	var hy: PackedByteArray = sim.hyphae()
	var sig: PackedFloat32Array = sim.signal()
	substrate.clear()
	matlayer.clear()
	for y in range(top_row, h):
		for x in w:
			var i := y * w + x
			_put_cell(substrate, x, y, SUB.get(int(sub[i]), ""), "", (x * 7 + y * 13) % 4)
			_paint_mat(x, y, int(hy[i]), sig[i])
	props.clear()
	for i in sim.salt_cells():
		put_prop(i % w, i / w, "tile.salt", "", i % 2)
	strata_names.queue_redraw()
	flooded = {}
	for i in sim.flooded_cells():
		flooded[int(i)] = true

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
	if refresh_strata():
		full_paint()
	var changed: PackedInt32Array = sim.changed_cells()
	if not changed.is_empty():
		var hy: PackedByteArray = sim.hyphae()
		var sig: PackedFloat32Array = sim.signal()
		var sub: PackedByteArray = sim.substrate()
		for i in changed:
			if i / w < top_row:
				continue
			# a breach opens a slab: its tiles change substrate
			_put_cell(substrate, i % w, i / w, SUB.get(int(sub[i]), ""), "", ((i % w) * 7 + (i / w) * 13) % 4)
			_paint_mat(i % w, i / w, int(hy[i]), sig[i])
	_repaint_flood(sim)
	crowd.queue_redraw()
	cargo.queue_redraw()
	threat_fx.queue_redraw()
	threat_bodies.queue_redraw()
	ivan_node.queue_redraw()
	gauges.queue_redraw()
	if view_level == 2:
		paint_traffic()
	elif overlay_kind != "":
		paint_overlay()

## The overlay (Full Design v3.12, Overlays): "" none, or nutrient, signal, light, zones, traffic.
func set_overlay(kind: String) -> void:
	overlay_kind = kind
	paint_overlay()

func paint_overlay() -> void:
	var rgba: PackedByteArray = Sim.ensure().overlay(overlay_kind) if overlay_kind != "" else PackedByteArray()
	overlay_sprite.visible = view_level < 2 and rgba.size() == w * h * 4
	if not overlay_sprite.visible:
		return
	var img := Image.create_from_data(w, h, false, Image.FORMAT_RGBA8, rgba)
	if overlay_sprite.texture == null or overlay_sprite.texture.get_width() != w:
		overlay_sprite.texture = ImageTexture.create_from_image(img)
	else:
		(overlay_sprite.texture as ImageTexture).update(img)

## The Stack view (Full Design v3.11): the stratum as a circuit board at one pixel a tile, drawn by the core
## (render::stack_view) — the Mat dim, traffic as light, a jam (a full tile with cargo waiting to enter it) Chalk
## white, consumers Acid, markets Hot Pink — in place of the tiles, which at three pixels a tile would only sample
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
	for layer in [substrate, matlayer, props, threat_fx, buildings, cargo, crowd, threat_bodies]:
		layer.visible = not stack
	if stack:
		paint_traffic()
	if bind_sim:
		paint_overlay()
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

## The handoff's bypass (Full Design v3.18): dashed in Ooze Glow tile by tile while proposed, solid once confirmed.
func _draw_proposal(sim: RoseSim) -> void:
	var p: Dictionary = sim.proposal()
	var tiles: PackedInt32Array = p.tiles
	var w: int = sim.width()
	for k in tiles.size():
		var i := tiles[k]
		var r := Rect2((i % w) * TILE, (i / w) * TILE, TILE, TILE)
		if bool(p.confirmed):
			buildings.draw_rect(r.grow(-10), Color(UiTheme.OOZE_GLOW, 0.8))
		else:
			for d in 4:
				buildings.draw_rect(Rect2(r.position.x + 2 + d * 8, r.position.y + 15, 4, 2), UiTheme.OOZE_GLOW)
				buildings.draw_rect(Rect2(r.position.x + 15, r.position.y + 2 + d * 8, 2, 4), UiTheme.OOZE_GLOW)

## The Mat's own structures (Full Design v3.18, Organs): drawn as their organ, never with a footprint outline.
const OWN_ORGANS := ["knot", "sphincter", "maw", "gland"]

## An organ's frame: the strip's first frame (the strip's width over four).
func _organ_frame(art: String) -> Texture2D:
	var st := Assets.organ_state(art)
	var strip := Assets.texture(art, st)
	if strip == null:
		return null
	return Assets.frame(art, strip.get_width() / 4, strip.get_height(), 0, st)

## The organs out of a housing and not the Mat's own: each at its Node, its sprite's bottom on the tile's bottom; a
## Lume Bulb its rule has darkened dimmed.
func _draw_organs(sim: RoseSim) -> void:
	for o in sim.organs():
		if bool(o.housed) or String(o.id) in OWN_ORGANS:
			continue
		var tex := _organ_frame(String(o.art))
		if tex == null:
			continue
		var dst := Rect2(int(o.x) * TILE, int(o.y) * TILE + TILE - tex.get_height(), tex.get_width(), tex.get_height())
		buildings.draw_texture_rect(tex, dst, false, Color(0.45, 0.45, 0.55) if bool(o.dark) else Color.WHITE)

## Structures stand on their footprint: the sprite's bottom edge on the footprint's bottom row, its width the
## footprint's. A ghost is half-transparent with its build progress as a bar; a damaged one shows its HP.
func _draw_buildings() -> void:
	if not bind_sim:
		return
	var sim: RoseSim = Sim.ensure()
	_draw_zones(sim)
	_draw_organs(sim)
	_draw_proposal(sim)
	for st in sim.structures():
		var own := String(st.def) in OWN_ORGANS
		var tex: Texture2D = _organ_frame(String(st.art)) if own else Assets.texture(String(st.art), "t%d" % int(st.tier))
		if tex == null:
			continue
		var fw := int(st.w) * TILE
		var fh := int(st.h) * TILE
		var dst := Rect2(int(st.x) * TILE, int(st.y) * TILE + fh - tex.get_height(), tex.get_width(), tex.get_height())
		var built := bool(st.built)
		buildings.draw_texture_rect(tex, dst, false, Color(1, 1, 1, 1.0 if built else 0.45))
		# the footprint's outline on the floor, Cyan for a ghost
		if not own:
			buildings.draw_rect(Rect2(int(st.x) * TILE, int(st.y) * TILE, fw, fh), UiTheme.CYAN if not built else Color(UiTheme.OOZE_GLOW, 0.5), false, 1.0)
		if not built or int(st.upgrading_to) > 0:
			var total := maxi(1, int(st.build_total))
			var frac := clampf(float(st.progress) / total, 0.0, 1.0)
			buildings.draw_rect(Rect2(int(st.x) * TILE, int(st.y) * TILE + fh + 2, fw, 3), UiTheme.SUMP)
			buildings.draw_rect(Rect2(int(st.x) * TILE, int(st.y) * TILE + fh + 2, fw * frac, 3), UiTheme.ACID if bool(st.paid) else UiTheme.HOT_PINK)
		elif int(st.hp) < int(st.hp_max):
			var frac := float(st.hp) / maxf(1.0, float(st.hp_max))
			buildings.draw_rect(Rect2(int(st.x) * TILE, int(st.y) * TILE + fh + 2, fw * frac, 2), UiTheme.HOT_PINK)
	_draw_stockpiles(sim)
	# a tile marked to be cut: a Hot Pink cross, the Build points done as a bar under it
	for k in sim.cuts():
		var r := Rect2(int(k.x) * TILE + 4, int(k.y) * TILE + 4, TILE - 8, TILE - 8)
		buildings.draw_line(r.position, r.end, UiTheme.HOT_PINK, 2.0)
		buildings.draw_line(Vector2(r.end.x, r.position.y), Vector2(r.position.x, r.end.y), UiTheme.HOT_PINK, 2.0)
		if int(k.progress) > 0:
			buildings.draw_rect(Rect2(r.position.x, r.end.y + 2, r.size.x * clampf(float(k.progress) / 8.0, 0.0, 1.0), 2), UiTheme.ACID)

const ZONE_COLORS := {"stockpile": Color8(25, 230, 255), "hab": Color8(246, 255, 46), "quarantine": Color8(255, 46, 151), "kill": Color8(181, 180, 176)}
const TIER_COLORS := {"common": Color8(142, 144, 151), "uncommon": Color8(25, 230, 255), "rare": Color8(255, 46, 151), "blackline": Color8(240, 166, 255), "relic": Color8(242, 239, 228)}

## Every zone's cells outlined in its kind's colour (the Zones overlay fills them).
func _draw_zones(sim: RoseSim) -> void:
	for z in sim.zones():
		var col: Color = ZONE_COLORS.get(String(z.kind), UiTheme.DUST)
		col.a = 0.55
		for c in z.cells:
			buildings.draw_rect(Rect2((int(c) % w) * TILE + 1, (int(c) / w) * TILE + 1, TILE - 2, TILE - 2), col, false, 1.0)

## What a stockpile holds: an item on its cell as a pip in its tier's colour, an item waiting for a hauler as a small
## pip along the store's footprint.
func _draw_stockpiles(sim: RoseSim) -> void:
	var stash: RoseStash = Sim.ensure_stash()
	var stores := {}
	for st in sim.structures():
		stores[int(st.id)] = st
	for room in stash.rooms():
		var waiting := 0
		var anchor: Dictionary = stores.get(int(room.anchor), {})
		for it in stash.room_items(int(room.zone)):
			var col: Color = TIER_COLORS.get(String(it.tier), UiTheme.DUST)
			var cell := int(it.cell)
			var p: Vector2
			if cell >= 0:
				p = Vector2((cell % w) * TILE + 10, (cell / w) * TILE + 10)
			elif not anchor.is_empty():
				p = Vector2(int(anchor.x) * TILE + 2 + waiting * 8, (int(anchor.y) + int(anchor.h)) * TILE - 8)
				waiting += 1
			else:
				continue
			var size := 12.0 if cell >= 0 else 6.0
			buildings.draw_rect(Rect2(p - Vector2.ONE, Vector2(size + 2, size + 2)), UiTheme.VOID)
			buildings.draw_rect(Rect2(p, Vector2(size, size)), col)

## Salt laid on one tile (the Colony's Salt tool).
func paint_salt(x: int, y: int) -> void:
	put_prop(x, y, "tile.salt", "", (y * w + x) % 2)

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
		if view_level == 2:
			# the Stack view carries only the district's name, over its hub
			var label := String(d.name).to_upper()
			districts.draw_set_transform(Vector2(hx, hy) - Vector2(label.length() * 2, 8) * k, 0.0, Vector2(k, k))
			districts.draw_rect(Rect2(-2, -6, label.length() * 4 + 3, 8), Color(UiTheme.SUMP, 0.85))
			districts.draw_string(font, Vector2.ZERO, label, HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.CYAN)
			districts.draw_set_transform(Vector2.ZERO)
			continue
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
	gauges.queue_redraw()

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

# ---------------------------------------------------------------- Pressure (Full Design v3.14)

const BODY_ART := {"scout": "oath.scout", "carrier": "oath.lamp", "sprayer": "oath.sprayer", "flamer": "oath.flamer", "crawler": "rat.crawler", "tamed": "rat.crawler", "envoy": "oath.envoy", "militia": "faction.militia"}
const POND_SCUM := Color8(122, 138, 58)

## The Flood's water: tiles that went under or came back are repainted from the city's substrate.
func _repaint_flood(sim: RoseSim) -> void:
	var now := {}
	for i in sim.flooded_cells():
		now[int(i)] = true
	if now.size() == flooded.size() and now.keys() == flooded.keys():
		return
	var sub: PackedByteArray = sim.substrate()
	for i in now:
		if not flooded.has(i):
			repaint_tile(i % w, i / w, int(sub[i]))
	for i in flooded:
		if not now.has(i):
			repaint_tile(i % w, i / w, int(sub[i]))
	flooded = now

## The Flood's risen water, bleach on the Mat as a pale stain, a Bloom's tissue in Pond Scum, and the white light:
## each lamp's disc of tiles (the rule the threats crate burns by) in hard Vestment White.
func _draw_threat_fx() -> void:
	if not bind_sim or view_level == 2:
		return
	var sim: RoseSim = Sim.ensure()
	# the Flood's water in Flood Light with Cyan ripples, its surface (air above) a Cyan line, so the risen water reads apart from the
	# standing water it falls back to
	if not flooded.is_empty():
		var sub: PackedByteArray = sim.substrate()
		for i in flooded:
			var r := Rect2((i % w) * TILE, (i / w) * TILE, TILE, TILE)
			threat_fx.draw_rect(r, Color(UiTheme.DRAIN_LIGHT, 0.55))
			threat_fx.draw_rect(Rect2(r.position + Vector2(4, 12), Vector2(10, 1)), Color(UiTheme.CYAN, 0.6))
			threat_fx.draw_rect(Rect2(r.position + Vector2(18, 22), Vector2(10, 1)), Color(UiTheme.CYAN, 0.6))
			if i - w >= 0 and int(sub[i - w]) != 5:
				threat_fx.draw_rect(Rect2(r.position, Vector2(TILE, 2)), UiTheme.CYAN)
	for i in sim.bloom_cells():
		threat_fx.draw_rect(Rect2((i % w) * TILE, (i / w) * TILE, TILE, TILE), Color(POND_SCUM, 0.45))
	var lit := {}
	for b in sim.threat_bodies():
		var r := int(b.lamp)
		if r <= 0:
			continue
		for dy in range(-r, r + 1):
			for dx in range(-r, r + 1):
				if dx * dx + dy * dy <= r * r:
					lit[(int(b.y) + dy) * w + int(b.x) + dx] = true
	# the pools: a hard edge where a lit tile meets an unlit one
	for i in lit:
		if i < 0 or i >= w * h:
			continue
		var r := Rect2((i % w) * TILE, (i / w) * TILE, TILE, TILE)
		threat_fx.draw_rect(r, Color(UiTheme.VESTMENT, 0.3))
		if not lit.has(i - w):
			threat_fx.draw_rect(Rect2(r.position, Vector2(TILE, 1)), UiTheme.VESTMENT)
		if not lit.has(i + w):
			threat_fx.draw_rect(Rect2(r.position + Vector2(0, TILE - 1), Vector2(TILE, 1)), UiTheme.VESTMENT)
		if not lit.has(i - 1):
			threat_fx.draw_rect(Rect2(r.position, Vector2(1, TILE)), UiTheme.VESTMENT)
		if not lit.has(i + 1):
			threat_fx.draw_rect(Rect2(r.position + Vector2(TILE - 1, 0), Vector2(1, TILE)), UiTheme.VESTMENT)
	# bleach over the light: a pale wash and drips, so a stain reads as bleach and not as light
	for i in sim.stain_cells():
		var r := Rect2((i % w) * TILE, (i / w) * TILE, TILE, TILE)
		threat_fx.draw_rect(r, Color(UiTheme.VESTMENT, 0.12))
		for k in 4:
			threat_fx.draw_rect(Rect2(r.position + Vector2(3 + k * 7, 4 + (k % 2) * 12), Vector2(3, 6)), Color(UiTheme.VESTMENT, 0.75))

## The bodies on the map, standing on their tile; HP under one that is hurt. A Bloom crawler is tinted Pond Scum, a
## tamed one Cyan.
func _draw_threat_bodies() -> void:
	if not bind_sim or view_level == 2:
		return
	var sim: RoseSim = Sim.ensure()
	var frame := int(Time.get_ticks_msec() / 160) % 6
	var on_tile := {}
	for b in sim.threat_bodies():
		var role := String(b.role)
		# a squad shares tiles: up to four drawn to a tile, 10 px apart across it
		var key := int(b.y) * w + int(b.x)
		var s: int = on_tile.get(key, 0)
		on_tile[key] = s + 1
		if s >= 4:
			continue
		var tex := Assets.texture(String(BODY_ART.get(role, "oath.scout")))
		if tex == null:
			continue
		var fw: int = 48 if tex.get_width() % 48 == 0 and tex.get_height() == 64 else 32
		var frames := maxi(1, tex.get_width() / fw)
		var x := int(b.x) * TILE + TILE / 2 - fw / 2 + s * 10 - 15
		var y := int(b.y) * TILE + TILE - tex.get_height()
		var tint := Color.WHITE
		if role == "crawler":
			tint = Color(0.75, 1.0, 0.6)
		elif role == "tamed":
			tint = Color(0.6, 1.0, 1.0)
		threat_bodies.draw_texture_rect_region(tex, Rect2(x, y, fw, tex.get_height()), Rect2((frame % frames) * fw, 0, fw, tex.get_height()), tint)
		var hp := float(b.hp)
		var mx := maxf(1.0, float(b.max_hp))
		if hp < mx:
			threat_bodies.draw_rect(Rect2(int(b.x) * TILE + 2, int(b.y) * TILE + TILE + 1, TILE - 4, 3), UiTheme.SUMP)
			threat_bodies.draw_rect(Rect2(int(b.x) * TILE + 2, int(b.y) * TILE + TILE + 1, (TILE - 4) * clampf(hp / mx, 0.0, 1.0), 3), UiTheme.HOT_PINK if bool(b.oathkeeper) else UiTheme.ACID)

## Attention at each grate the Mat reaches (Full Design v3.14, Pressure): G and the grate's number over a gauge in
## three steps — a pip lit for each line the zone is past, Acid for the Patrol's, Hot Pink for the Sweep's, Vestment
## White for the Cleansing's — and under it the faction whose street the grate is (v3.15, Economy); while Corpo intel
## runs, the number itself beside the G. The grate's number blinks while an event is down it. Pinned under the grate,
## the same size on screen at every zoom.
func _draw_gauges() -> void:
	if not bind_sim:
		return
	var cam := get_viewport().get_camera_2d()
	var k := 1.0 / (cam.zoom.x if cam != null else 1.0)
	var font := UiTheme.pixel_font()
	var lit := [UiTheme.ACID, UiTheme.HOT_PINK, UiTheme.VESTMENT]
	for z in Sim.ensure().attention():
		if not bool(z.reached):
			continue
		var step := int(z.step)
		var at := Vector2(int(z.x) * TILE + TILE / 2, (int(z.y) + 1) * TILE + 4 * k)
		gauges.draw_set_transform(at, 0.0, Vector2(k, k))
		var who := String(z.get("faction", "")).split(" ")[0].to_upper()
		var wide := maxi(29, who.length() * 4 + 5)
		gauges.draw_rect(Rect2(-wide / 2, -1, wide, 22), Color(UiTheme.SUMP, 0.9))
		var blink: bool = bool(z.down) and int(Time.get_ticks_msec() / 250) % 2 == 0
		var label := String(z.get("label", "G%d" % int(z.number)))
		if bool(z.get("exact", false)):
			label += " %d" % int(round(float(z.attention)))
		gauges.draw_string(font, Vector2(-label.length() * 2, 5), label, HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.VESTMENT if blink else UiTheme.CHALK)
		for p in 3:
			gauges.draw_rect(Rect2(-12 + p * 9, 8, 7, 4), lit[p] if step > p else UiTheme.VOID)
		gauges.draw_string(font, Vector2(-who.length() * 2, 19), who, HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.DUST)
		gauges.draw_set_transform(Vector2.ZERO)

## Each stratum's name: in Dust across a stratum not yet reached (Void until the Mat reaches it), at the left edge
## of one reached (Full Design v3.19, The stack and the breach).
func _draw_strata_names() -> void:
	if not bind_sim or strata_info.is_empty():
		return
	var cam := get_viewport().get_camera_2d()
	var k := 1.0 / (cam.zoom.x if cam != null else 1.0)
	var font := UiTheme.pixel_font()
	for st in strata_info:
		var y0 := int(st.ceiling) * TILE
		var name := "S%d  %s" % [int(st.n), String(st.name).to_upper()]
		if bool(st.reached):
			strata_names.draw_set_transform(Vector2(4 * k, y0 + 4 * k), 0.0, Vector2(k, k))
			strata_names.draw_string(font, Vector2.ZERO + Vector2(0, 6), name, HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.DUST)
		else:
			var rows := int(st.rows) * TILE
			strata_names.draw_set_transform(Vector2(w * TILE / 2.0, y0 + rows / 2.0), 0.0, Vector2(k, k) * 2.0)
			strata_names.draw_string(font, Vector2(-name.length() * 2, 3), name, HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.DUST)
			strata_names.draw_string(font, Vector2(-38, 13), "NOT YET REACHED", HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.DUST)
	strata_names.draw_set_transform(Vector2.ZERO)

## Ivan: a 48 x 64 figure standing on his tile, his Authority state the frame (ivan.idle's four), Ooze Glow where he is
## going, his HP under him when hurt; nothing once he has fallen.
func _draw_ivan() -> void:
	if not bind_sim or view_level == 2:
		return
	var v: Dictionary = Sim.ensure().ivan()
	if v.is_empty() or bool(v.fell):
		return
	var tex := Assets.texture("ivan.idle", "a%d" % clampi(int(v.state), 1, 4))
	var x := int(v.x) * TILE + TILE / 2 - 24
	var y := int(v.y) * TILE + TILE - 64
	if tex != null:
		var frames := maxi(1, tex.get_width() / 48)
		var f := int(Time.get_ticks_msec() / 250) % frames
		ivan_node.draw_texture_rect_region(tex, Rect2(x, y, 48, 64), Rect2(f * 48, 0, 48, 64))
	if int(v.goal_x) >= 0:
		var g := Vector2(int(v.goal_x) * TILE + TILE / 2, int(v.goal_y) * TILE + TILE / 2)
		ivan_node.draw_dashed_line(Vector2(int(v.x) * TILE + TILE / 2, int(v.y) * TILE + TILE / 2), g, UiTheme.OOZE_GLOW, 2.0, 6.0)
		ivan_node.draw_rect(Rect2(g - Vector2(6, 6), Vector2(12, 12)), UiTheme.OOZE_GLOW, false, 2.0)
	if float(v.hp) < float(v.hp_max):
		ivan_node.draw_rect(Rect2(int(v.x) * TILE + 2, int(v.y) * TILE + TILE + 1, TILE - 4, 3), UiTheme.SUMP)
		ivan_node.draw_rect(Rect2(int(v.x) * TILE + 2, int(v.y) * TILE + TILE + 1, (TILE - 4) * clampf(float(v.hp) / float(v.hp_max), 0.0, 1.0), 3), UiTheme.OOZE_GLOW)
