use core::fmt::Arguments;

#[macro_export]
macro_rules! call_location {
    ($level:expr) => {
        $crate::fields::Metadata {
            level: $level,
            file: file!(),
            line: line!(),
            colum: column!(),
            name: None,
        }
    };
    ($level:expr, name = $name:expr) => {
        $crate::fields::Metadata {
            level: $level,
            file: file!(),
            line: line!(),
            colum: column!(),
            name: Some($name),
        }
    };
}

#[macro_export]
macro_rules! event {
    ($level:expr, name = $name:expr, $($arg:tt)*) => {
        if $crate::consumer::is_consumer_enabled() {
            let meta = $crate::call_location!($level, name = $name);
            if $crate::consumer::is_filter_pass(&meta) {
                $crate::fields::Event::dispatch(&meta, format_args!($($arg)*));
            }
        }
    };
    ($level:expr, $($arg:tt)*) => {
        if $crate::consumer::is_consumer_enabled() {
            let meta = $crate::call_location!($level);
            if $crate::consumer::is_filter_pass(&meta) {
                $crate::fields::Event::dispatch(&meta, format_args!($($arg)*));
            }
        }
    };
}

#[macro_export]
macro_rules! trace {
    (name = $name:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Trace, name = $name, $($arg)*);
    };
    ($($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Trace, $($arg)*);
    };
}

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug {
    (name = $name:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Debug, name = $name, $($arg)*);
    };
    ($($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Debug, $($arg)*);
    };
}

#[macro_export]
macro_rules! info {
    (name = $name:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Info, name = $name, $($arg)*);
    };
    ($($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Info, $($arg)*);
    };
}

#[macro_export]
macro_rules! warn {
    (name = $name:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Warn, name = $name, $($arg)*);
    };
    ($($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Warn, $($arg)*);
    };
}

#[macro_export]
macro_rules! error {
    (name = $name:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Error, name = $name, $($arg)*);
    };
    ($($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Error, $($arg)*);
    };
}
