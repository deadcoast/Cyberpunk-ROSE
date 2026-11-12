extends ScreenBase
## The Stash (Full Design v3.4): a view of the city's one grid (Sim.stash, the inventory crate through RoseStash).
## Filter and sort bar on top; the grid left, every item a tile at its real size, scrolling when the Stash
## outgrows the screen; right column: the item card with the three numbers and the four verbs (Sell, Feed, Craft,
## Keep), the bulk bar that applies Sell or Feed to the filtered view, and the queue panel with the ledger. At the
## shift end (Sim.end_shift, after a run) the queue leaves the Heart as cargo and each item pays or feeds when it
## lands (Full Design v3.11, Cargo); the report shows what landed and what left on the way back. The tabs over the
## grid page between the Heart's grid and each stockpile (v3.12, Stockpiles): a room shows its cells in order, the
## items waiting on the store's footprint for a hauler marked, and every verb works on an item where it lies.

const CELL := 32
const CATEGORIES := ["", "chrome", "part", "organic", "tech", "gear", "contraband", "key", "curio", "relic"]
const TIERS := ["", "common", "uncommon", "rare", "blackline", "relic"]
const CONDS := [101, 20, 50]
const SORTS := ["tier", "value", "weight", "condition", "category"]

var stash: RoseStash
var items: RoseItems
var grid_frame: Panel
var scroll: ScrollContainer
var cells: Control
var card: Panel
var bulk: Panel
var queue_panel: Panel
var selected := -1
var haul_note := ""
var filter_cat := 0
var filter_tier := 0
var filter_cond := 0
var filter_unid := false
var sort_key := 0
var market := 0
var pit := 0
var cat_buttons: Array[Button] = []
var tier_button: Button
var cond_button: Button
var unid_button: Button
var sort_button: Button
var status: Label
var room := -1            # the zone of the stockpile shown; -1 the Heart's grid, -2 the Heart's floor
var tabs: Control

func _ready() -> void:
	screen_title = "STASH"
	super._ready()

func _build_content(c: Control) -> void:
	stash = Sim.ensure_stash()
	items = RoseItems.new()
	# ---- the filter row over the grid
	var x := 0
	for i in CATEGORIES.size():
		var label := "ALL" if CATEGORIES[i] == "" else String(CATEGORIES[i]).left(5).to_upper()
		var b := make_button(c, label, Vector2(x, 0), func(): _set_cat(i))
		cat_buttons.append(b)
		x += label.length() * 4 + 16
	# ---- the sort cluster over the right column
	tier_button = make_button(c, "", Vector2(398, 0), func():
		filter_tier = (filter_tier + 1) % TIERS.size()
		_redraw())
	unid_button = make_button(c, "UNID", Vector2(470, 0), func():
		filter_unid = not filter_unid
		_redraw())
	cond_button = make_button(c, "", Vector2(510, 0), func():
		filter_cond = (filter_cond + 1) % CONDS.size()
		_redraw())
	sort_button = make_button(c, "", Vector2(574, 0), func():
		sort_key = (sort_key + 1) % SORTS.size()
		stash.sort_by(SORTS[sort_key])
		_redraw())
	# ---- the grid, scrolling
	tabs = Control.new()
	tabs.position = Vector2(0, 19)
	c.add_child(tabs)
	grid_frame = make_frame(c, Vector2(0, 34), Vector2(392, 272))
	scroll = ScrollContainer.new()
	scroll.position = Vector2(4, 4)
	scroll.size = Vector2(384, 264)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	grid_frame.add_child(scroll)
	cells = Control.new()
	scroll.add_child(cells)
	# ---- the right column
	card = make_frame(c, Vector2(398, 20), Vector2(234, 132))
	bulk = make_frame(c, Vector2(398, 156), Vector2(234, 62))
	queue_panel = make_frame(c, Vector2(398, 222), Vector2(234, 84))
	status = make_label(c, "", Vector2(0, 0), UiTheme.DUST)
	status.visible = false
	# a Runner's rig handed over by the Run on extraction
	if Sim.haul_note != "":
		haul_note = Sim.haul_note
		Sim.haul_note = ""
	_redraw()

func _set_cat(i: int) -> void:
	filter_cat = i
	_redraw()

## Does an item pass the filter bar?
func _passes(it: Dictionary) -> bool:
	if CATEGORIES[filter_cat] != "" and String(it.category) != CATEGORIES[filter_cat]:
		return false
	if TIERS[filter_tier] != "" and String(it.tier) != TIERS[filter_tier]:
		return false
	if int(it.condition) >= CONDS[filter_cond]:
		return false
	if filter_unid and bool(it.identified):
		return false
	return true

## The items of the room shown: the Heart's grid, or a stockpile's in its cells' order.
func _view_items() -> Array:
	if room == -2:
		return stash.floor_items()
	return stash.items() if room < 0 else stash.room_items(room)

func _room() -> Dictionary:
	for r in stash.rooms():
		if int(r.zone) == room:
			return r
	return {}

func _draw_tabs() -> void:
	for ch in tabs.get_children():
		ch.queue_free()
	var x := 0
	var heart := make_button(tabs, "HEART %d" % stash.count_grid(), Vector2(x, 0), func():
		room = -1
		selected = -1
		_redraw())
	heart.add_theme_color_override("font_color", UiTheme.ACID if room == -1 else UiTheme.CHALK)
	x += 56
	var on_floor: int = stash.floor_items().size()
	if on_floor > 0 or room == -2:
		var fl := make_button(tabs, "FLOOR %d" % on_floor, Vector2(x, 0), func():
			room = -2
			selected = -1
			_redraw())
		fl.add_theme_color_override("font_color", UiTheme.ACID if room == -2 else UiTheme.HOT_PINK)
		x += 56
	for r in stash.rooms():
		var z := int(r.zone)
		var label := "%s %d/%d" % [String(r.name), int(r.count), int(r.cap)]
		var b := make_button(tabs, label, Vector2(x, 0), func():
			room = z
			selected = -1
			_redraw())
		b.add_theme_color_override("font_color", UiTheme.ACID if room == z else UiTheme.CHALK)
		x += label.length() * 4 + 14
	if stash.rooms().is_empty():
		make_label(tabs, "no stockpiles: paint one against a Rack or Cold Store", Vector2(x + 4, 3), UiTheme.DUST)

func _redraw() -> void:
	for ch in cells.get_children():
		ch.queue_free()
	if room >= 0 and _room().is_empty():
		room = -1
	if room == -2 and stash.floor_items().is_empty():
		room = -1
	_draw_tabs()
	var w: int = stash.width()
	var h: int = stash.height()
	if room >= 0:
		w = 8
		h = maxi(1, int(ceil(float(int(_room().cap)) / 8.0)))
	elif room == -2:
		w = 8
		h = maxi(1, int(ceil(float(stash.floor_items().size()) / 8.0)))
	cells.custom_minimum_size = Vector2(w * CELL, h * CELL)
	cells.size = cells.custom_minimum_size
	for y in h:
		for x in w:
			var cell := ColorRect.new()
			cell.color = Color8(23, 26, 38)
			cell.position = Vector2(x * CELL, y * CELL)
			cell.size = Vector2(CELL - 1, CELL - 1)
			cells.add_child(cell)
	var shown := 0
	for it in _view_items():
		var hnd: int = it.handle
		var tile := make_item_tile(cells, Vector2(it.x * CELL, it.y * CELL), int(it.w), int(it.h), String(it.tier), String(it.id), bool(it.identified), CELL)
		var passes := _passes(it)
		if passes:
			shown += 1
		else:
			tile.modulate = Color(0.45, 0.45, 0.5, 0.6)
		if int(it.get("cell", 0)) == -1:
			# on the store's footprint, waiting for a hauler to carry it to its cell
			var wait := ColorRect.new()
			wait.color = UiTheme.OOZE_GLOW
			wait.position = Vector2(1, tile.size.y - 5)
			wait.size = Vector2(tile.size.x - 3, 3)
			tile.add_child(wait)
		if bool(it.pinned):
			var pin := ColorRect.new()
			pin.color = UiTheme.ACID
			pin.position = Vector2(tile.size.x - 5, 1)
			pin.size = Vector2(4, 4)
			tile.add_child(pin)
		if hnd == selected:
			var sel := ColorRect.new()
			sel.color = Color(UiTheme.CYAN, 0.25)
			sel.size = tile.size - Vector2(1, 1)
			tile.add_child(sel)
		var btn := Button.new()
		btn.flat = true
		btn.size = tile.size
		btn.focus_mode = Control.FOCUS_NONE
		btn.pressed.connect(func(): _select(hnd))
		tile.add_child(btn)
	# the bar
	for i in cat_buttons.size():
		cat_buttons[i].modulate = Color.WHITE if i == filter_cat else Color(0.6, 0.6, 0.66)
	tier_button.text = "TIER " + ("ALL" if TIERS[filter_tier] == "" else String(TIERS[filter_tier]).left(3).to_upper())
	unid_button.modulate = Color.WHITE if filter_unid else Color(0.6, 0.6, 0.66)
	cond_button.text = "COND ALL" if CONDS[filter_cond] > 100 else "COND <%d" % CONDS[filter_cond]
	sort_button.text = "SORT " + String(SORTS[sort_key]).left(4).to_upper()
	_draw_card()
	_draw_bulk(shown)
	_draw_queue()

func _item(handle: int) -> Dictionary:
	for it in _view_items():
		if int(it.handle) == handle:
			return it
	return {}

func _select(handle: int) -> void:
	selected = handle
	_redraw()

func _draw_card() -> void:
	for ch in card.get_children():
		ch.queue_free()
	var it := _item(selected)
	if it.is_empty():
		if room == -2:
			make_label(card, "THE HEART'S FLOOR   %d items" % stash.floor_items().size(), Vector2(6, 4), UiTheme.HOT_PINK)
			make_label(card, "what the grid has no room for: a haul's overflow,", Vector2(6, 18), UiTheme.DUST)
			make_label(card, "a product home to a full grid", Vector2(6, 30), UiTheme.DUST)
			make_label(card, "it tries the grid again at shift end; a stockpile", Vector2(6, 42), UiTheme.DUST)
			make_label(card, "at priority 1-2 takes it; sell or feed it here", Vector2(6, 54), UiTheme.DUST)
			return
		if room >= 0:
			var r := _room()
			make_label(card, "%s   %d of %d cells" % [String(r.name), int(r.count), int(r.cap)], Vector2(6, 4), UiTheme.CYAN)
			var cats: PackedStringArray = r.categories
			make_label(card, "takes %s%s%s" % ["any category" if cats.is_empty() else ", ".join(cats), ("   tier %d+" % int(r.min_tier)) if int(r.min_tier) > 0 else "", ("   condition %d+" % int(r.min_condition)) if int(r.min_condition) > 0 else ""], Vector2(6, 18), UiTheme.DUST)
			make_label(card, "priority %d: %s" % [int(r.priority), "draws from the Heart" if int(r.priority) >= 3 else "takes the Heart's overflow"], Vector2(6, 30), UiTheme.DUST)
			make_label(card, "%d waiting on the store for a hauler%s" % [int(r.waiting), "   cold: no spoiling" if bool(r.cold) else ""], Vector2(6, 42), UiTheme.OOZE_GLOW if int(r.waiting) > 0 else UiTheme.DUST)
			make_label(card, "set its filter and priority on its card in the Colony", Vector2(6, 54), UiTheme.DUST)
			return
		make_label(card, "STASH   tier %d   %d items   %d free" % [stash.tier(), stash.count(), stash.free_cells()], Vector2(6, 4), UiTheme.CYAN)
		make_label(card, "click an item for its card", Vector2(6, 18), UiTheme.DUST)
		make_label(card, "filters select the view; bulk acts on it", Vector2(6, 30), UiTheme.DUST)
		make_label(card, "sells and feeds ride the Mat within the minute", Vector2(6, 42), UiTheme.DUST)
		make_label(card, "the grid grows with the Heart's tier", Vector2(6, 54), UiTheme.DUST)
		return
	var d: Dictionary = items.get(int(it.index))
	make_item_tile(card, Vector2(6, 6), 1, 1, String(it.tier), String(it.id), bool(it.identified), 32)
	make_label(card, (String(it.name) if bool(it.identified) else "UNIDENTIFIED").to_upper(), Vector2(44, 6), UiTheme.CYAN)
	make_label(card, "%s %s   %dx%d   %.1f kg" % [String(it.tier), String(it.category), int(it.w), int(it.h), float(it.weight)], Vector2(44, 18), UiTheme.DUST)
	var where := "found in run" if bool(it.found) else "from the city"
	if room >= 0:
		where = ("in " if int(it.cell) >= 0 else "waits at ") + String(_room().name)
	elif room == -2:
		where = "on the Heart's floor"
	if int(it.get("bound", -1)) >= 0:
		where = "a contract's: send from Market"
	make_label(card, "condition %d   %s%s" % [int(it.condition), where, "   PINNED" if bool(it.pinned) else ""], Vector2(44, 30), UiTheme.DUST)
	# the three numbers
	var y := 46
	var can_sell := Sim.markets().size() > 0 and int(d.value) > 0
	var quote := int(stash.quote(selected, market)) if can_sell else 0
	if Sim.markets().size() == 0:
		make_label(card, "SELL   no market: grow the Mat to a grate", Vector2(6, y), UiTheme.DUST)
	elif int(d.value) == 0:
		make_label(card, "SELL   nobody buys this", Vector2(6, y), UiTheme.DUST)
	elif quote < 0:
		make_label(card, "SELL   no stall at %s buys this" % Sim.market_name(market), Vector2(6, y), UiTheme.DUST)
	else:
		make_label(card, "SELL   %d e at %s%s" % [quote, Sim.market_name(market), "  (blind 40%)" if not bool(it.identified) else ""], Vector2(6, y), UiTheme.ACID)
	var feed := float(it.feed)
	if feed > 0.0:
		var cells_ := feed * float(Sim.ensure().cells_per_nutrient())
		make_label(card, "FEED   %.2f nutrient  %d cells of Mat  at %s" % [feed, int(round(cells_)), Sim.pit_name(pit)], Vector2(6, y + 12), UiTheme.OOZE_GLOW)
	else:
		make_label(card, "FEED   the Mat does not eat this", Vector2(6, y + 12), UiTheme.DUST)
	var tags: PackedStringArray = it.get("tags", PackedStringArray())
	var best: Dictionary = stash.best_recipe(selected)
	if best.is_empty():
		make_label(card, "CRAFT  " + ("no open recipe takes " + ", ".join(tags) if tags.size() > 0 else "no craft tag"), Vector2(6, y + 24), UiTheme.DUST)
	else:
		make_label(card, "CRAFT  %s as the %s  (tier %d)" % [String(best.product_name), String(best.slot_tag), int(best.tier)], Vector2(6, y + 24), UiTheme.CHALK)
	if it.has("crafted_by"):
		var from: PackedStringArray = it.get("crafted_from", PackedStringArray())
		make_label(card, ("made by %s from %s" % [String(it.crafted_by), ", ".join(from)]).left(56), Vector2(6, y + 36), UiTheme.OOZE_GLOW)
	elif it.has("stats"):
		var st: Dictionary = it.stats
		var bits := []
		for key in ["dmg", "armour", "noise", "light", "mag"]:
			if int(st[key]) != 0:
				bits.append("%s %d" % [key, int(st[key])])
		make_label(card, "   ".join(bits), Vector2(6, y + 36), UiTheme.DUST)
	# the four verbs, in order
	var hnd := selected
	var sell_b := make_button(card, "SELL", Vector2(6, 104), func():
		if stash.queue_sell(hnd, market, Sim.day):
			selected = -1
			_redraw())
	sell_b.disabled = not can_sell or quote < 0
	var feed_b := make_button(card, "FEED", Vector2(58, 104), func():
		if stash.queue_feed(hnd, pit, Sim.day):
			selected = -1
			_redraw())
	feed_b.disabled = feed <= 0.0
	var craft_b := make_button(card, "CRAFT", Vector2(110, 104), func():
		Sim.bench_slot = it.duplicate()
		Screens.show_screen("bench"))
	craft_b.disabled = best.is_empty()
	make_button(card, "UNPIN" if bool(it.pinned) else "KEEP", Vector2(168, 104), func():
		stash.pin(hnd, not bool(it.pinned))
		_redraw())

## Bulk: the verb applied to every unpinned item the filter bar selects.
func _draw_bulk(shown: int) -> void:
	for ch in bulk.get_children():
		ch.queue_free()
	var handles: PackedInt32Array = stash.bulk_handles(CATEGORIES[filter_cat], TIERS[filter_tier], CONDS[filter_cond], filter_unid)
	if room != -1:
		handles = PackedInt32Array()
		for it in _view_items():
			if _passes(it) and not bool(it.pinned):
				handles.append(int(it.handle))
	var total_e := 0
	var total_n := 0.0
	var feedable := 0
	for h in handles:
		var it := _item(h)
		if it.is_empty():
			continue
		total_e += maxi(0, int(stash.quote(h, market)))
		if float(it.feed) > 0.0:
			total_n += float(it.feed)
			feedable += 1
	make_label(bulk, "BULK   view %d   unpinned %d" % [shown, handles.size()], Vector2(6, 4), UiTheme.CYAN)
	var markets_n := Sim.markets().size()
	var pits_n := Sim.pits().size()
	var sell_all := make_button(bulk, "SELL ALL  %d e" % total_e, Vector2(6, 18), func():
		for h in handles:
			if int(stash.quote(h, market)) >= 0:
				stash.queue_sell(h, market, Sim.day)
		selected = -1
		_redraw())
	sell_all.disabled = handles.size() == 0 or markets_n == 0
	var feed_all := make_button(bulk, "FEED ALL  %d  %.1f n" % [feedable, total_n], Vector2(110, 18), func():
		for h in handles:
			stash.queue_feed(h, pit, Sim.day)
		selected = -1
		_redraw())
	feed_all.disabled = feedable == 0
	if markets_n > 0:
		make_button(bulk, "TO %s" % Sim.market_name(market), Vector2(6, 40), func():
			market = (market + 1) % maxi(1, markets_n)
			_redraw())
	else:
		make_label(bulk, "no market reached", Vector2(6, 44), UiTheme.DUST)
	make_button(bulk, "INTO %s" % Sim.pit_name(pit), Vector2(110, 40), func():
		pit = (pit + 1) % maxi(1, pits_n)
		_redraw())

func _draw_queue() -> void:
	for ch in queue_panel.get_children():
		ch.queue_free()
	make_label(queue_panel, "LEDGER", Vector2(6, 4), UiTheme.CYAN)
	make_label(queue_panel, "%d e" % int(stash.eddies()), Vector2(40, 4), UiTheme.CYAN)
	var q: Array = stash.queued()
	var y := 16
	if haul_note != "":
		make_label(queue_panel, haul_note.left(54), Vector2(6, y), UiTheme.OOZE_GLOW)
		y += 12
	var shown := 0
	var sell_sum := 0
	var feed_sum := 0.0
	for e in q:
		if String(e.verb) == "sell":
			sell_sum += maxi(0, int(e.quote))
		elif String(e.verb) == "feed":
			feed_sum += float(e.nutrient)
		if shown >= (3 if haul_note != "" else 4):
			continue
		var idx := int(e.index)
		var what := (String(e.name) if bool(e.identified) else "unidentified %s" % String(e.tier)).left(18)
		var verb := String(e.verb)
		var amount := "%.2f n" % float(e.nutrient)
		if verb == "sell":
			amount = ("%d e" % int(e.quote)) if int(e.quote) >= 0 else "no buyer"
		elif verb == "deliver":
			amount = "contract"
		var line := "%s  %s  %s" % [verb.to_upper(), what, amount]
		make_label(queue_panel, line, Vector2(6, y), UiTheme.OOZE_GLOW if verb == "feed" else UiTheme.ACID)
		make_button(queue_panel, "BACK", Vector2(192, y - 3), func():
			stash.unqueue(idx)
			_redraw())
		y += 12
		shown += 1
	if q.size() > shown:
		make_label(queue_panel, "+%d more queued" % (q.size() - shown), Vector2(6, y), UiTheme.DUST)
		y += 12
	var foot := ""
	var now: Dictionary = stash.landed()
	if q.size() > 0:
		foot = "%d queued: %d e, %.1f n   leave within the minute" % [q.size(), sell_sum, feed_sum]
		if int(now.riding) > 0:
			foot += "   %d on the Mat" % int(now.riding)
	elif int(now.riding) > 0 or int(now.sold) + int(now.fed) > 0:
		# cargo this shift: what has landed so far and what is still riding
		foot = "this shift: sold %d for %d e, fed %d   %d on the Mat" % [int(now.sold), int(now.earned), int(now.fed), int(now.riding)]
	elif not Sim.shift_report.is_empty():
		var r: Dictionary = Sim.shift_report
		foot = "last shift: sold %d for %d e, fed %d, crafted %d, %d left" % [int(r.sold), int(r.earned), int(r.fed), int(r.get("crafted", 0)), int(r.get("left", 0))]
		if int(r.get("spilled", 0)) > 0:
			foot += ", %d spilled" % int(r.spilled)
		# the roster's first line of the shift (who worked what, a level gained), where the frame has room for it
		# whole; the rest are on the Slimo card
		if Sim.people_lines.size() > 0 and foot.length() + 3 + String(Sim.people_lines[0]).length() <= 54:
			foot += "   " + String(Sim.people_lines[0])
	else:
		foot = "nothing queued"
	make_label(queue_panel, foot.left(54), Vector2(6, 70), UiTheme.DUST)
