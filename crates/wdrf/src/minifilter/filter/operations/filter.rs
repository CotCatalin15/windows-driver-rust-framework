use crate::minifilter::filter::{FltRelatedObjects, FltVolumeType};

#[derive(Debug, Clone, Copy)]
pub enum UnloadStatus {
    Unload,
    NoDetach,
}

#[derive(Debug, Clone, Copy)]
pub enum InstanceSetupStatus {
    Success,
    DoNotAttach,
}

#[derive(Debug, Clone, Copy)]
pub enum FltDeviceType {
    CdRom,
    Disk,
    Network,
}

pub trait FilterUnload<C>
where
    C: 'static + Sized + Sync + Send,
{
    fn call(minifilter_context: &'static C, mandatory: bool) -> UnloadStatus;
}

#[macro_export]
macro_rules! fn_flt_unload {
    ($fn_name:ident($minifilter_context:ident: $t1:ty, $mandatory:ident: $t2:ty) $body:block) => {
        struct $fn_name;

        impl FilterUnload<C> for $fn_name
        where
            C: 'static + Sized + Sync + Send,
            {
                fn call($minifilter_context: $t1, $mandatory: $t2) $body
            }
    };
}
