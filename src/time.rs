use std::time::{Duration, SystemTime, SystemTimeError};
use std::{
    cell::RefCell,
    collections::hash_map::{Entry, HashMap},
    time::Instant,
};

use byond_fn::byond_fn;

thread_local!( static INSTANTS: RefCell<HashMap<String, Instant>> = RefCell::new(HashMap::new()) );

fn time(instant_id: String) -> Duration {
    INSTANTS.with(|instants| {
        let mut map = instants.borrow_mut();
        let instant = match map.entry(instant_id.into()) {
            Entry::Occupied(elem) => elem.into_mut(),
            Entry::Vacant(elem) => elem.insert(Instant::now()),
        };
        instant.elapsed()
    })
}

#[byond_fn]
fn time_microseconds(instant_id: String) -> String {
    time(instant_id).as_micros().to_string()
}

#[byond_fn]
fn time_milliseconds(instant_id: String) -> String {
    time(instant_id).as_millis().to_string()
}

#[byond_fn]
fn time_reset(instant_id: String) {
    INSTANTS.with(|instants| {
        let mut map = instants.borrow_mut();
        map.insert(instant_id.into(), Instant::now());
    })
}

#[byond_fn]
fn unix_timestamp() -> Result<String, SystemTimeError> {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|x| x.as_secs_f64())
        .map(|x| x.to_string())
}
