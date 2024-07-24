#[macro_export]
macro_rules! call_location {
    ($level:expr) => {
        $crate::fields::Metadata {
            level: $level,
            file: file!(),
            line: line!(),
            module: module_path!(),
            colum: column!(),
            name: None,
        }
    };
    ($level:expr, name = $name:expr) => {
        $crate::fields::Metadata {
            level: $level,
            file: file!(),
            line: line!(),
            module: module_path!(),
            colum: column!(),
            name: Some($name),
        }
    };
}

#[macro_export]
macro_rules! event {
    ($level:expr, name = $name:expr, result = $rez:expr, $($arg:tt)*) => {
        if $crate::consumer::get_global_registry().enabled() {
            let meta = $crate::call_location!($level, name = $name);
            if $crate::consumer::get_global_registry().should_log_event(&meta) {
                let _ = $rez.as_ref().inspect_err(|e| {
                    $crate::fields::Event::dispatch(&meta, format_args!("Result error: {:#?}, event: {:#?}", e, format_args!($($arg)*)));
                });
            }
        }
    };
    ($level:expr, result = $rez:expr, $($arg:tt)*) => {
        if $crate::consumer::get_global_registry().enabled() {
            let meta = $crate::call_location!($level);
            if $crate::consumer::get_global_registry().should_log_event(&meta) {
                let _ = $rez.as_ref().inspect_err(|e| {
                    $crate::fields::Event::dispatch(&meta, format_args!("Result error: {:#?}, event: {:#?}", e, format_args!($($arg)*)));
                });
            }
        }
    };
    ($level:expr, name = $name:expr, $($arg:tt)*) => {
        if $crate::consumer::get_global_registry().enabled() {
            let meta = $crate::call_location!($level, name = $name);
            if $crate::consumer::get_global_registry().should_log_event(&meta) {
                $crate::fields::Event::dispatch(&meta, format_args!($($arg)*));
            }
        }
    };
    ($level:expr, $($arg:tt)*) => {
        if $crate::consumer::get_global_registry().enabled() {
            let meta = $crate::call_location!($level);
            if $crate::consumer::get_global_registry().should_log_event(&meta) {
                $crate::fields::Event::dispatch(&meta, format_args!($($arg)*));
            }
        }
    };
}

#[macro_export]
macro_rules! trace {
    (name = $name:expr, result = $rez:expr, $($arg:tt)*) => {
       $crate::event!($crate::fields::Level::Trace, name = $name, result =$rez, $($arg)*);
    };
    (result = $rez:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Trace, result =$rez, $($arg)*);
    };
    (name = $name:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Trace, name = $name, $($arg)*);
    };
    ($($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Trace, $($arg)*);
    };
}

#[macro_export]
macro_rules! info {
    (name = $name:expr, result = $rez:expr, $($arg:tt)*) => {
       $crate::event!($crate::fields::Level::Info, name = $name, result =$rez, $($arg)*);
    };
    (result = $rez:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Info, result =$rez, $($arg)*);
    };
    (name = $name:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Info, name = $name, $($arg)*);
    };
    ($($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Info, $($arg)*);
    };
}

#[macro_export]
macro_rules! warn {
    (name = $name:expr, result = $rez:expr, $($arg:tt)*) => {
       $crate::event!($crate::fields::Level::Warn, name = $name, result =$rez, $($arg)*);
    };
    (result = $rez:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Warn, result =$rez, $($arg)*);
    };
    (name = $name:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Warn, name = $name, $($arg)*);
    };
    ($($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Warn, $($arg)*);
    };
}

#[macro_export]
macro_rules! error {
    (name = $name:expr, result = $rez:expr, $($arg:tt)*) => {
       $crate::event!($crate::fields::Level::Error, name = $name, result =$rez, $($arg)*);
    };
    (result = $rez:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Error, result =$rez, $($arg)*);
    };
    (name = $name:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Error, name = $name, $($arg)*);
    };
    ($($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Error, $($arg)*);
    };
}

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug {
    (name = $name:expr, result = $rez:expr, $($arg:tt)*) => {
       $crate::event!($crate::fields::Level::Debug, name = $name, result =$rez, $($arg)*);
    };
    (result = $rez:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Debug, result =$rez, $($arg)*);
    };
    (name = $name:expr, $($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Debug, name = $name, $($arg)*);
    };
    ($($arg:tt)*) => {
        $crate::event!($crate::fields::Level::Debug, $($arg)*);
    };
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {};
}
