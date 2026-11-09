extends ScreenBase
## Job grid: rows slimos, columns the nine jobs, 1-4 priority cells that cycle on click.

const JOBS := ["HAUL", "BUILD", "SCAV", "TWEAK", "TEND", "GUARD", "COOK", "APPRS", "RSRCH"]
const NAMES := ["Voss", "Okafor", "Tiny Ren", "Deacon", "Pale Jo", "Marrow", "Sixteen", "Hull"]

func _ready() -> void:
	screen_title = "JOBS"
	super._ready()

func _build_content(c: Control) -> void:
	make_label(c, "1 first  4 last  .  never   district priorities override when the foreman's are higher", Vector2(0, 0), UiTheme.DUST)
	make_slot(c, "slimo.d2.idle", Vector2(590, 0), 32, 48, 4.0)
	var x0 := 90
	for j in JOBS.size():
		make_label(c, JOBS[j], Vector2(x0 + j * 52, 14), UiTheme.CYAN)
	for r in NAMES.size():
		var y := 30 + r * 22
		make_label(c, NAMES[r], Vector2(0, y + 4))
		for j in JOBS.size():
			var b := Button.new()
			b.text = str((r + j) % 5) if (r + j) % 5 > 0 else "."
			b.position = Vector2(x0 + j * 52, y)
			b.custom_minimum_size = Vector2(44, 18)
			b.focus_mode = Control.FOCUS_NONE
			b.pressed.connect(func():
				var v := 0 if b.text == "." else int(b.text)
				v = (v + 1) % 5
				b.text = str(v) if v > 0 else ".")
			c.add_child(b)
