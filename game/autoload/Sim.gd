extends Node
## The one simulation, shared by Colony and Run. Started on first use; the clock lives here.

var sim: RoseSim
var paused := false
var tick_hz := 2.0
var accum := 0.0
var ticks_per_frame := 0   # headless tests: fixed ticks per frame instead of the clock
var seed_value := 11
var agents := 3000
var grow_ticks := 500
signal ticked

func ensure() -> RoseSim:
	if sim == null:
		sim = RoseSim.new()
		sim.start_scale(seed_value, agents, grow_ticks)
	return sim

func _process(delta: float) -> void:
	if sim == null:
		return
	if ticks_per_frame > 0:
		for i in ticks_per_frame:
			sim.step()
		ticked.emit()
		return
	if paused:
		return
	accum += delta
	var period := 1.0 / tick_hz
	var n := 0
	while accum >= period and n < 8:
		accum -= period
		sim.step()
		n += 1
	if n > 0:
		ticked.emit()

func set_speed(hz: float) -> void:
	tick_hz = hz
