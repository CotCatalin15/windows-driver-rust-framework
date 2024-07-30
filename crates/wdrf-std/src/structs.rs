use windows_sys::Wdk::{
    Foundation::{FILE_OBJECT, KEVENT},
    System::SystemServices::{KSEMAPHORE, PROCESS_ACCESS_TOKEN},
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _KPROCESS {
    _unused: [u8; 0],
}
pub type PEPROCESS = *mut _KPROCESS;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _KTHREAD {
    _unused: [u8; 0],
}
pub type PETHREAD = *mut _KTHREAD;

pub type PKTHREAD = *mut _KTHREAD;
pub type PRKTHREAD = *mut _KTHREAD;
pub type PKPROCESS = *mut _KPROCESS;
pub type PRKPROCESS = *mut _KPROCESS;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _OBJECT_TYPE {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _KTRANSACTION {
    _unused: [u8; 0],
}
pub type KTRANSACTION = _KTRANSACTION;
pub type PKTRANSACTION = *mut _KTRANSACTION;
pub type PRKTRANSACTION = *mut _KTRANSACTION;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _KENLISTMENT {
    _unused: [u8; 0],
}
pub type KENLISTMENT = _KENLISTMENT;
pub type PKENLISTMENT = *mut _KENLISTMENT;
pub type PRKENLISTMENT = *mut _KENLISTMENT;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _KRESOURCEMANAGER {
    _unused: [u8; 0],
}
pub type KRESOURCEMANAGER = _KRESOURCEMANAGER;
pub type PKRESOURCEMANAGER = *mut _KRESOURCEMANAGER;
pub type PRKRESOURCEMANAGER = *mut _KRESOURCEMANAGER;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _KTM {
    _unused: [u8; 0],
}
pub type KTM = _KTM;
pub type PKTM = *mut _KTM;

#[allow(non_camel_case_types)]
pub type PKEVENT = *mut KEVENT;
#[allow(non_camel_case_types)]
pub type PKSEMAPHORE = *mut KSEMAPHORE;
#[allow(non_camel_case_types)]
pub type PFILE_OBJECT = *mut FILE_OBJECT;
#[allow(non_camel_case_types)]
pub type PPROCESS_ACCESS_TOKEN = *mut PROCESS_ACCESS_TOKEN;
