use std::{cell::RefCell, fs::OpenOptions, io::Write};

use chrono::Utc;

thread_local! {
    static LAST_BYOND_FN: RefCell<Option<String>> = RefCell::new(None);
}

pub fn write_to_error_log(contents: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("rustg_panic.log")
        .unwrap();

    writeln!(file, "[{}] {}", Utc::now().format("%F %T%.3f"), contents).ok();
}

pub fn set_last_byond_fn(name: &str) {
    // Be overly cautious because anything that happens in this file is all about caring about stuff that shouldn't happen
    if LAST_BYOND_FN
        .try_with(|cell| match cell.try_borrow_mut() {
            Ok(mut cell) => {
                *cell = Some(name.to_owned());
            }

            Err(_) => {
                write_to_error_log("Failed to borrow LAST_BYOND_FN");
            }
        })
        .is_err()
    {
        write_to_error_log("Failed to access LAST_BYOND_FN");
    }
}

#[ctor::ctor]
fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let mut message = "global panic hook triggered: ".to_owned();

        if let Some(location) = panic_info.location() {
            message.push_str(&format!("{}:{}: ", location.file(), location.line()));
        }

        if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
            message.push_str(payload);
        } else if let Some(payload) = panic_info.payload().downcast_ref::<String>() {
            message.push_str(payload);
        } else {
            message.push_str("unknown panic");
        }

        LAST_BYOND_FN.with(|cell| match cell.try_borrow() {
            Ok(cell) => {
                if let Some(last_byond_fn) = &*cell {
                    message.push_str(&format!(" (last byond fn: {})", last_byond_fn));
                }
            }

            Err(_) => {
                message.push_str(" (failed to get last byond fn)");
            }
        });

        write_to_error_log(&message);
    }));
}
