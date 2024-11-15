use core::ffi::c_void;

use windows_sys::Wdk::{
    Foundation::MDL,
    System::SystemServices::{KernelMode, MmCached, MmMapLockedPagesSpecifyCache},
};

bitflags::bitflags! {
    pub struct MdlFlags: i16 {
        const MAPPED_TO_SYSTEM_VA = 0x0001;
        const PAGES_LOCKED = 0x0002;
        const SOURCE_IS_NONPAGED_POOL = 0x0004;
        const ALLOCATED_FIXED_SIZE = 0x0008;
        const PARTIAL = 0x0010;
        const PARTIAL_HAS_BEEN_MAPPED = 0x0020;
        const IO_PAGE_READ = 0x0040;
        const WRITE_OPERATION = 0x0080;
        const LOCKED_PAGE_TABLES = 0x0100;
        const PARENT_MAPPED_SYSTEM_VA = Self::LOCKED_PAGE_TABLES.bits();
        const FREE_EXTRA_PTES = 0x0200;
        const DESCRIBES_AWE = 0x0400;
        const IO_SPACE = 0x0800;
        const NETWORK_HEADER = 0x1000;
        const MAPPING_CAN_FAIL = 0x2000;
        const PAGE_CONTENTS_INVARIANT = 0x4000;
        const ALLOCATED_MUST_SUCCEED = Self::PAGE_CONTENTS_INVARIANT.bits();

        const MAPPING_FLAGS = Self::MAPPED_TO_SYSTEM_VA.bits()
                            | Self::PAGES_LOCKED.bits()
                            | Self::SOURCE_IS_NONPAGED_POOL.bits()
                            | Self::PARTIAL_HAS_BEEN_MAPPED.bits()
                            | Self::PARENT_MAPPED_SYSTEM_VA.bits()
                            | Self::IO_SPACE.bits();
    }
}

unsafe fn mm_get_system_address_for_mdl_safe(
    mdl: &mut MDL,
    priority: u32, // MM_PAGE_PRIORITY logically OR'd with MdlMapping*
) -> *mut c_void {
    const GET_SYSTEM_ADDR_FLAGS: i16 =
        MdlFlags::MAPPED_TO_SYSTEM_VA.bits() | MdlFlags::SOURCE_IS_NONPAGED_POOL.bits();
    if (mdl.MdlFlags & GET_SYSTEM_ADDR_FLAGS) == GET_SYSTEM_ADDR_FLAGS {
        mdl.MappedSystemVa
    } else {
        MmMapLockedPagesSpecifyCache(
            mdl,
            KernelMode as _,
            MmCached,
            core::ptr::null(),
            false as _,
            priority,
        )
    }
}

#[repr(transparent)]
pub struct Mdl(&'static mut MDL);

impl Mdl {
    pub unsafe fn from_raw(mdl: *mut MDL) -> Self {
        unsafe { Self(&mut *mdl) }
    }

    pub unsafe fn raw(&self) -> *mut MDL {
        let ptr: *const MDL = self.0;

        ptr as _
    }

    pub fn system_address(&self, priority: u32) -> *mut c_void {
        unsafe {
            let ptr: *mut MDL = (self.0 as *const MDL) as _;
            mm_get_system_address_for_mdl_safe(&mut *ptr, priority)
        }
    }
}
