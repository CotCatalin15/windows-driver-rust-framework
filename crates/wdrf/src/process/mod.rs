mod structs;

use core::alloc::Layout;

pub use structs::*;
use wdrf_std::{
    boxed::{Box, BoxExt},
    constants::PoolFlags,
    kmalloc::{alloc, GlobalKernelAllocator, MemoryTag},
    nt_success,
    object::{ArcKernelObj, NonNullKrnlResource},
    structs::{PEPROCESS, PKPROCESS},
    NtStatusError,
};
use windows_sys::Wdk::Storage::FileSystem::{
    KeStackAttachProcess, KeUnstackDetachProcess, PsLookupProcessByProcessId, KAPC_STATE,
};

pub mod notifier;
pub mod process_create_notifier;

#[derive(Debug)]
pub enum ProcessCollectorError {
    NoMemory,
    ContextRegisterError,
    NtStatus(NtStatusError),
}

pub fn ps_lookup_by_process_id(pid: isize) -> Option<ArcKernelObj<PKPROCESS>> {
    unsafe {
        let mut process_as_isize: isize = 0;

        let status = PsLookupProcessByProcessId(pid, &mut process_as_isize);
        if !nt_success(status) {
            return None;
        }

        let process: PEPROCESS = process_as_isize as *mut _;
        let process = NonNullKrnlResource::new(process);

        process.map(|eprocess| ArcKernelObj::new(eprocess, false))
    }
}

pub fn ke_stack_attach_process<R, CB: FnOnce() -> R>(
    process: &ArcKernelObj<PEPROCESS>,
    fnc: CB,
) -> anyhow::Result<R> {
    unsafe {
        let allocator = GlobalKernelAllocator::new(
            MemoryTag::new_from_bytes(b"kapc"),
            PoolFlags::POOL_FLAG_NON_PAGED,
        );
        let mut kapc_state: Box<KAPC_STATE> = Box::try_create_in(core::mem::zeroed(), allocator)?;

        KeStackAttachProcess(process.as_raw_obj() as isize, kapc_state.as_mut());

        let ret = fnc();

        KeUnstackDetachProcess(kapc_state.as_ref());

        Ok(ret)
    }
}
