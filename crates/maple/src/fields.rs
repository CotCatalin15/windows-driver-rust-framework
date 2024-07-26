use core::fmt::{Arguments, Debug};

use crate::consumer::get_global_registry;

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
    pub module: &'static str,
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
    pub fn meta(&self) -> &'a Metadata {
        self.meta
    }

    pub fn args(&'a self) -> &'a Arguments<'a> {
        &self.args
    }
}

impl<'a> Event<'a> {
    pub fn dispatch(meta: &'a Metadata, args: Arguments<'a>) {
        let event = Event { meta, args };

        get_global_registry().consumer().event(&event);
    }
}
