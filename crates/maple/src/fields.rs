use core::fmt::Arguments;

use crate::consumer::get_global;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
pub enum Level {
    #[cfg(feature = "debug")]
    Debug,

    Trace,
    Info,
    Warn,
    Error,
}

#[derive(Debug)]
pub struct Metadata {
    pub level: Level,
    pub file: &'static str,
    pub line: u32,
    pub colum: u32,
    pub name: Option<&'static str>,
}

#[derive(Debug)]
pub struct Event<'a> {
    meta: &'a Metadata,
    args: Arguments<'a>,
}

impl<'a> Event<'a> {
    pub fn dispatch(meta: &'a Metadata, args: Arguments<'a>) {
        let event = Event {
            meta: meta,
            args: args,
        };

        get_global().event(&event);
    }
}
