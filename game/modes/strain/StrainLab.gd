extends ScreenBase
## Strain Lab: the ten genes, drift arrows from the feed plan, organ thresholds, the Strainbook.

const GENES := ["VIGOR", "BRANCHING", "THRIFT", "APPETITE", "LUME", "TOXICITY", "CHROME AFF.", "FERAL DRIVE", "CONDUCTANCE", "PLASTICITY"]
const ORGANS := [["mat.organ.bulb", "LUME BULB", "Lume > 0.6"], ["mat.organ.lung", "FILTER LUNG", "Thrift > 0.55, by water"], ["mat.organ.sphincter", "SPHINCTER", "Chrome > 0.5, chokepoint"], ["mat.organ.sac", "SPORE SAC", "Feral > 0.7"]]

func _ready() -> void:
	screen_title = "STRAIN LAB"
	super._ready()

func _build_content(c: Control) -> void:
	var genes := make_frame(c, Vector2(0, 0), Vector2(300, 150))
	make_label(genes, "GENOME   strain: Sump Heart   class: none yet", Vector2(6, 4), UiTheme.CYAN)
	var y := 20
	for i in GENES.size():
		var v := 0.5 + 0.3 * sin(i * 1.3)
		make_gauge(genes, "ui.gauge.ooze", Vector2(6, y), v, GENES[i])
		y += 12
	var organs := make_frame(c, Vector2(306, 0), Vector2(326, 150))
	make_label(organs, "ORGANS  grow by themselves at a Node when the gene crosses the line", Vector2(6, 4), UiTheme.CYAN)
	var oy := 20
	for o in ORGANS:
		make_slot(organs, o[0], Vector2(6, oy), 32, 32, 4.0, "open" if o[0] == "mat.organ.sphincter" else "")
		make_label(organs, o[1], Vector2(44, oy + 4), UiTheme.CHALK)
		make_label(organs, o[2], Vector2(44, oy + 16), UiTheme.DUST)
		oy += 32
	var plan := make_frame(c, Vector2(0, 156), Vector2(300, 134))
	make_label(plan, "FEED PLAN  per Pit: tier order and quota", Vector2(6, 4), UiTheme.CYAN)
	make_label(plan, "Heart Pit     Junk > Street        keep 40   drift -> Thrift, Lume", Vector2(6, 20), UiTheme.CHALK)
	make_label(plan, "East Pit      Street > Corpo       keep 20   drift -> Vigor, Conductance", Vector2(6, 32), UiTheme.CHALK)
	make_label(plan, "Never fed: Blackline, Relic (Vaulted)", Vector2(6, 44), UiTheme.DUST)
	var book := make_frame(c, Vector2(306, 156), Vector2(326, 134))
	make_label(book, "STRAINBOOK   2 / 37", Vector2(6, 4), UiTheme.CYAN)
	make_label(book, "Lume Bulb - seen day 2   -   Gene reflected off 1.0 - seen day 5", Vector2(6, 20), UiTheme.DUST)
