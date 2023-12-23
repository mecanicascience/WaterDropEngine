extern crate tracing;

// Export error macro
#[macro_export]
macro_rules! throw {
    ($($arg:tt)*) => ({
        use wde_logger::error;

        wde_logger::error!($($arg)*);
        panic!($($arg)*);
    })
}