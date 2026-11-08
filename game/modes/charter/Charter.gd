extends ScreenBase
## The Charter: laws as cards with cost and effect; districts, foremen, Mood, petitions.

const LAWS := [
	["RATIONING", "slimo draw halved", "Mood -5/shift"],
	["OPEN GRATES", "trade any faction +1", "Attention +50%"],
	["CONSCRIPTION", "D2s are militia on alarm", "scav -20% on alarm"],
	["QUARANTINE", "auto-sever lost regions", "false positives"],
	["DISSOLUTION RITE", "D5 feeding, no event hit", "Mood -3 standing"],
	["TITHE", "10% T1 to Oathkeepers", "they map the Heart"],
	["CURFEW", "Pods during Dark", "output down"],
	["FOREMAN'S CUT", "foremen keep 5%", "quotas slower"],
]

func _ready() -> void:
	screen_title = "CHARTER"
	super._ready()

func _build_content(c: Control) -> void:
	make_slot(c, "ivan.portrait", Vector2(568, 0), 0, 0, 0.0, "a2")
	make_label(c, "Authority 3   -   a law propagates from the Heart as a signal wave; far districts feel it a shift later", Vector2(0, 0), UiTheme.DUST)
	var x := 0
	var y := 16
	for i in LAWS.size():
		var l: Array = LAWS[i]
		var f := make_frame(c, Vector2(x, y), Vector2(154, 70))
		make_label(f, l[0], Vector2(6, 4), UiTheme.CYAN)
		make_label(f, l[1], Vector2(6, 18), UiTheme.CHALK)
		make_label(f, "cost: " + l[2], Vector2(6, 30), UiTheme.HOT_PINK)
		make_button(f, "SIGNED" if i == 1 else "SIGN", Vector2(6, 44))
		x += 158
		if x > 480:
			x = 0
			y += 74
	var d := make_frame(c, Vector2(0, 170), Vector2(632, 120))
	make_label(d, "DISTRICTS", Vector2(6, 4), UiTheme.CYAN)
	make_label(d, "Sump-side     foreman Okafor    Mood 61    priorities feed > build > trade", Vector2(6, 20), UiTheme.CHALK)
	make_label(d, "East Culvert  foreman Deacon    Mood 38    petition: 'the Pit is dry'  [ANSWER]", Vector2(6, 32), UiTheme.ACID)
