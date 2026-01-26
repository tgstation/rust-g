use super::{gags, image_cache, spritesheet};
use crate::{byond::catch_panic, iconforge::spritesheet::HeadlessResult, jobs};
use tracy_full::frame;

byond_fn!(fn iconforge_generate(file_path, spritesheet_name, sprites, hash_icons, generate_dmi, flatten) {
    let file_path = file_path.to_owned();
    let spritesheet_name = spritesheet_name.to_owned();
    let sprites = sprites.to_owned();
    let hash_icons = hash_icons.to_owned();
    let generate_dmi = generate_dmi.to_owned();
    let flatten = flatten.to_owned();
    let result = Some(match catch_panic(|| spritesheet::generate_spritesheet(&file_path, &spritesheet_name, &sprites, &hash_icons, &generate_dmi, &flatten)) {
        Ok(o) => match o {
            Ok(o) => o,
            Err(e) => e.to_string()
        },
        Err(e) => e.to_string()
    });
    frame!();
    result
});

byond_fn!(fn iconforge_generate_async(file_path, spritesheet_name, sprites, hash_icons, generate_dmi, flatten) {
    let file_path = file_path.to_owned();
    let spritesheet_name = spritesheet_name.to_owned();
    let sprites = sprites.to_owned();
    let hash_icons = hash_icons.to_owned();
    let generate_dmi = generate_dmi.to_owned();
    let flatten = flatten.to_owned();
    Some(jobs::start(move || {
        let result = match catch_panic(|| spritesheet::generate_spritesheet(&file_path, &spritesheet_name, &sprites, &hash_icons, &generate_dmi, &flatten)) {
            Ok(o) => match o {
                Ok(o) => o,
                Err(e) => e.to_string()
            },
            Err(e) => e.to_string()
        };
        frame!();
        result
    }))
});

byond_fn!(fn iconforge_generate_headless(file_path, sprites, flatten) {
    let file_path = file_path.to_owned();
    let sprites = sprites.to_owned();
    let flatten = flatten.to_owned();
    let result = Some(match catch_panic::<_, HeadlessResult>(|| spritesheet::generate_headless(&file_path, &sprites, &flatten)) {
        Ok(o) => match serde_json::to_string::<HeadlessResult>(&o) {
            Ok(o) => o,
            Err(_) => String::from("{\"error\":\"Serde serialization error\"}") // nigh impossible but whatever
        },
        Err(e) => match serde_json::to_string::<HeadlessResult>(&HeadlessResult {
            file_path: None,
            width: None,
            height: None,
            error: Some(e.to_string()),
        }) {
            Ok(o) => o,
            Err(_) => String::from("{\"error\":\"Serde serialization error\"}")
        }
    });
    frame!();
    result
});

byond_fn!(fn iconforge_check(id) {
    Some(jobs::check(id))
});

byond_fn!(
    fn iconforge_cleanup() {
        // Only perform cleanup if no jobs are currently using the icon cache
        if image_cache::CACHE_ACTIVE.load(std::sync::atomic::Ordering::SeqCst) > 0 {
            return Some("Skipped, cache in use");
        }

        image_cache::icon_cache_clear();
        image_cache::image_cache_clear();
        Some("Ok")
    }
);

byond_fn!(fn iconforge_cache_valid(input_hash, dmi_hashes, sprites) {
    let input_hash = input_hash.to_owned();
    let dmi_hashes = dmi_hashes.to_owned();
    let sprites = sprites.to_owned();
    let result = Some(match catch_panic(|| spritesheet::cache_valid(&input_hash, &dmi_hashes, &sprites)) {
        Ok(o) => match o {
            Ok(o) => o,
            Err(e) => e.to_string()
        },
        Err(e) => e.to_string()
    });
    frame!();
    result
});

byond_fn!(fn iconforge_cache_valid_async(input_hash, dmi_hashes, sprites) {
    let input_hash = input_hash.to_owned();
    let dmi_hashes = dmi_hashes.to_owned();
    let sprites = sprites.to_owned();
    let result = Some(jobs::start(move || {
        match catch_panic(|| spritesheet::cache_valid(&input_hash, &dmi_hashes, &sprites)) {
            Ok(o) => match o {
                Ok(o) => o,
                Err(e) => e.to_string()
            },
            Err(e) => e.to_string()
        }
    }));
    frame!();
    result
});

byond_fn!(fn iconforge_load_gags_config(config_path, config_json, config_icon_path) {
    let config_path = config_path.to_owned();
    let config_json = config_json.to_owned();
    let config_icon_path = config_icon_path.to_owned();
    let result = Some(match catch_panic(|| gags::load_gags_config(&config_path, &config_json, &config_icon_path)) {
        Ok(o) => match o {
            Ok(o) => o,
            Err(e) => e.to_string()
        },
        Err(e) => e.to_string()
    });
    frame!();
    result
});

byond_fn!(fn iconforge_load_gags_config_async(config_path, config_json, config_icon_path) {
    let config_path = config_path.to_owned();
    let config_json = config_json.to_owned();
    let config_icon_path = config_icon_path.to_owned();
    Some(jobs::start(move || {
        let result = match catch_panic(|| gags::load_gags_config(&config_path, &config_json, &config_icon_path)) {
            Ok(o) => match o {
                Ok(o) => o,
                Err(e) => e.to_string()
            },
            Err(e) => e.to_string()
        };
        frame!();
        result
    }))
});

byond_fn!(fn iconforge_gags(config_path, colors, output_dmi_path) {
    let config_path = config_path.to_owned();
    let colors = colors.to_owned();
    let output_dmi_path = output_dmi_path.to_owned();
    let result = Some(match catch_panic(|| gags::gags(&config_path, &colors, &output_dmi_path)) {
        Ok(o) => match o {
            Ok(o) => o,
            Err(e) => e.to_string()
        },
        Err(e) => e.to_string()
    });
    frame!();
    result
});

byond_fn!(fn iconforge_gags_async(config_path, colors, output_dmi_path) {
    let config_path = config_path.to_owned();
    let colors = colors.to_owned();
    let output_dmi_path = output_dmi_path.to_owned();
    Some(jobs::start(move || {
        let result = match catch_panic(|| gags::gags(&config_path, &colors, &output_dmi_path)) {
            Ok(o) => match o {
                Ok(o) => o,
                Err(e) => e.to_string()
            },
            Err(e) => e.to_string()
        };
        frame!();
        result
    }))
});
