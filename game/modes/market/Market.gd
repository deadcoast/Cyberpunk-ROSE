extends ScreenBase
## The Market (Full Design v3.16, UI and screens: Market; Economy). One faction at a time, read from RoseSim.market:
## a tab per trading faction with the grates of its street (lit where the Mat reaches one) and the surface's events;
## the standing line (reputation pips, favour toward the next step, the tier open and what the next asks, the lever,
## the rival, the services running); left, today's stock tier by tier with ORDER on the picked entry, then the
## market's queue and this faction's orders; right, the day's demand for the seven categories a stall buys with the
## buy-list value ticked and the glut marked, then the offers (ACCEPT) and the links in hand (SEND). Every number is
## the economy crate's; a recover's body laid as it is taken joins the Board's corpses (Sim.lay_bodies).

const SLOT_W := 48
const SLOT_H := 58
const BAR_W := 120
const SERVICE_TAG := {"protection": "PROT", "intel": "INTEL", "convoy": "CONVOY", "timings": "TIMES", "hire": "HIRE"}

var faction := -1
var picked_entry := -1
var picked_offer := -1
var m: Dictionary = {}
var tabs: Control
var standing: Panel
var stock_frame: Panel
var queue_frame: Panel
var demand_frame: Panel
var contracts_frame: Panel
var status := ""

func _ready() -> void:
	screen_title = "MARKET"
	super._ready()

func _build_content(c: Control) -> void:
	tabs = Control.new()
	c.add_child(tabs)
	standing = make_frame(c, Vector2(0, 16), Vector2(632, 30))
	stock_frame = make_frame(c, Vector2(0, 50), Vector2(300, 160))
	queue_frame = make_frame(c, Vector2(0, 214), Vector2(300, 92))
	demand_frame = make_frame(c, Vector2(306, 50), Vector2(326, 104))
	contracts_frame = make_frame(c, Vector2(306, 158), Vector2(326, 148))
	# the first faction whose street the Mat reaches
	var first: Dictionary = Sim.ensure().market(Sim.ensure_people(), 0)
	for t in first.get("factions", []):
		if bool(t.reached):
			faction = int(t.index)
			break
	if faction < 0:
		faction = 0
	_redraw()

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		var k: int = event.keycode - KEY_1
		var list: Array = m.get("factions", [])
		if k >= 0 and k < list.size():
			_tab(int(list[k].index))
			get_viewport().set_input_as_handled()

func _tab(f: int) -> void:
	faction = f
	picked_entry = -1
	picked_offer = -1
	status = ""
	_redraw()

func _redraw() -> void:
	m = Sim.ensure().market(Sim.ensure_people(), faction)
	if m.is_empty():
		return
	_draw_tabs()
	_draw_standing()
	_draw_stock()
	_draw_queue()
	_draw_demand()
	_draw_contracts()

func _clear(n: Node) -> void:
	for ch in n.get_children():
		ch.queue_free()

# ---------------------------------------------------------------- tabs and the standing line
func _draw_tabs() -> void:
	_clear(tabs)
	var x := 0
	var i := 1
	for t in m.get("factions", []):
		var f := int(t.index)
		var label := "%d %s %s" % [i, String(t.name).split(" ")[0].to_upper(), String(t.grates)]
		var b := make_button(tabs, label, Vector2(x, 0), func(): _tab(f))
		var lit := bool(t.reached)
		b.add_theme_color_override("font_color", UiTheme.CYAN if f == faction else (UiTheme.CHALK if lit else UiTheme.DUST))
		if not lit:
			b.modulate = Color(0.7, 0.7, 0.76)
		x += label.length() * 4 + 16
		i += 1
	var ev: PackedStringArray = m.get("events", PackedStringArray())
	var right := "LEDGER %d E" % int(m.eddies)
	make_label(tabs, right, Vector2(632 - right.length() * 4, 4), UiTheme.CYAN)
	if ev.size() > 0:
		var line := "TOPSIDE: " + ", ".join(ev).to_upper()
		make_label(tabs, line, Vector2(632 - right.length() * 4 - 12 - line.length() * 4, 4), UiTheme.ACID)

func _draw_standing() -> void:
	_clear(standing)
	var rep := int(m.rep)
	make_label(standing, String(m.name).to_upper(), Vector2(6, 4), UiTheme.CYAN)
	var x := 6 + String(m.name).length() * 4 + 10
	make_label(standing, "REP", Vector2(x, 4), UiTheme.DUST)
	for k in 7:
		var v := k - 3
		var pip := ColorRect.new()
		pip.position = Vector2(x + 16 + k * 8, 5)
		pip.size = Vector2(6, 5)
		pip.color = Color(UiTheme.DUST, 0.3)
		if v == rep:
			pip.color = UiTheme.ACID if v > 0 else (UiTheme.HOT_PINK if v < 0 else UiTheme.CHALK)
		elif v == 0:
			pip.color = UiTheme.DRAIN_LIGHT
		standing.add_child(pip)
	x += 16 + 7 * 8 + 4
	make_label(standing, "%+d" % rep, Vector2(x, 4), UiTheme.CHALK)
	x += 18
	# favour toward the next step
	var step := maxi(1, int(m.step))
	var fav := int(m.favour)
	var track := ColorRect.new()
	track.position = Vector2(x, 5)
	track.size = Vector2(60, 5)
	track.color = Color(UiTheme.DUST, 0.3)
	standing.add_child(track)
	var fill := ColorRect.new()
	fill.size = Vector2(round(60.0 * clampf(absf(fav) / float(step), 0.0, 1.0)), 5)
	fill.position = Vector2(x, 5)
	fill.color = UiTheme.ACID if fav >= 0 else UiTheme.HOT_PINK
	standing.add_child(fill)
	make_label(standing, "FAVOUR %d/%d" % [fav, step], Vector2(x + 64, 4), UiTheme.DUST)
	x += 64 + 52
	var tier := int(m.tier)
	var tline := "CLOSED: NO STALL TRADES WITH YOU" if bool(m.closed) else "TIER %d" % tier
	var nx: Dictionary = m.get("next", {})
	if not nx.is_empty() and not bool(m.closed):
		tline += "   TIER %d AT REP %+d AND %d E TRADED: %d" % [int(nx.tier), int(nx.rep), int(nx.volume), int(m.traded)]
	make_label(standing, tline, Vector2(x, 4), UiTheme.HOT_PINK if bool(m.closed) else UiTheme.CHALK)
	# line two: where, the lever, the rival, the services running
	var where := ("AT " + String(m.market_name).to_upper()) if int(m.market) >= 0 else String(m.why).to_upper()
	make_label(standing, where, Vector2(6, 16), UiTheme.CYAN if int(m.market) >= 0 else UiTheme.HOT_PINK)
	var sv: Dictionary = m.get("services", {})
	var running := []
	if int(sv.get("intel", 0)) > 0:
		running.append("INTEL %d" % int(sv.intel))
	if int(sv.get("timings", 0)) > 0:
		running.append("TIMINGS %d" % int(sv.timings))
	if int(sv.get("protection", 0)) > 0:
		running.append("PROTECTION")
	if bool(sv.get("convoy", false)):
		running.append("CONVOY")
	var lever := String(m.lever).to_upper()
	var rest := ("LEVER " + lever) if lever != "" else ""
	if String(m.rival) != "":
		rest += "   RIVAL " + String(m.rival).to_upper()
	if running.size() > 0:
		rest += "   RUNNING " + " ".join(running)
	make_label(standing, rest, Vector2(6 + where.length() * 4 + 12, 16), UiTheme.DUST)

# ---------------------------------------------------------------- stock and the order line
func _draw_stock() -> void:
	_clear(stock_frame)
	make_label(stock_frame, "STOCK TODAY", Vector2(6, 4), UiTheme.CYAN)
	var stock: Array = m.get("stock", [])
	var col := 0
	var row := 0
	var picked: Dictionary = {}
	for e in stock:
		var pos := Vector2(6 + col * SLOT_W, 16 + row * SLOT_H)
		var f := make_frame(stock_frame, pos, Vector2(SLOT_W - 4, SLOT_H - 4))
		var open := bool(e.open)
		if String(e.item) != "":
			make_item_tile(f, Vector2(5, 3), 1, 1, String(e.item_tier), String(e.item), true, 32)
		else:
			var tile := ColorRect.new()
			tile.position = Vector2(5, 3)
			tile.size = Vector2(32, 32)
			tile.color = Color(UiTheme.OOZE_GLOW, 0.18)
			f.add_child(tile)
			var tag := String(SERVICE_TAG.get(String(e.service), String(e.service).left(5).to_upper()))
			make_label(f, tag, Vector2(5 + (32 - tag.length() * 4) / 2.0, 17), UiTheme.OOZE_GLOW)
		if open:
			make_label(f, "x%d" % int(e.units), Vector2(4, 37), UiTheme.CHALK)
			make_label(f, "%d E" % int(e.price), Vector2(4, 46), UiTheme.ACID)
		else:
			f.modulate = Color(0.45, 0.45, 0.52)
			make_label(f, "TIER %d" % int(e.tier), Vector2(4, 41), UiTheme.DUST)
		if int(e.entry) == picked_entry:
			picked = e
			_outline(f, UiTheme.CYAN)
		var entry := int(e.entry)
		_hit(f, func():
			picked_entry = entry
			status = ""
			_redraw())
		col += 1
		if col * SLOT_W + SLOT_W > 296:
			col = 0
			row += 1
	if stock.is_empty():
		make_label(stock_frame, "nothing in stock today", Vector2(6, 18), UiTheme.DUST)
	# the picked entry and ORDER
	var y := 146
	if picked.is_empty():
		make_label(stock_frame, "pick an entry to order" if status == "" else status, Vector2(6, y), UiTheme.DUST if status == "" else UiTheme.ACID)
		return
	var line := "%s   %d E A UNIT   %d LEFT" % [String(picked.name).to_upper(), int(picked.price), int(picked.units)]
	if not bool(picked.open):
		line = "%s   AT THEIR TIER %d" % [String(picked.name).to_upper(), int(picked.tier)]
	make_label(stock_frame, line if status == "" else status, Vector2(6, y), UiTheme.CHALK if status == "" else UiTheme.ACID)
	var b := make_button(stock_frame, "ORDER", Vector2(258, y - 4), _order)
	b.disabled = not bool(picked.open) or int(picked.units) <= 0 or int(m.market) < 0 or bool(m.closed)

func _order() -> void:
	var why: String = Sim.ensure().market_order(Sim.ensure_people(), faction, picked_entry, 1)
	status = "ordered: paid at the shift's end" if why == "" else why
	_redraw()

func _draw_queue() -> void:
	_clear(queue_frame)
	make_label(queue_frame, "AT THEIR STALL THIS SHIFT", Vector2(6, 4), UiTheme.CYAN)
	var y := 16
	var total := 0
	var shown := 0
	for q in m.get("queue", []):
		if shown >= 5:
			break
		var quote := int(q.quote)
		var amount := "CONTRACT" if String(q.verb) == "deliver" else (("%d E" % quote) if quote >= 0 else "NO BUYER")
		if String(q.verb) == "sell":
			total += maxi(0, quote)
		var idx := int(q.index)
		var b := make_button(queue_frame, "%s  %s  %s" % [String(q.verb).to_upper(), String(q.name).to_upper(), amount], Vector2(4, y), func():
			Sim.ensure_stash().unqueue(idx)
			status = "pulled back into the Stash"
			_redraw())
		b.flat = true
		y += 11
		shown += 1
	for o in m.get("orders", []):
		if shown >= 6:
			break
		make_label(queue_frame, "ORDER  %s x%d  %d E" % [String(o.name).to_upper(), int(o.units), int(o.each) * int(o.units)], Vector2(6, y + 2), UiTheme.OOZE_GLOW)
		total -= int(o.each) * int(o.units)
		y += 11
		shown += 1
	if shown == 0:
		make_label(queue_frame, "nothing riding here: sell from the Stash, order from the stock", Vector2(6, 18), UiTheme.DUST)
	else:
		make_label(queue_frame, "NET %+d E AT THE SHIFT'S END   CLICK A LINE TO PULL IT BACK" % total, Vector2(6, 80), UiTheme.DUST)

# ---------------------------------------------------------------- demand and glut
func _draw_demand() -> void:
	_clear(demand_frame)
	make_label(demand_frame, "DEMAND TODAY   0.6 - 1.6   TICK: THEIR LIST", Vector2(6, 4), UiTheme.CYAN)
	var y := 18
	for b in m.get("demand", []):
		var today := float(b.today)
		make_label(demand_frame, String(b.category).to_upper(), Vector2(6, y), UiTheme.DUST)
		var track := ColorRect.new()
		track.position = Vector2(64, y + 1)
		track.size = Vector2(BAR_W, 5)
		track.color = Color(UiTheme.DUST, 0.3)
		demand_frame.add_child(track)
		var fill := ColorRect.new()
		fill.position = Vector2(64, y + 1)
		fill.size = Vector2(round(BAR_W * clampf((today - 0.6) / 1.0, 0.0, 1.0)), 5)
		fill.color = UiTheme.ACID if today > 1.0 else UiTheme.DUST
		demand_frame.add_child(fill)
		for t in [float(b.base)]:
			var tick := ColorRect.new()
			tick.position = Vector2(64 + round(BAR_W * clampf((t - 0.6) / 1.0, 0.0, 1.0)), y - 1)
			tick.size = Vector2(1, 9)
			tick.color = UiTheme.VESTMENT
			demand_frame.add_child(tick)
		var line := "%.2f" % today
		# an event pushing one tier: its value beside
		for tier in ["uncommon", "rare"]:
			var v := float(b[tier])
			if absf(v - today) > 0.01:
				line += "  %s %.2f" % [tier.left(3).to_upper(), v]
		make_label(demand_frame, line, Vector2(64 + BAR_W + 6, y), UiTheme.CYAN)
		var glut := int(b.glut)
		if glut > 0:
			make_label(demand_frame, "GLUT -%d%%" % glut, Vector2(270, y), UiTheme.HOT_PINK)
		y += 12

# ---------------------------------------------------------------- contracts
func _draw_contracts() -> void:
	_clear(contracts_frame)
	make_label(contracts_frame, "OFFERS", Vector2(6, 4), UiTheme.CYAN)
	var y := 15
	var offers: Array = m.get("offers", [])
	if offers.is_empty():
		make_label(contracts_frame, "none while the Mat reaches no street of theirs" if int(m.market) < 0 else "none today", Vector2(6, y + 2), UiTheme.DUST)
		y += 12
	for o in offers:
		var id := int(o.id)
		var line := "%d LINKS  %s   %d E  %d SHIFTS  TO DAY %d" % [int(o.links), String(o.ask).to_upper(), int(o.eddies), int(o.shifts), int(o.until)]
		var b := make_button(contracts_frame, line, Vector2(4, y), func():
			picked_offer = id
			status = ""
			_redraw())
		b.flat = true
		if id == picked_offer:
			b.add_theme_color_override("font_color", UiTheme.CYAN)
		y += 11
	var accept := make_button(contracts_frame, "ACCEPT", Vector2(276, 2), _accept)
	accept.disabled = picked_offer < 0 or int(m.market) < 0 or bool(m.closed)
	y += 4
	make_label(contracts_frame, "IN HAND", Vector2(6, y), UiTheme.CYAN)
	y += 12
	var held: Array = m.get("contracts", [])
	if held.is_empty():
		make_label(contracts_frame, "no link of theirs in hand", Vector2(6, y), UiTheme.DUST)
	for c in held:
		var line := "%d/%d  %s  %d/%d   %d E BY SHIFT %d" % [int(c.link), int(c.links), String(c.ask).to_upper(), int(c.got), int(c.n), int(c.eddies), int(c.due)]
		make_label(contracts_frame, line, Vector2(6, y), UiTheme.CHALK)
		if bool(c.sendable):
			var cid := int(c.id)
			var send := make_button(contracts_frame, "SEND", Vector2(286, y - 3), func():
				var n: int = Sim.ensure().market_send(cid, Sim.day)
				status = ("%d sent to their stall" % n) if n > 0 else ("nothing in the Stash fits" if n == 0 else "the Mat reaches no street of theirs")
				_redraw())
			send.disabled = int(m.market) < 0
		y += 11
		if String(c.where) != "":
			make_label(contracts_frame, String(c.where).to_upper(), Vector2(18, y), UiTheme.DUST)
			y += 11
	if status != "" and picked_entry < 0:
		make_label(contracts_frame, status, Vector2(6, 134), UiTheme.ACID)

func _accept() -> void:
	var r: Dictionary = Sim.ensure().market_accept(Sim.ensure_people(), picked_offer, Sim.day)
	if String(r.get("why", "")) != "":
		status = String(r.why)
	else:
		Sim.lay_bodies(r.get("laid", []))
		status = String(r.get("line", ""))
		picked_offer = -1
	picked_entry = -1
	_redraw()

# ---------------------------------------------------------------- helpers
func _outline(f: Control, col: Color) -> void:
	for edge in [[0, 0, f.size.x, 1], [0, f.size.y - 1, f.size.x, 1], [0, 0, 1, f.size.y], [f.size.x - 1, 0, 1, f.size.y]]:
		var line := ColorRect.new()
		line.position = Vector2(edge[0], edge[1])
		line.size = Vector2(edge[2], edge[3])
		line.color = col
		f.add_child(line)

func _hit(f: Control, on_press: Callable) -> void:
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.size = f.size
	b.pressed.connect(on_press)
	f.add_child(b)
