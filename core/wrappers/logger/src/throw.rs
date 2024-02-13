extern crate tracing;

// Macro throw! is used to log error messages and panic.
#[macro_export]
macro_rules! throw {
    ($($arg:tt)*) => ({
        use wde_logger::error;

        wde_logger::error!($($arg)*);
        panic!($($arg)*);
    })
}