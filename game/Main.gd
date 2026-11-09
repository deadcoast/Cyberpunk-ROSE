extends Node
## Root: hands the screen root to the Screens autoload and opens the menu.

func _ready() -> void:
	Screens.attach($ScreenRoot)
	Screens.show_screen("menu")

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_ESCAPE:
		if Screens.current_id != "menu":
			Screens.show_screen("menu")
