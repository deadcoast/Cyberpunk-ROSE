extends Node
## Screen registry: fourteen shells, one visible at a time, under the root Main hands us.

const SCREENS := {
	"menu": "res://modes/menu/Menu.tscn",
	"colony": "res://modes/colony/Colony.tscn",
	"build": "res://modes/build/Build.tscn",
	"slimo": "res://modes/slimo/SlimoCard.tscn",
	"jobs": "res://modes/jobs/JobGrid.tscn",
	"stash": "res://modes/stash/Stash.tscn",
	"bench": "res://modes/bench/Bench.tscn",
	"market": "res://modes/market/Market.tscn",
	"strain": "res://modes/strain/StrainLab.tscn",
	"charter": "res://modes/charter/Charter.tscn",
	"route": "res://modes/route/RouteBoard.tscn",
	"run": "res://modes/run/Run.tscn",
	"codex": "res://modes/codex/Codex.tscn",
	"settings": "res://modes/settings/Settings.tscn",
}

var root: Control
var current_id := ""
var current: Node

func attach(screen_root: Control) -> void:
	root = screen_root

func ids() -> Array:
	return SCREENS.keys()

func show_screen(id: String) -> Node:
	assert(SCREENS.has(id), "unknown screen " + id)
	if current != null:
		current.queue_free()
		current = null
	var scene: PackedScene = load(SCREENS[id])
	current = scene.instantiate()
	current_id = id
	root.add_child(current)
	return current
