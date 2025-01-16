pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err(From::from(format!($($arg)*)))
    };
}
