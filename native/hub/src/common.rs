use anyhow::Result;
use rinf::debug_print;

pub fn check_err<T>(result: Result<T>) -> Result<T> {
    if let Err(e) = &result {
        debug_print!("{e:?}");
    }
    result
}
