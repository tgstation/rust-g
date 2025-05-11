use chrono::{FixedOffset, Local, Utc};
use std::{
    cell::RefCell,
    collections::hash_map::{Entry, HashMap},
    time::Instant,
};

thread_local!( static INSTANTS: RefCell<HashMap<String, Instant>> = RefCell::new(HashMap::new()) );

byond_fn!(fn time_microseconds(instant_id) {
    INSTANTS.with(|instants| {
        let mut map = instants.borrow_mut();
        let instant = match map.entry(instant_id.into()) {
            Entry::Occupied(elem) => elem.into_mut(),
            Entry::Vacant(elem) => elem.insert(Instant::now()),
        };
        Some(instant.elapsed().as_micros().to_string())
    })
});

byond_fn!(fn time_milliseconds(instant_id) {
    INSTANTS.with(|instants| {
        let mut map = instants.borrow_mut();
        let instant = match map.entry(instant_id.into()) {
            Entry::Occupied(elem) => elem.into_mut(),
            Entry::Vacant(elem) => elem.insert(Instant::now()),
        };
        Some(instant.elapsed().as_millis().to_string())
    })
});

byond_fn!(fn time_reset(instant_id) {
    INSTANTS.with(|instants| {
        let mut map = instants.borrow_mut();
        map.insert(instant_id.into(), Instant::now());
        Some("")
    })
});

byond_fn!(
    fn unix_timestamp() {
        Some(format!(
            "{:.6}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        ))
    }
);

byond_fn!(
    fn formatted_timestamp(format, offset) {
        format_timestamp(format, offset)
    }
);

fn format_timestamp(format: &str, offset: &str) -> Option<String> {
    if offset.is_empty() {
        Some(Local::now().format(format).to_string())
    } else {
        let offset_seconds = offset.parse::<i32>().ok()? * 3600;
        let timezone = FixedOffset::east_opt(offset_seconds)?;
        Some(
            Utc::now()
                .with_timezone(&timezone)
                .format(format)
                .to_string(),
        )
    }
}
