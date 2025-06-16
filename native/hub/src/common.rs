use anyhow::Result;
use rinf::debug_print;
use std::time::Duration;

pub const DATE_FILE_FMT: &str = "%F";

pub fn check_err<T>(result: Result<T>) -> Result<T> {
    if let Err(e) = &result {
        debug_print!("{e:?}");
    }
    result
}

pub fn condense_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    if seconds > 0 {
        format!("{seconds} s")
    } else {
        let millis = duration.as_millis();
        format!("{millis} ms")
    }
}
