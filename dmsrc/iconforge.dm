/// Generates a spritesheet at: [file_path][spritesheet_name]_[size_id].png
/// Spritesheet will contain all sprites listed within "sprites".
/// Sprite object format: list(
/// 	icon_file = 'icons/path_to/an_icon.dmi',
/// 	icon_state = "some_icon_state",
/// 	dir = SOUTH,
/// 	frame = 1,
/// 	transform = list(transform_object, ...)
///	)
/// transform_object format:
/// list("type" = "Color", "color" = "#ff0000", "blend_mode" = ICON_MULTIPLY)
/// list("type" = "Icon", "icon" = sprite_object, "blend_mode" = ICON_OVERLAY)
/// list("type" = "Scale", "width" = 32, "height" = 32)
/// list("type" = "Crop", "x1" = 0, "y1" = 0, "x2" = 32, "y2" = 32)
/// Returns a SpritesheetResult as JSON, containing fields:
///		sizes: list("32x32", "64x64", ...etc)
///		sprites: list("sprite_name" = list("size_id" = "32x32", "position" = 0), ...)
///		error: A string, empty if there were no errors.
/// In the event of an unrecoverable error, where the spritesheet could not even generate, returns a string containing the error.
#define rustg_iconforge_generate(file_path, spritesheet_name, sprites) RUSTG_CALL(RUST_G, "iconforge_generate")(file_path, spritesheet_name, sprites)
/// Returns a job_id for use with rustg_iconforge_check()
#define rustg_iconforge_generate_async(file_path, spritesheet_name, sprites) RUSTG_CALL(RUST_G, "iconforge_generate_async")(file_path, spritesheet_name, sprites)
/// Returns the status of a job_id
#define rustg_iconforge_check(job_id) RUSTG_CALL(RUST_G, "iconforge_check")("[job_id]")
/// Clears all cached DMIs and images, freeing up memory.
/// This should be used after spritesheets are done being generated.
#define rustg_iconforge_cleanup() RUSTG_CALL(RUST_G, "iconforge_cleanup")()
