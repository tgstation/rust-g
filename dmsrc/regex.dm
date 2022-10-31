#define RUSTG_REGEX_FLAG_GLOBAL (1 << 0)

/// A drop in replacement for /regex using rust-g.
/// You should be able to replace anywhere you use regex() with this.
/// MBTODO: ...but not now.
/datum/rustg_regex
	var/index
	var/next
	var/group

	var/flags = 0

	var/pattern

// MBTODO: Flags
/datum/rustg_regex/New(pattern, flags)
	// MBTODO: Validate
	src.pattern = pattern

	if (!istext(flags))
		CRASH("Expected string for flags, received [flags]")

	for (var/character_index in length(flags))
		var/character = copytext(flags, character_index, character_index + 1)
		switch (character)
			if ("g")
				flags |= RUSTG_REGEX_FLAG_GLOBAL
			else
				CRASH("unknown flag passed to regex: [character]")

// MBTODO: End
/datum/rustg_regex/proc/Find(haystack, start, end = 0)
	if (isnull(start))
		if ((flags & RUSTG_REGEX_FLAG_GLOBAL) && !isnull(next))
			start = next
		else
			start = 1

	var/list/result = json_decode(RUSTG_CALL(RUST_G, "regex_captures")(haystack, "[start - 1]"))
	if (!result["success"])
		CRASH(result["reason"])
		return

	var/list/regex_result = result["result"]

	next = regex_result["next"] + 1
	index = regex_result["index"] + 1
	group = regex_result["captures"]

	return index

#undef RUSTG_REGEX_FLAG_GLOBAL
