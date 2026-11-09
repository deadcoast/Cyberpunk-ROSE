extends ScreenBase
## Strain Lab (Full Design, UI and screens: Strain Lab): the colony strain's ten genes as they stand (RoseSim.genes)
## and the feed plan that steers them — every Pit's quota and its tier ceiling, set on the Pit's card in the Colony
## (v3.11, Quotas). The genome drifts as feed lands at a Pit. Organs, strain classes, the lineage and the Strainbook
## are PAC-080's (Phase E); until then their panels name the thresholds Pass 2 gives and nothing else.

const GENES := ["VIGOR", "BRANCHING", "THRIFT", "APPETITE", "LUME", "TOXICITY", "CHROME AFF.", "FERAL DRIVE", "CONDUCTANCE", "PLASTICITY"]
const ORGANS := [["mat.organ.bulb", "LUME BULB", "Lume > 0.6"], ["mat.organ.lung", "FILTER LUNG", "Thrift > 0.55, by water"], ["mat.organ.sphincter", "SPHINCTER", "Chrome > 0.5, chokepoint"], ["mat.organ.sac", "SPORE SAC", "Feral > 0.7"]]

func _ready() -> void:
	screen_title = "STRAIN LAB"
	super._ready()

func _build_content(c: Control) -> void:
	var sim: RoseSim = Sim.ensure()
	var genes := make_frame(c, Vector2(0, 0), Vector2(300, 150))
	make_label(genes, "GENOME   the colony strain, read-only", Vector2(6, 4), UiTheme.CYAN)
	var g: PackedFloat32Array = sim.genes()
	var y := 20
	for i in GENES.size():
		var v: float = g[i] if i < g.size() else 0.0
		make_gauge(genes, "ui.gauge.ooze", Vector2(6, y), v, "%s %.2f" % [GENES[i], v])
		y += 12
	var organs := make_frame(c, Vector2(306, 0), Vector2(326, 150))
	make_label(organs, "ORGANS   grow at a Node when the gene crosses the line (PAC-080)", Vector2(6, 4), UiTheme.CYAN)
	var oy := 20
	for o in ORGANS:
		make_slot(organs, o[0], Vector2(6, oy), 32, 32, 4.0, "open" if o[0] == "mat.organ.sphincter" else "")
		make_label(organs, o[1], Vector2(44, oy + 4), UiTheme.CHALK)
		make_label(organs, o[2], Vector2(44, oy + 16), UiTheme.DUST)
		oy += 32
	# the feed plan: every Pit's quota, the Heart's first
	var plan := make_frame(c, Vector2(0, 156), Vector2(300, 150))
	make_label(plan, "FEED PLAN   each Pit's quota, set on its card in the Colony", Vector2(6, 4), UiTheme.CYAN)
	var py := 20
	var k := 0
	for st in sim.structures():
		if not (String(st.def) in ["heart", "pit"]) or not bool(st.built):
			continue
		var q: Dictionary = sim.quota(int(st.id))
		if q.is_empty():
			continue
		var keep := "keep %.1f" % float(q.target) if float(q.target) > 0.0 else "no quota"
		var name := "HEART" if String(st.def) == "heart" else "PIT %d" % k
		k += 1
		make_label(plan, "%s   %s   to %s   %s   stock %.2f%s" % [name, keep, String(q.ceiling_name).to_upper(), String(q.from), maxf(0.0, float(q.stock)), "" if bool(q.reached) else "   NOT ON THE MAT"], Vector2(6, py), UiTheme.CHALK if float(q.target) > 0.0 else UiTheme.DUST)
		py += 11
		if py > 130:
			break
	make_label(plan, "a pull takes Chrome and Relics, the lowest tier first, never pinned", Vector2(6, 136), UiTheme.DUST)
	var book := make_frame(c, Vector2(306, 156), Vector2(326, 150))
	make_label(book, "STRAINBOOK   classes, lineage, notable mutations: PAC-080", Vector2(6, 4), UiTheme.CYAN)
