extends ScreenBase
## The Job grid (Full Design v3.12, UI and screens: Job grid): a row a slimo (name, depth, district, the job the
## colony gave it this shift), a column a job, each cell the slimo's priority 0–4 (4 the most wanted, . never) with
## the foreman's mark where the district's stands over it. Click cycles a cell; a drag paints the first cell's new
## value over every cell it crosses. Priorities are the roster's (RosePeople.set_priority); the plan is the colony's.

const ROWS := 14
const CELL := Vector2(38, 16)
const X0 := 252          # the first job column
const Y0 := 32           # the first row

var people: RosePeople
var sim: RoseSim
var rows: Array = []
var jobs: PackedStringArray
var district := -1       # -1 every district
var page := 0
var grid: Control
var head: Control
var paint_value := -1    # the value a drag paints, -1 when no drag
var painted := {}        # "id:job" cells this drag has set

func _ready() -> void:
	screen_title = "JOBS"
	super._ready()

func _build_content(c: Control) -> void:
	people = Sim.ensure_people()
	sim = Sim.ensure()
	jobs = people.jobs()
	head = Control.new()
	c.add_child(head)
	var frame := make_frame(c, Vector2(0, 26), Vector2(632, CELL.y * ROWS + 12))
	grid = Control.new()
	grid.position = Vector2(0, 26)
	grid.size = frame.size
	grid.mouse_filter = Control.MOUSE_FILTER_STOP
	grid.draw.connect(_draw_grid)
	grid.gui_input.connect(_on_grid_input)
	c.add_child(grid)
	_reload()

func _reload() -> void:
	rows = []
	for r in sim.job_grid(people):
		if district < 0 or int(r.district) == district:
			rows.append(r)
	page = clampi(page, 0, maxi(0, (rows.size() - 1) / ROWS))
	_draw_head()
	grid.queue_redraw()

## The district filter, the column heads and the pages.
func _draw_head() -> void:
	for ch in head.get_children():
		ch.queue_free()
	var x := 0
	var all := make_button(head, "ALL", Vector2(x, 0), func():
		district = -1
		_reload())
	all.add_theme_color_override("font_color", UiTheme.ACID if district < 0 else UiTheme.CHALK)
	x += 26
	for d in sim.districts(people):
		var id := int(d.id)
		var b := make_button(head, String(d.name).to_upper(), Vector2(x, 0), func():
			district = id
			_reload())
		b.add_theme_color_override("font_color", UiTheme.ACID if district == id else UiTheme.CHALK)
		x += String(d.name).length() * 4 + 14
	make_label(head, "4 most wanted   1 least   . never   F the foreman's, where it stands over", Vector2(maxi(x + 6, 300), 3), UiTheme.DUST)
	make_label(head, "SLIMO", Vector2(4, 16), UiTheme.CYAN)
	make_label(head, "DISTRICT", Vector2(118, 16), UiTheme.CYAN)
	make_label(head, "THIS SHIFT", Vector2(180, 16), UiTheme.CYAN)
	for j in jobs.size():
		make_label(head, jobs[j].to_upper().left(8), Vector2(X0 + j * (CELL.x + 4) + 2, 16), UiTheme.CYAN)
	var pages := maxi(1, int(ceil(float(rows.size()) / ROWS)))
	if pages > 1:
		make_label(head, "page %d/%d" % [page + 1, pages], Vector2(500, 292), UiTheme.DUST)
		make_button(head, "PREV", Vector2(550, 288), func():
			page = maxi(0, page - 1)
			_reload())
		make_button(head, "NEXT", Vector2(590, 288), func():
			page = mini(pages - 1, page + 1)
			_reload())

func _visible_rows() -> Array:
	return rows.slice(page * ROWS, mini(rows.size(), (page + 1) * ROWS))

func _draw_grid() -> void:
	var font := UiTheme.pixel_font()
	var y := 6.0
	for r in _visible_rows():
		var pri: PackedInt32Array = r.priorities
		var eff: PackedInt32Array = r.effective
		grid.draw_string(font, Vector2(4, y + 11), String(r.name).to_upper().left(22), HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.CHALK)
		grid.draw_string(font, Vector2(96, y + 11), String(r.depth), HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.DUST)
		grid.draw_string(font, Vector2(118, y + 11), String(r.district_name).to_upper().left(14) if int(r.district) >= 0 else "-", HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.DUST)
		var job := String(r.job).to_upper() if String(r.job) != "" else ("-" if String(r.status) == "idle" else String(r.status).to_upper())
		grid.draw_string(font, Vector2(180, y + 11), job.left(16), HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.ACID if String(r.job) != "" else UiTheme.DUST)
		for j in jobs.size():
			var cell := Rect2(X0 + j * (CELL.x + 4), y, CELL.x, CELL.y - 2)
			var own: int = pri[j]
			var over: bool = eff[j] > own
			grid.draw_rect(cell, Color(UiTheme.VOID, 0.9))
			grid.draw_rect(cell, UiTheme.DUST if not over else UiTheme.CYAN, false, 1.0)
			grid.draw_string(font, cell.position + Vector2(6, 10), str(own) if own > 0 else ".", HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.CHALK if own > 0 else UiTheme.DUST)
			if over:
				grid.draw_string(font, cell.position + Vector2(20, 10), "F%d" % eff[j], HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.CYAN)
		y += CELL.y
	if rows.is_empty():
		grid.draw_string(font, Vector2(4, 16), "no slimo in this district", HORIZONTAL_ALIGNMENT_LEFT, -1, 6, UiTheme.DUST)

## The cell under a point of the grid: [row index in the page, job], or [] for none.
func _cell_at(p: Vector2) -> Array:
	var ri := int(floor((p.y - 6.0) / CELL.y))
	var j := int(floor((p.x - X0) / (CELL.x + 4)))
	if ri < 0 or ri >= _visible_rows().size() or j < 0 or j >= jobs.size() or p.x - X0 - j * (CELL.x + 4) > CELL.x:
		return []
	return [ri, j]

func _set_cell(at: Array, value: int) -> void:
	var r: Dictionary = _visible_rows()[at[0]]
	var key := "%d:%d" % [int(r.id), at[1]]
	if painted.has(key):
		return
	painted[key] = true
	people.set_priority(int(r.id), at[1], value)
	var pri: PackedInt32Array = r.priorities
	pri[at[1]] = value
	r.priorities = pri

func _on_grid_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed:
			var at := _cell_at(event.position)
			if at.is_empty():
				return
			var r: Dictionary = _visible_rows()[at[0]]
			paint_value = (int((r.priorities as PackedInt32Array)[at[1]]) + 1) % 5
			painted = {}
			_set_cell(at, paint_value)
			grid.queue_redraw()
		elif paint_value >= 0:
			if not painted.is_empty():
				var first: Array = painted.keys()[0].split(":")
				sim.log_priority("%d cell%s" % [painted.size(), "" if painted.size() == 1 else "s"], jobs[int(first[1])], paint_value)
			paint_value = -1
			_reload()
	elif event is InputEventMouseMotion and paint_value >= 0 and (event.button_mask & MOUSE_BUTTON_MASK_LEFT):
		var at := _cell_at(event.position)
		if not at.is_empty():
			_set_cell(at, paint_value)
			grid.queue_redraw()
