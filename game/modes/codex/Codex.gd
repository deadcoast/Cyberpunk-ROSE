extends ScreenBase
## Codex: Scrapdex grid of every item (silhouetted until seen), Strainbook, Memorial, Trophies, Lore.

var items: RoseItems

func _ready() -> void:
	screen_title = "CODEX"
	super._ready()

func _build_content(c: Control) -> void:
	items = RoseItems.new()
	var x := 0
	for tab in ["SCRAPDEX", "STRAINBOOK", "CHARTER", "MEMORIAL", "TROPHIES", "LORE"]:
		make_button(c, tab, Vector2(x, 0))
		x += 74
	var n: int = items.count()
	make_label(c, "SCRAPDEX  %d / %d seen" % [8, n], Vector2(0, 24), UiTheme.CYAN)
	var grid := Control.new()
	grid.position = Vector2(0, 40)
	c.add_child(grid)
	var cols := 19
	for i in n:
		var d: Dictionary = items.get(i)
		var s := make_slot(grid, icon_for(d.id), Vector2((i % cols) * 33, (i / cols) * 33))
		if i >= 8:
			s.modulate = Color(0.18, 0.18, 0.22)
