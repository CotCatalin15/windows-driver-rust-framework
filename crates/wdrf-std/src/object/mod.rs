pub mod handle;
pub mod object;

use wdk_sys::{
    ExEventObjectType, ExSemaphoreObjectType, IoFileObjectType, PsProcessType, PsThreadType,
    SeTokenObjectType, TmEnlistmentObjectType, TmResourceManagerObjectType,
    TmTransactionManagerObjectType, TmTransactionObjectType, POBJECT_TYPE,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KernelObjectType {
    Event,
    Semaphore,
    File,
    Process,
    Thread,
    Token,
    Enlistment,
    ResourceManager,
    TranscationManager,
    Transcation,
}

impl KernelObjectType {
    pub fn into_kernel_object_type(self) -> POBJECT_TYPE {
        unsafe {
            match self {
                KernelObjectType::Event => *ExEventObjectType,
                KernelObjectType::Semaphore => *ExSemaphoreObjectType,
                KernelObjectType::File => *IoFileObjectType,
                KernelObjectType::Process => *PsProcessType,
                KernelObjectType::Thread => *PsThreadType,
                KernelObjectType::Token => *SeTokenObjectType,
                KernelObjectType::Enlistment => *TmEnlistmentObjectType,
                KernelObjectType::ResourceManager => *TmResourceManagerObjectType,
                KernelObjectType::TranscationManager => *TmTransactionManagerObjectType,
                KernelObjectType::Transcation => *TmTransactionObjectType,
            }
        }
    }
}
