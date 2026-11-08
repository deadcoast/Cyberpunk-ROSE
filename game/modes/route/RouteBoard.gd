extends ScreenBase
## Route Board: route cards with danger and timing, the Runner and followers, the rig. LAUNCH opens the run.

const ROUTES := [
	["SUMP EAST", "S1", "danger low", "Flood in 2 shifts", "3 containers known", ""],
	["CULVERT NORTH", "S2", "danger medium", "Flood in 2 shifts", "grate held", "corpse: Hull"],
	["STORM DRAIN", "S3", "danger high", "Flood in 1 shift", "Bloom 40 tiles west", ""],
]

func _ready() -> void:
	screen_title = "ROUTE BOARD"
	super._ready()

func _build_content(c: Control) -> void:
	var x := 0
	for r in ROUTES:
		var f := make_frame(c, Vector2(x, 0), Vector2(200, 110))
		make_label(f, r[0], Vector2(6, 4), UiTheme.CYAN)
		make_label(f, r[1] + "  -  " + r[2], Vector2(6, 18), UiTheme.CHALK)
		make_label(f, r[3], Vector2(6, 30), UiTheme.DUST)
		make_label(f, r[4], Vector2(6, 42), UiTheme.DUST)
		if r[5] != "":
			make_label(f, r[5], Vector2(6, 54), UiTheme.HOT_PINK)
		make_button(f, "SELECT", Vector2(6, 80))
		x += 206
	var crew := make_frame(c, Vector2(0, 116), Vector2(300, 174))
	make_label(crew, "CREW", Vector2(6, 4), UiTheme.CYAN)
	make_slot(crew, "slimo.d2.idle", Vector2(6, 20), 32, 48, 4.0)
	make_label(crew, "Voss  Runner  Scav 6 Fight 4", Vector2(44, 24), UiTheme.CHALK)
	make_slot(crew, "slimo.d3.idle", Vector2(6, 72), 32, 48, 4.0)
	make_label(crew, "Deacon  follower  Tweak 5", Vector2(44, 76), UiTheme.CHALK)
	make_slot(crew, "slimo.d2.idle", Vector2(6, 124), 32, 48, 4.0)
	make_label(crew, "Pale Jo  follower  Guard 3", Vector2(44, 128), UiTheme.CHALK)
	var rig := make_frame(c, Vector2(306, 116), Vector2(326, 174))
	make_label(rig, "RIG   6x4 backpack   4 quick   10.5 / 12 kg", Vector2(6, 4), UiTheme.CYAN)
	var gx := 6
	for id in ["ger_nail_rifle", "ger_lantern", "org_antiseptic", "org_stim", "ger_pry_bar", "prt_salt"]:
		var g := make_frame(rig, Vector2(gx, 20), Vector2(40, 40))
		make_slot(g, icon_for(id), Vector2(4, 4))
		gx += 44
	make_button(rig, "LAUNCH RUN", Vector2(6, 130), func(): Screens.show_screen("run"))
