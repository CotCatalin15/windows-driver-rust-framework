use windows_sys::Wdk::Foundation::POBJECT_TYPE;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PoolFlags : u64 {
        const POOL_FLAG_REQUIRED_START = 0x0000000000000001;
        const POOL_FLAG_USE_QUOTA= 0x0000000000000001;
        const POOL_FLAG_UNINITIALIZED= 0x0000000000000002;
        const POOL_FLAG_SESSION = 0x0000000000000004;
        const POOL_FLAG_CACHE_ALIGNED = 0x0000000000000008;
        const POOL_FLAG_RESERVED1 = 0x0000000000000010;
        const POOL_FLAG_RAISE_ON_FAILURE = 0x0000000000000020;
        const POOL_FLAG_NON_PAGED = 0x0000000000000040;
        const POOL_FLAG_NON_PAGED_EXECUTE = 0x0000000000000080;
        const POOL_FLAG_PAGED = 0x0000000000000100;
        const POOL_FLAG_RESERVED2 = 0x0000000000000200;
        const POOL_FLAG_RESERVED3 = 0x0000000000000400;
        const POOL_FLAG_REQUIRED_END = 0x0000000080000000;
        const POOL_FLAG_OPTIONAL_START = 0x0000000100000000;
        const POOL_FLAG_SPECIAL_POOL = 0x0000000100000000;
        const POOL_FLAG_OPTIONAL_END = 0x8000000000000000;
    }
}

extern "C" {
    pub static mut IoFileObjectType: *mut POBJECT_TYPE;
}
extern "C" {
    pub static mut ExEventObjectType: *mut POBJECT_TYPE;
}
extern "C" {
    pub static mut ExSemaphoreObjectType: *mut POBJECT_TYPE;
}
extern "C" {
    pub static mut TmTransactionManagerObjectType: *mut POBJECT_TYPE;
}
extern "C" {
    pub static mut TmResourceManagerObjectType: *mut POBJECT_TYPE;
}
extern "C" {
    pub static mut TmEnlistmentObjectType: *mut POBJECT_TYPE;
}
extern "C" {
    pub static mut TmTransactionObjectType: *mut POBJECT_TYPE;
}
extern "C" {
    pub static mut PsProcessType: *mut POBJECT_TYPE;
}
extern "C" {
    pub static mut PsThreadType: *mut POBJECT_TYPE;
}
extern "C" {
    pub static mut PsJobType: *mut POBJECT_TYPE;
}
extern "C" {
    pub static mut SeTokenObjectType: *mut POBJECT_TYPE;
}
