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

#[allow(unused_variables)]
pub trait FilterOperationVisitor: Send + Sync + 'static {
    fn instance_setup<'a>(
        &self,
        related: FltRelatedObjects<'a>,
        flags: u32,
        device_type: FltDeviceType,
        volume_type: FltVolumeType,
    ) -> InstanceSetupStatus {
        InstanceSetupStatus::Success
    }

    fn unload(&self, mandatory: bool) -> UnloadStatus {
        UnloadStatus::Unload
    }
}
