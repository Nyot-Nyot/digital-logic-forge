class_name CanvasRenderer
extends Control
## Centralized custom canvas renderer untuk Digital Logic Forge.
##
## Menggambar seluruh komponen, wire, dan pin secara terpusat melalui fungsi `_draw()`
## tanpa membuat ribuan SceneTree Node individual (sesuai Bab 3.5.1 proposal).
## Dilengkapi uniform-grid spatial hash untuk hit-testing O(1).

# Representasi visual ringan elemen
class CircuitVisualItem:
	var id: int
	var rect: Rect2
	var label: String
	var is_selected: bool = false
	var color: Color

# Daftar item visual
var _items: Array[CircuitVisualItem] = []

# Kamera / viewport
var zoom_level: float = 1.0
var pan_offset: Vector2 = Vector2.ZERO

# Spatial Hash Grid untuk hit-testing cepat
const CELL_SIZE: float = 100.0
var _spatial_grid: Dictionary = {} # Vector2i -> Array[CircuitVisualItem]

# Warna styling
const COLOR_GATE_DEFAULT := Color(0.18, 0.22, 0.28)
const COLOR_GATE_BORDER := Color(0.4, 0.5, 0.65)
const COLOR_SELECTED_BORDER := Color(0.95, 0.75, 0.1)
const COLOR_WIRE := Color(0.3, 0.75, 0.4)
const COLOR_TEXT := Color(0.9, 0.9, 0.95)

func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_PASS

## Membangun benchmark dataset komponen sintetis
func generate_synthetic_circuit(count: int) -> void:
	_items.clear()
	_spatial_grid.clear()
	
	var cols: int = int(ceil(sqrt(count)))
	var spacing: float = 120.0
	
	for i in range(count):
		var col: int = i % cols
		var row: int = i / cols
		var pos := Vector2(col * spacing + 50.0, row * spacing + 50.0)
		var item := CircuitVisualItem.new()
		item.id = i + 1
		item.rect = Rect2(pos, Vector2(80, 50))
		item.label = "G_%04d" % (i + 1)
		item.color = COLOR_GATE_DEFAULT
		_items.append(item)
		_insert_to_spatial_grid(item)
	
	queue_redraw()

func _insert_to_spatial_grid(item: CircuitVisualItem) -> void:
	var min_cell := _get_cell_coord(item.rect.position)
	var max_cell := _get_cell_coord(item.rect.end)
	
	for cx in range(min_cell.x, max_cell.x + 1):
		for cy in range(min_cell.y, max_cell.y + 1):
			var cell := Vector2i(cx, cy)
			if not _spatial_grid.has(cell):
				_spatial_grid[cell] = []
			_spatial_grid[cell].append(item)

func _get_cell_coord(world_pos: Vector2) -> Vector2i:
	return Vector2i(int(floor(world_pos.x / CELL_SIZE)), int(floor(world_pos.y / CELL_SIZE)))

## Konversi koordinat layar ke koordinat dunia kanvas
func screen_to_world(screen_pos: Vector2) -> Vector2:
	return (screen_pos - pan_offset) / zoom_level

## Hit-testing O(1) berbasis uniform-grid spatial hash
func hit_test(world_pos: Vector2) -> CircuitVisualItem:
	var cell := _get_cell_coord(world_pos)
	if _spatial_grid.has(cell):
		var candidates: Array = _spatial_grid[cell]
		# Iterasi mundur untuk mengambil elemen paling atas
		for idx in range(candidates.size() - 1, -1, -1):
			var candidate: CircuitVisualItem = candidates[idx]
			if candidate.rect.has_point(world_pos):
				return candidate
	return null

## Pilih elemen pada posisi kursor layar
func select_at(screen_pos: Vector2, toggle_multi: bool = false) -> CircuitVisualItem:
	var world_pos := screen_to_world(screen_pos)
	var clicked_item := hit_test(world_pos)
	
	if not toggle_multi:
		for it in _items:
			it.is_selected = false
			
	if clicked_item != null:
		clicked_item.is_selected = true
		
	queue_redraw()
	return clicked_item

func _draw() -> void:
	# Terapkan transformasi zoom dan pan
	draw_set_transform(pan_offset, 0.0, Vector2(zoom_level, zoom_level))
	
	# Gambar koneksi sintetis antar komponen
	var wire_color := COLOR_WIRE
	for i in range(1, _items.size()):
		var prev := _items[i - 1].rect.get_center()
		var curr := _items[i].rect.get_center()
		if prev.distance_to(curr) < 200.0:
			draw_line(prev, curr, wire_color, 1.5)
	
	# Gambar badan komponen
	var default_font := ThemeDB.fallback_font
	for item in _items:
		var fill_color := item.color
		var border_color := COLOR_SELECTED_BORDER if item.is_selected else COLOR_GATE_BORDER
		var border_width := 2.5 if item.is_selected else 1.0
		
		# Gambar kotak komponen
		draw_rect(item.rect, fill_color, true)
		draw_rect(item.rect, border_color, false, border_width)
		
		# Gambar pin sederhana
		var pin_in := Vector2(item.rect.position.x, item.rect.get_center().y)
		var pin_out := Vector2(item.rect.end.x, item.rect.get_center().y)
		draw_circle(pin_in, 3.0, Color.DEEP_SKY_BLUE)
		draw_circle(pin_out, 3.0, Color.ORANGE)
		
		# Gambar label
		var text_pos := Vector2(item.rect.position.x + 10, item.rect.get_center().y + 5)
		draw_string(default_font, text_pos, item.label, HORIZONTAL_ALIGNMENT_LEFT, -1, 12, COLOR_TEXT)
