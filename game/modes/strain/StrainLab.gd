extends ScreenBase
## Strain Lab (Full Design v3.18, UI and screens: Strain Lab): the colony strain's ten genes as they stand, the feed
## plan that steers them (every Pit's quota and its tier ceiling, set on the Pit's card in the Colony), the twelve
## organs with their lines, the count grown and the cap, the five classes with their lines and which are held, the
## lineage as each gene's line over the shifts, and the Strainbook's count. Every rule is the mat and progress
## crates' (RoseSim.strain); the screen reads them.

const GENES := ["VIGOR", "BRANCHING", "THRIFT", "APPETITE", "LUME", "TOXICITY", "CHROME AFF.", "FERAL DRIVE", "CONDUCTANCE", "PLASTICITY"]

func _ready() -> void:
	screen_title = "STRAIN LAB"
	super._ready()

func _build_content(c: Control) -> void:
	var sim: RoseSim = Sim.ensure()
	var s: Dictionary = sim.strain()
	var genes := make_frame(c, Vector2(0, 0), Vector2(300, 150))
	make_label(genes, "GENOME   the colony strain, read-only", Vector2(6, 4), UiTheme.CYAN)
	var g: PackedFloat32Array = sim.genes()
	var y := 20
	for i in GENES.size():
		var v: float = g[i] if i < g.size() else 0.0
		make_gauge(genes, "ui.gauge.ooze", Vector2(6, y), v, "%s %.2f" % [GENES[i], v])
		y += 12
	_organs(c, s)
	_plan(c, sim)
	_classes(c, s)

## The twelve organs, three columns of four: the organ, its line, how many the strain holds of its cap.
func _organs(c: Control, s: Dictionary) -> void:
	var f := make_frame(c, Vector2(306, 0), Vector2(326, 150))
	var organs: Array = s.get("organs", [])
	var grown := 0
	for o in organs:
		grown += int(o.count)
	make_label(f, "ORGANS   %d grown   they grow at a Node past their line" % grown, Vector2(6, 4), UiTheme.CYAN)
	for k in organs.size():
		var o: Dictionary = organs[k]
		var pos := Vector2(4 + (k % 3) * 107, 16 + (k / 3) * 33)
		var crossed := bool(o.crossed)
		var count := int(o.count)
		var icon := make_slot(f, String(o.art), pos, 32, 32, 4.0 if count > 0 else 0.0, Assets.organ_state(String(o.art)))
		if count == 0:
			icon.modulate = Color(0.4, 0.4, 0.46)
		var name_col: Color = UiTheme.ACID if count > 0 else (UiTheme.CHALK if crossed else UiTheme.DUST)
		make_label(f, String(o.name).to_upper().left(13), pos + Vector2(34, 1), name_col)
		make_label(f, "%s %s %.2f" % [String(o.gene).left(5).to_upper(), ">" , float(o.line)], pos + Vector2(34, 11), UiTheme.CYAN if crossed else UiTheme.DUST)
		make_label(f, "%d OF %d" % [count, int(o.cap)] if count > 0 or crossed else "-", pos + Vector2(34, 21), UiTheme.CHALK if count > 0 else UiTheme.DUST)

## The feed plan: every Pit's quota, the Heart's first.
func _plan(c: Control, sim: RoseSim) -> void:
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

## The classes, the lineage and the Strainbook's count.
func _classes(c: Control, s: Dictionary) -> void:
	var f := make_frame(c, Vector2(306, 156), Vector2(326, 150))
	var classes: Array = s.get("classes", [])
	var held := 0
	for cl in classes:
		if bool(cl.held):
			held += 1
	make_label(f, "CLASSES   %d held of two   STRAINBOOK %d / %d" % [held, int(s.get("book", 0)), int(s.get("book_total", 38))], Vector2(6, 4), UiTheme.CYAN)
	var y := 18
	var held_genes: Array = []
	for cl in classes:
		var genes: PackedStringArray = cl.genes
		var h := bool(cl.held)
		make_label(f, String(cl.name).to_upper(), Vector2(6, y), UiTheme.OOZE_GLOW if h else UiTheme.CHALK)
		make_label(f, "%s + %s > %.2f" % [genes[0].to_upper(), genes[1].to_upper(), float(cl.line)], Vector2(86, y), UiTheme.CYAN if h else UiTheme.DUST)
		make_label(f, "HELD" if h else "", Vector2(290, y), UiTheme.ACID)
		if h:
			for gn in genes:
				held_genes.append(String(gn).to_upper())
		y += 11
	# the lineage: each gene's line over the shifts, the held classes' genes in Ooze Glow
	var box := Rect2(Vector2(6, 78), Vector2(314, 66))
	var bed := ColorRect.new()
	bed.color = UiTheme.VOID
	bed.position = box.position
	bed.size = box.size
	f.add_child(bed)
	make_label(f, "LINEAGE", box.position + Vector2(3, 1), UiTheme.DUST)
	var rows: Array = s.get("lineage", [])
	for gi in rows.size():
		var vals: PackedFloat32Array = rows[gi]
		if vals.size() < 2:
			continue
		var line := Line2D.new()
		line.width = 1.0
		line.antialiased = false
		var name := String(["VIGOR", "BRANCHING", "THRIFT", "APPETITE", "LUME", "TOXICITY", "CHROME AFFINITY", "FERAL DRIVE", "CONDUCTANCE", "PLASTICITY"][gi])
		line.default_color = UiTheme.OOZE_GLOW if held_genes.has(name) else UiTheme.DUST
		var n := vals.size()
		for k in n:
			var x := box.position.x + 2 + (box.size.x - 4) * float(k) / float(n - 1)
			var yy := box.position.y + box.size.y - 2 - (box.size.y - 12) * clampf(vals[k], 0.0, 1.0)
			line.add_point(Vector2(roundf(x), roundf(yy)))
		f.add_child(line)
	if rows.is_empty() or (rows[0] as PackedFloat32Array).size() < 2:
		make_label(f, "a line a gene from the second shift", box.position + Vector2(60, 28), UiTheme.DUST)
