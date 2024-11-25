/// Provided a static RSC file path or a raw text file path, returns the duration of the file in deciseconds as a float.
/proc/rustg_sound_length(file_path)
	var/static/list/sound_cache
	if(isnull(sound_cache))
		sound_cache = list()

	. = 0

	if(!istext(file_path))
		if(!isfile(file_path))
			CRASH("rustg_sound_length error: Passed non-text object")

		if(length("[file_path]")) // Runtime generated RSC references stringify into 0-length strings.
			file_path = "[file_path]"
		else
			CRASH("rustg_sound_length does not support non-static file refs.")

	var/cached_length = sound_cache[file_path]
	if(!isnull(cached_length))
		return cached_length

	var/ret = RUSTG_CALL(RUST_G, "sound_len")(file_path)
	var/as_num = text2num(ret)
	if(isnull(ret))
		. = 0
		CRASH("rustg_sound_length error: [ret]")

	sound_cache[file_path] = as_num
	return as_num


#define RUSTG_SOUNDLEN_SUCCESSES "successes"
#define RUSTG_SOUNDLEN_ERRORS "errors"
/**
 * Returns a nested key-value list containing "successes" and "errors"
 * The format is as follows:
 * list(
 *  RUSTG_SOUNDLEN_SUCCESES = list("sounds/test.ogg" = 25.34),
 *  RUSTG_SOUNDLEN_ERRORS = list("sound/bad.png" = "SoundLen: Unable to decode file."),
 *)
*/
#define rustg_sound_length_list(file_paths) json_decode(RUSTG_CALL(RUST_G, "sound_len_list")(json_encode(file_paths)))
