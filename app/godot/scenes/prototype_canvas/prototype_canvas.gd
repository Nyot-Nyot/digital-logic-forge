extends Control
## Controller utama untuk prototype canvas terpusat DLF.
##
## Mengelola input pan/zoom mouse, selection, dan mencatat metrik baseline performa.

@onready var renderer: CanvasRenderer = $CanvasRenderer
@onready var lbl_info: Label = $HUD/MarginContainer/VBoxContainer/LblInfo
@onready var lbl_status: Label = $HUD/MarginContainer/VBoxContainer/LblStatus

var _is_panning: bool = false
var _pan_start: Vector2 = Vector2.ZERO

const INITIAL_COUNT: int = 1200

func _ready() -> void:
	renderer.generate_synthetic_circuit(INITIAL_COUNT)
	_update_hud("Initialized with %d components" % INITIAL_COUNT)

func _process(_delta: float) -> void:
	if lbl_info != null:
		var fps := Engine.get_frames_per_second()
		lbl_info.text = "Komponen: %d | FPS: %d | Zoom: %.2fx | Pan: (%d, %d)" % [
			renderer._items.size(),
			fps,
			renderer.zoom_level,
			int(renderer.pan_offset.x),
			int(renderer.pan_offset.y)
		]

func _gui_input(event: InputEvent) -> void:
	# 1. Pan dengan Middle Click atau Right Click drag
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index == MOUSE_BUTTON_MIDDLE or mb.button_index == MOUSE_BUTTON_RIGHT:
			if mb.pressed:
				_is_panning = true
				_pan_start = mb.position - renderer.pan_offset
			else:
				_is_panning = false
		
		# 2. Zoom dengan Mouse Wheel
		elif mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
			_zoom_at(mb.position, 1.15)
		elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
			_zoom_at(mb.position, 0.85)
			
		# 3. Hit-test & Select dengan Left Click
		elif mb.button_index == MOUSE_BUTTON_LEFT and mb.pressed:
			var t_start := Time.get_ticks_usec()
			var selected_item := renderer.select_at(mb.position)
			var t_elapsed_us := Time.get_ticks_usec() - t_start
			
			if selected_item != null:
				_update_hud("Terpilih: %s (ID: %d) dalam %d µs" % [selected_item.label, selected_item.id, t_elapsed_us])
			else:
				_update_hud("Klik di ruang kosong (Hit-test %d µs)" % t_elapsed_us)

	elif event is InputEventMouseMotion and _is_panning:
		var mm := event as InputEventMouseMotion
		renderer.pan_offset = mm.position - _pan_start
		renderer.queue_redraw()

func _zoom_at(pivot: Vector2, factor: float) -> void:
	var prev_zoom := renderer.zoom_level
	var new_zoom := clampf(prev_zoom * factor, 0.2, 5.0)
	
	# Zoom memusat ke posisi kursor mouse
	var world_pivot := renderer.screen_to_world(pivot)
	renderer.zoom_level = new_zoom
	renderer.pan_offset = pivot - world_pivot * new_zoom
	renderer.queue_redraw()

func _update_hud(msg: String) -> void:
	if lbl_status != null:
		lbl_status.text = msg
