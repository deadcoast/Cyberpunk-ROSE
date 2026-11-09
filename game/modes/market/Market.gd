extends ScreenBase
## Seep Market: vendor tabs, stock, sell queue, demand bars, contracts. Phase A: layout from the item table.

const VENDORS := ["GANGS", "NOMADS", "CORPO", "BADGES"]
var items: RoseItems

func _ready() -> void:
	screen_title = "MARKET"
	super._ready()

func _build_content(c: Control) -> void:
	items = RoseItems.new()
	var x := 0
	for v in VENDORS:
		make_button(c, v, Vector2(x, 0))
		x += 70
	make_label(c, "reputation +1   stock tier 2   glut none", Vector2(290, 4), UiTheme.DUST)
	var stock := make_frame(c, Vector2(0, 24), Vector2(300, 160))
	make_label(stock, "STOCK", Vector2(6, 4), UiTheme.CYAN)
	var cat: PackedInt32Array = items.by_category("gear")
	var sx := 6
	var sy := 20
	for k in mini(14, cat.size()):
		var d: Dictionary = items.get(cat[k])
		var f := make_frame(stock, Vector2(sx, sy), Vector2(40, 52))
		make_slot(f, icon_for(d.id), Vector2(4, 4))
		make_label(f, str(d.value), Vector2(4, 38), UiTheme.ACID)
		sx += 42
		if sx > 250:
			sx = 6
			sy += 56
	var demand := make_frame(c, Vector2(306, 24), Vector2(326, 100))
	make_label(demand, "DEMAND BY CATEGORY", Vector2(6, 4), UiTheme.CYAN)
	var dy := 20
	for row in [["CHROME", 0.7], ["PARTS", 0.5], ["ORGANICS", 0.9], ["TECH", 0.4], ["CONTRABAND", 1.0]]:
		make_gauge(demand, "ui.gauge.ooze", Vector2(6, dy), row[1], row[0])
		dy += 12
	var contracts := make_frame(c, Vector2(306, 130), Vector2(326, 160))
	make_label(contracts, "CONTRACTS", Vector2(6, 4), UiTheme.CYAN)
	make_label(contracts, "Deliver 8 Street Chrome at condition 60+   3 shifts   240 eddies  chain 1/3", Vector2(6, 20), UiTheme.CHALK)
	make_label(contracts, "Clear the Bloom at S3-west   5 shifts   rep +1, stock unlock", Vector2(6, 34), UiTheme.CHALK)
	make_label(contracts, "Recover: corpse of 'Hull' in the east drain   2 shifts   his rifle back", Vector2(6, 48), UiTheme.CHALK)
	make_button(contracts, "ACCEPT", Vector2(6, 120))
	var sell := make_frame(c, Vector2(0, 190), Vector2(300, 100))
	make_label(sell, "SELL QUEUE  resolves at shift end", Vector2(6, 4), UiTheme.CYAN)
	make_label(sell, "6 x Junk Chrome  24 eddies   -   1 x Cold Light Cell  18 eddies", Vector2(6, 20), UiTheme.DUST)
