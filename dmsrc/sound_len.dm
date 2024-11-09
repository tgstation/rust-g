// Provided a static RSC file path or a raw text file path, returns the duration of the file in deciseconds as a float.
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

#define RUSTG_SOUNDLEN_SUCCESSES "successes"
#define RUSTG_SOUNDLEN_ERRORS "errors"
// Returns a list of lists "successes" and "errors". Successes are file_path : duration. Errors are file_path : error.
#define rustg_sound_length_list(file_paths) json_decode(RUSTG_CALL(RUST_G, "sound_len_list")(json_encode(file_paths)))
