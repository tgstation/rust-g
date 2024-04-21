/// Generates a spritesheet at: [file_path][spritesheet_name]_[size_id].png
/// The resulting spritesheet arranges icons in a random order, with the position being denoted in the "sprites" return value.
/// All icons have the same y coordinate, and their x coordinate is equal to `icon_width * position`.
///
/// hash_icons is a boolean (0 or 1), and determines if the generator will spend time creating hashes for the output field dmi_hashes.
/// These hashes can be heplful for 'smart' caching (see rustg_iconforge_cache_valid), but require extra computation.
///
/// Spritesheet will contain all sprites listed within "sprites".
/// "sprites" format:
/// list(
///     "sprite_name" = list( // <--- this list is a [SPRITE_OBJECT]
///         icon_file = 'icons/path_to/an_icon.dmi',
///         icon_state = "some_icon_state",
///         dir = SOUTH,
///         frame = 1,
///         transform = list([TRANSFORM_OBJECT], ...)
///     ),
///     ...,
/// )
/// TRANSFORM_OBJECT format:
/// list("type" = RUSTG_ICONFORGE_BLEND_COLOR, "color" = "#ff0000", "blend_mode" = ICON_MULTIPLY)
/// list("type" = RUSTG_ICONFORGE_BLEND_ICON, "icon" = [SPRITE_OBJECT], "blend_mode" = ICON_OVERLAY)
/// list("type" = RUSTG_ICONFORGE_SCALE, "width" = 32, "height" = 32)
/// list("type" = RUSTG_ICONFORGE_CROP, "x1" = 1, "y1" = 1, "x2" = 32, "y2" = 32) // (BYOND icons index from 1,1 to the upper bound, inclusive)
///
/// Returns a SpritesheetResult as JSON, containing fields:
/// list(
///     "sizes" = list("32x32", "64x64", ...),
///     "sprites" = list("sprite_name" = list("size_id" = "32x32", "position" = 0), ...),
///     "dmi_hashes" = list("icons/path_to/an_icon.dmi" = "d6325c5b4304fb03", ...),
///     "sprites_hash" = "a2015e5ff403fb5c", // This is the xxh64 hash of the INPUT field "sprites".
///     "error" = "[A string, empty if there were no errors.]"
/// )
/// In the case of an unrecoverable panic from within Rust, this function ONLY returns a string containing the error.
#define rustg_iconforge_generate(file_path, spritesheet_name, sprites, hash_icons) RUSTG_CALL(RUST_G, "iconforge_generate")(file_path, spritesheet_name, sprites, "[hash_icons]")
/// Returns a job_id for use with rustg_iconforge_check()
#define rustg_iconforge_generate_async(file_path, spritesheet_name, sprites, hash_icons) RUSTG_CALL(RUST_G, "iconforge_generate_async")(file_path, spritesheet_name, sprites, "[hash_icons]")
/// Returns the status of an async job_id, or its result if it is completed. See RUSTG_JOB DEFINEs.
#define rustg_iconforge_check(job_id) RUSTG_CALL(RUST_G, "iconforge_check")("[job_id]")
/// Clears all cached DMIs and images, freeing up memory.
/// This should be used after spritesheets are done being generated.
#define rustg_iconforge_cleanup RUSTG_CALL(RUST_G, "iconforge_cleanup")
/// Takes in a set of hashes, generate inputs, and DMI filepaths, and compares them to determine cache validity.
/// input_hash: xxh64 hash of "sprites" from the cache.
/// dmi_hashes: xxh64 hashes of the DMIs in a spritesheet, given by `rustg_iconforge_generate` with `hash_icons` enabled. From the cache.
/// sprites: The new input that will be passed to rustg_iconforge_generate().
/// Returns a CacheResult with the following structure: list(
///     "result": "1" (if cache is valid) or "0" (if cache is invalid)
///     "fail_reason": "" (emtpy string if valid, otherwise a string containing the invalidation reason or an error with ERROR: prefixed.)
/// )
/// In the case of an unrecoverable panic from within Rust, this function ONLY returns a string containing the error.
#define rustg_iconforge_cache_valid(input_hash, dmi_hashes, sprites) RUSTG_CALL(RUST_G, "iconforge_cache_valid")(input_hash, dmi_hashes, sprites)
/// Returns a job_id for use with rustg_iconforge_check()
#define rustg_iconforge_cache_valid_async(input_hash, dmi_hashes, sprites) RUSTG_CALL(RUST_G, "iconforge_cache_valid_async")(input_hash, dmi_hashes, sprites)

#define RUSTG_ICONFORGE_BLEND_COLOR "BlendColor"
#define RUSTG_ICONFORGE_BLEND_ICON "BlendIcon"
#define RUSTG_ICONFORGE_CROP "Crop"
#define RUSTG_ICONFORGE_SCALE "Scale"
