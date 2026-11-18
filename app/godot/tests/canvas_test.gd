extends SceneTree
## Headless verification & baseline benchmark for centralized canvas prototype.

func _init() -> void:
	print("======================================================")
	print("--- Running DLF Centralized Canvas Prototype Test ---")
	print("======================================================")
	
	var canvas_script = load("res://scenes/prototype_canvas/canvas_renderer.gd")
	if canvas_script == null:
		printerr("[FAIL] Could not load canvas_renderer.gd")
		quit(1)
		return
	
	var renderer: CanvasRenderer = canvas_script.new()
	root.add_child(renderer)
	
	# Test 1: Generate 2500 components
	var count := 2500
	var t0 := Time.get_ticks_usec()
	renderer.generate_synthetic_circuit(count)
	var t_gen_us := Time.get_ticks_usec() - t0
	print("[1/4] Generated %d components in %.2f ms (Spatial hash cells: %d)" % [
		count,
		t_gen_us / 1000.0,
		renderer._spatial_grid.size()
	])
	
	if renderer._items.size() != count:
		printerr("[FAIL] Expected %d items, got %d" % [count, renderer._items.size()])
		quit(1)
		return

	# Test 2: Pan & Zoom transformation math
	print("[2/4] Verifying Pan & Zoom coordinate transformations...")
	renderer.pan_offset = Vector2(150.0, 75.0)
	renderer.zoom_level = 2.0
	
	var screen_pt := Vector2(350.0, 275.0)
	var world_pt := renderer.screen_to_world(screen_pt)
	# Expected: (350 - 150)/2 = 100, (275 - 75)/2 = 100
	if abs(world_pt.x - 100.0) > 0.001 or abs(world_pt.y - 100.0) > 0.001:
		printerr("[FAIL] Incorrect screen_to_world transformation: ", world_pt)
		quit(1)
		return
	print("      screen_to_world((350, 275)) -> ", world_pt, " [OK]")

	# Test 3: Hit-testing accuracy & performance benchmark
	print("[3/4] Benchmarking uniform-grid spatial hash hit-testing (10,000 queries)...")
	var target_item: CanvasRenderer.CircuitVisualItem = renderer._items[42]
	var query_point := target_item.rect.get_center()
	
	var t_hit_start := Time.get_ticks_usec()
	var queries := 10000
	var found_count := 0
	for i in range(queries):
		var res := renderer.hit_test(query_point)
		if res != null and res.id == target_item.id:
			found_count += 1
	var t_hit_total_us := Time.get_ticks_usec() - t_hit_start
	var avg_hit_us := float(t_hit_total_us) / queries
	
	print("      Hit-tested 10,000 queries in %.2f ms (Avg: %.3f µs per query)" % [
		t_hit_total_us / 1000.0,
		avg_hit_us
	])
	if found_count != queries:
		printerr("[FAIL] Hit-test failed to consistently find target item.")
		quit(1)
		return

	# Test 4: Selection
	print("[4/4] Testing selection at target screen position...")
	renderer.pan_offset = Vector2.ZERO
	renderer.zoom_level = 1.0
	var sel_item := renderer.select_at(query_point)
	if sel_item == null or sel_item.id != target_item.id or not sel_item.is_selected:
		printerr("[FAIL] Selection failed.")
		quit(1)
		return
	print("      Selected Item ID: %d (%s) [OK]" % [sel_item.id, sel_item.label])

	print("======================================================")
	print("[PASS] Centralized Canvas Prototype verification PASSED!")
	print("======================================================")
	renderer.queue_free()
	quit(0)
