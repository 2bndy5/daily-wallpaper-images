use anyhow::Result;
use rinf::debug_print;

pub const DATE_FILE_FMT: &str = "%F";

pub fn check_err<T>(result: Result<T>) -> Result<T> {
    if let Err(e) = &result {
        debug_print!("{e:?}");
    }
    result
}
