/// Generates a spritesheet at: [file_path][spritesheet_name]_[size_id].[png or dmi]
/// The resulting spritesheet arranges icons in a random order, with the position being denoted in the "sprites" return value.
/// All icons have the same y coordinate, and their x coordinate is equal to `icon_width * position`.
///
/// hash_icons is a boolean (0 or 1), and determines if the generator will spend time creating hashes for the output field dmi_hashes.
/// These hashes can be helpful for 'smart' caching (see rustg_iconforge_cache_valid), but require extra computation.
///
/// generate_dmi is a boolean (0 or 1), and determines if the generator will save the sheet as a DMI or stripped PNG file.
/// DMI files can be used to replace bulk Insert() operations, PNGs are more useful for asset transport or UIs. DMI generation is slower due to more metadata.
/// flatten is a boolean (0 or 1), and determines if the DMI output will be flattened to a single frame/dir if unscoped (null/0 dir or frame values).
/// PNGs are always flattened, regardless of argument.
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
/// list("type" = RUSTG_ICONFORGE_MAP_COLORS, "rr" = 0.5, "rg" = 0.5, "rb" = 0.5, "ra" = 1, "gr" = 1, "gg" = 1, "gb" = 1, "ga" = 1, ...) // alpha arguments and rgba0 optional
/// list("type" = RUSTG_ICONFORGE_FLIP, "dir" = SOUTH)
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
#define rustg_iconforge_generate(file_path, spritesheet_name, sprites, hash_icons, generate_dmi, flatten) RUSTG_CALL(RUST_G, "iconforge_generate")(file_path, spritesheet_name, sprites, "[hash_icons]", "[generate_dmi]", "[flatten]")
/// Returns a job_id for use with rustg_iconforge_check()
#define rustg_iconforge_generate_async(file_path, spritesheet_name, sprites, hash_icons, generate_dmi, flatten) RUSTG_CALL(RUST_G, "iconforge_generate_async")(file_path, spritesheet_name, sprites, "[hash_icons]", "[generate_dmi]", "[flatten]")
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
/// Provided a /datum/greyscale_config typepath, JSON string containing the greyscale config, and path to a DMI file containing the base icons,
/// Loads that config into memory for later use by rustg_iconforge_gags(). The config_path is the unique identifier used later.
/// JSON Config schema: https://hackmd.io/@tgstation/GAGS-Layer-Types
/// Adding dirs or frames (via blending larger icons) to icons with more than 1 dir or 1 frame is not supported.
/// Returns "OK" if successful, otherwise, returns a string containing the error.
#define rustg_iconforge_load_gags_config(config_path, config_json, config_icon_path) RUSTG_CALL(RUST_G, "iconforge_load_gags_config")("[config_path]", config_json, config_icon_path)
/// Given a config_path (previously loaded by rustg_iconforge_load_gags_config), and a string of hex colors formatted as "#ff00ff#ffaa00"
/// Outputs a DMI containing all of the states within the config JSON to output_dmi_path, creating any directories leading up to it if necessary.
/// Returns "OK" if successful, otherwise, returns a string containing the error.
#define rustg_iconforge_gags(config_path, colors, output_dmi_path) RUSTG_CALL(RUST_G, "iconforge_gags")("[config_path]", colors, output_dmi_path)
/// Returns a job_id for use with rustg_iconforge_check()
#define rustg_iconforge_load_gags_config_async(config_path, config_json, config_icon_path) RUSTG_CALL(RUST_G, "iconforge_load_gags_config_async")("[config_path]", config_json, config_icon_path)
/// Returns a job_id for use with rustg_iconforge_check()
#define rustg_iconforge_gags_async(config_path, colors, output_dmi_path) RUSTG_CALL(RUST_G, "iconforge_gags_async")("[config_path]", colors, output_dmi_path)

#define RUSTG_ICONFORGE_BLEND_COLOR "BlendColor"
#define RUSTG_ICONFORGE_BLEND_ICON "BlendIcon"
#define RUSTG_ICONFORGE_CROP "Crop"
#define RUSTG_ICONFORGE_SCALE "Scale"
#define RUSTG_ICONFORGE_MAP_COLORS "MapColors"
#define RUSTG_ICONFORGE_FLIP "Flip"
#define RUSTG_ICONFORGE_TURN "Turn"
#define RUSTG_ICONFORGE_SHIFT "Shift"
