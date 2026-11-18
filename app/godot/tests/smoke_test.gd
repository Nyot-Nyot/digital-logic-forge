extends SceneTree

func _init() -> void:
	print("======================================================")
	print("--- Running DLF GDExtension Full Smoke & Worker Test ---")
	print("======================================================")
	
	if not ClassDB.class_exists(&"DlfFacade"):
		printerr("[FAIL] Class 'DlfFacade' not found in ClassDB! GDExtension failed to load.")
		quit(1)
		return
	
	var facade: Object = ClassDB.instantiate(&"DlfFacade")
	if facade == null:
		printerr("[FAIL] Failed to instantiate 'DlfFacade'.")
		quit(1)
		return
	
	# 1. Test ping
	var ping_result: String = facade.ping()
	print("[1/5] Facade.ping() -> ", ping_result)
	if not ("Digital Logic Forge Core" in ping_result):
		printerr("[FAIL] ping() failed.")
		quit(1)
		return
	
	# 2. Test get_core_version
	var core_version: String = facade.get_core_version()
	print("[2/5] Facade.get_core_version() -> ", core_version)
	if core_version.is_empty():
		printerr("[FAIL] get_core_version() is empty.")
		quit(1)
		return

	# 3. Test worker_start
	print("[3/5] Starting background simulation worker thread...")
	var start_ok: bool = facade.worker_start()
	if not start_ok:
		printerr("[FAIL] worker_start() returned false.")
		quit(1)
		return
	
	OS.delay_msec(50)
	var snap1: Dictionary = facade.poll_snapshot()
	print("      Initial Snapshot: Status=", snap1.get("status"), " Cycles=", snap1.get("cycle_count"))
	if snap1.get("status") != "Paused":
		printerr("[FAIL] Expected initial status to be Paused after start, got: ", snap1.get("status"))
		quit(1)
		return

	# 4. Test worker_step
	print("[4/5] Testing worker_step(25)...")
	facade.worker_step(25)
	OS.delay_msec(50)
	var snap2: Dictionary = facade.poll_snapshot()
	print("      Stepped Snapshot: Status=", snap2.get("status"), " Cycles=", snap2.get("cycle_count"), " Outputs=", snap2.get("outputs"))
	if snap2.get("cycle_count") != 25:
		printerr("[FAIL] Expected 25 cycles after step(25), got: ", snap2.get("cycle_count"))
		quit(1)
		return
	
	# 5. Test worker_run & worker_pause
	print("[5/5] Testing worker_run() & worker_pause()...")
	facade.worker_run()
	OS.delay_msec(100) # Biarkan berjalan 100ms
	facade.worker_pause()
	OS.delay_msec(50)
	
	var snap3: Dictionary = facade.poll_snapshot()
	print("      Running/Paused Snapshot: Status=", snap3.get("status"), " Cycles=", snap3.get("cycle_count"), " CPS=", snap3.get("speed_cps"))
	if snap3.get("cycle_count") <= 25:
		printerr("[FAIL] Expected cycle count to advance after run(), got: ", snap3.get("cycle_count"))
		quit(1)
		return
	
	# Cleanup
	facade.worker_stop()
	print("======================================================")
	print("[PASS] All GDScript -> Rust Worker & Snapshot tests PASSED!")
	print("======================================================")
	quit(0)
