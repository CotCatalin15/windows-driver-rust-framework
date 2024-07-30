use windows_sys::Win32::Storage::FileSystem::STANDARD_RIGHTS_ALL;

pub const IRP_MJ_OPERATION_END: u32 = 0x80;

pub const FLT_PORT_CONNECT: u32 = 0x0001;
pub const FLT_PORT_ALL_ACCESS: u32 = FLT_PORT_CONNECT | STANDARD_RIGHTS_ALL;
