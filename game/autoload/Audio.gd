extends Node
## The sound channel (Full Design v3.24, Feel: sound; Phase G, PAC-107). Four buses under Master — Music, World, UI
## and Voice — and every sound a slot id, as every drawable is: `play("sfx.reward")` resolves
## `res://assets/sound/<id>.wav` (or .ogg) when one exists, else a placeholder tone made here from the slot's recipe,
## so the game sounds before any audio is made and a made file replaces a placeholder with no code change. The bus
## volumes are the Settings screen's, kept in user://settings.cfg.

const SOUND := "res://assets/sound/"
const BUSES := ["Music", "World", "UI", "Voice"]
const SETTINGS := "user://settings.cfg"

## slot id -> [bus, placeholder recipe: [wave ("square" | "noise" | "tri"), start Hz, end Hz, ms, volume 0-1]]
const SLOTS := {
	"sfx.reward": ["UI", ["square", 660.0, 990.0, 120, 0.35]],
	"sfx.reward.big": ["UI", ["square", 523.0, 1046.0, 260, 0.4]],
	"sfx.loss": ["UI", ["tri", 330.0, 165.0, 220, 0.4]],
	"sfx.alert": ["UI", ["square", 880.0, 880.0, 80, 0.25]],
	"sfx.click": ["UI", ["square", 1200.0, 1200.0, 18, 0.2]],
	"sfx.place": ["World", ["noise", 400.0, 200.0, 90, 0.3]],
	"sfx.feed": ["World", ["tri", 220.0, 440.0, 180, 0.35]],
	"sfx.land": ["World", ["tri", 440.0, 660.0, 70, 0.25]],
	"sfx.grow": ["World", ["tri", 180.0, 260.0, 140, 0.2]],
	"sfx.siren": ["World", ["square", 440.0, 620.0, 400, 0.3]],
	"sfx.ivan": ["Voice", ["tri", 140.0, 120.0, 160, 0.3]],
	"music.sump": ["Music", ["tri", 55.0, 55.0, 1200, 0.15]],
}

const MIX_RATE := 22050
var _cache := {}
var _players: Array[AudioStreamPlayer] = []
var _next := 0
## what played, for the tests: [slot ids]
var played: Array = []
var muted := false   # the headless tests: resolve and count, make no sound

func _ready() -> void:
	ensure_buses()
	for i in 8:
		var p := AudioStreamPlayer.new()
		add_child(p)
		_players.append(p)
	load_volumes()

## The bus layout: Music, World, UI and Voice under Master (made here, so the project needs no layout file).
func ensure_buses() -> void:
	for b in BUSES:
		if AudioServer.get_bus_index(b) < 0:
			AudioServer.add_bus()
			var i := AudioServer.bus_count - 1
			AudioServer.set_bus_name(i, b)
			AudioServer.set_bus_send(i, "Master")

## Play a slot. "" when it played (or would, muted), else why not.
func play(slot_id: String) -> String:
	if not SLOTS.has(slot_id):
		return "no sound slot %s" % slot_id
	var s := stream(slot_id)
	if s == null:
		return "no sound for %s" % slot_id
	played.append(slot_id)
	if muted or DisplayServer.get_name() == "headless":
		return ""
	var p := _players[_next]
	_next = (_next + 1) % _players.size()
	p.stream = s
	p.bus = String(SLOTS[slot_id][0])
	p.play()
	return ""

## A slot's stream: the made file, else its placeholder.
func stream(slot_id: String) -> AudioStream:
	if _cache.has(slot_id):
		return _cache[slot_id]
	var s: AudioStream = null
	for ext in [".wav", ".ogg"]:
		var path: String = ProjectSettings.globalize_path(SOUND + slot_id + ext)
		if FileAccess.file_exists(path):
			s = AudioStreamWAV.load_from_file(path) if ext == ".wav" else AudioStreamOggVorbis.load_from_file(path)
			if s != null:
				break
	if s == null and SLOTS.has(slot_id):
		s = _placeholder(SLOTS[slot_id][1])
	_cache[slot_id] = s
	return s

## Whether a slot's sound is a made file (false: the placeholder).
func is_made(slot_id: String) -> bool:
	for ext in [".wav", ".ogg"]:
		if FileAccess.file_exists(ProjectSettings.globalize_path(SOUND + slot_id + ext)):
			return true
	return false

## A placeholder tone: a square, triangle or noise sweep with a hard 8-bit envelope (attack 2 ms, linear decay), 16-bit
## mono. Seeded noise, so a placeholder is the same every run.
func _placeholder(r: Array) -> AudioStreamWAV:
	var wave := String(r[0])
	var f0 := float(r[1])
	var f1 := float(r[2])
	var n := int(MIX_RATE * int(r[3]) / 1000)
	var vol := float(r[4])
	var data := PackedByteArray()
	data.resize(n * 2)
	var phase := 0.0
	var seed_v := 0x1234
	var held := 0.0
	for i in n:
		var t := float(i) / float(maxi(n - 1, 1))
		var f := lerpf(f0, f1, t)
		phase = fmod(phase + f / MIX_RATE, 1.0)
		var v := 0.0
		match wave:
			"square":
				v = 1.0 if phase < 0.5 else -1.0
			"tri":
				v = 4.0 * absf(phase - 0.5) - 1.0
			"noise":
				# a new random level each period of the sweep: crunchy, not hiss
				if phase < f / MIX_RATE:
					seed_v = (seed_v * 1103515245 + 12345) & 0x7fffffff
					held = float(seed_v % 2001) / 1000.0 - 1.0
				v = held
		var env := minf(1.0, float(i) / (MIX_RATE * 0.002)) * (1.0 - t)
		var sample := int(clampf(v * env * vol, -1.0, 1.0) * 32767.0)
		data.encode_s16(i * 2, sample)
	var s := AudioStreamWAV.new()
	s.format = AudioStreamWAV.FORMAT_16_BITS
	s.mix_rate = MIX_RATE
	s.stereo = false
	s.data = data
	return s

# ---------------------------------------------------------------- the volumes (Settings: AUDIO)
## 0-100 a bus: "Master" and the four.
func volume(bus: String) -> int:
	var i := AudioServer.get_bus_index(bus)
	if i < 0:
		return 0
	return 0 if AudioServer.is_bus_mute(i) else int(round(db_to_linear(AudioServer.get_bus_volume_db(i)) * 100.0))

func set_volume(bus: String, pct: int) -> void:
	var i := AudioServer.get_bus_index(bus)
	if i < 0:
		return
	pct = clampi(pct, 0, 100)
	AudioServer.set_bus_mute(i, pct == 0)
	AudioServer.set_bus_volume_db(i, linear_to_db(maxf(pct / 100.0, 0.0001)))
	save_volumes()

func save_volumes() -> void:
	var cfg := ConfigFile.new()
	cfg.load(SETTINGS)
	for b in ["Master"] + BUSES:
		cfg.set_value("audio", b, volume(b))
	cfg.save(SETTINGS)

func load_volumes() -> void:
	var cfg := ConfigFile.new()
	if cfg.load(SETTINGS) != OK:
		return
	for b in ["Master"] + BUSES:
		var i := AudioServer.get_bus_index(b)
		var pct := int(cfg.get_value("audio", b, 100))
		if i >= 0:
			AudioServer.set_bus_mute(i, pct == 0)
			AudioServer.set_bus_volume_db(i, linear_to_db(maxf(pct / 100.0, 0.0001)))
