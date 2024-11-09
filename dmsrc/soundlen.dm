/proc/rustg_sound_length(file_path)
	var/static/list/sound_cache
	if(isnull(sound_cache))
		sound_cache = list()

	. = 0

	if(!istext(file_path))
		if(!isfile(file_path))
			CRASH("rustg_sound_length error: Passed non-text object")

		if(length("[file_path]"))
			file_path = "[file_path]"
		else
			CRASH("rustg_sound_length does not support non-static file refs.")

	if(!isnull((. = sound_cache[file_path])))
		return .

	var/ret = RUSTG_CALL(RUST_G, "sound_len")(file_path)
	var/as_num = text2num(ret)
	if(isnull(ret))
		. = 0
		CRASH("rustg_sound_length error: [ret]")

	sound_cache[file_path] = as_num
	return as_num
