mod minifilter_builder;
mod operation_factory;

pub use minifilter_builder::*;
pub use operation_factory::*;

use windows_sys::Wdk::Storage::FileSystem::Minifilters::FLT_OPERATION_REGISTRATION;

pub trait IntoFltOpRegistrationFactory {
    fn into_operations(&mut self) -> &[FLT_OPERATION_REGISTRATION];
}
