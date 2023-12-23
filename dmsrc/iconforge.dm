#define rustg_iconforge_generate(file_path, spritesheet_name, sprites) RUSTG_CALL(RUST_G, "iconforge_generate")(file_path, spritesheet_name, sprites)
#define rustg_iconforge_generate_async(file_path, spritesheet_name, sprites) RUSTG_CALL(RUST_G, "iconforge_generate_async")(file_path, spritesheet_name, sprites)
#define rustg_iconforge_check(job_id) RUSTG_CALL(RUST_G, "iconforge_check")("[job_id]")
