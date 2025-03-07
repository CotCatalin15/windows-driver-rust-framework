use core::time::Duration;

use windows_sys::Wdk::System::SystemServices::KeQuerySystemTimePrecise;

use crate::kmalloc::{MemoryTag, TaggedObject};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Timeout {
    timeout: Option<i64>,
}

impl Timeout {
    pub fn infinite() -> Self {
        Self { timeout: None }
    }

    pub fn relative(relative: u64) -> Self {
        Self {
            timeout: Some(relative as i64),
        }
    }

    pub fn timeout(timeout_in_ns: u64) -> Self {
        Self {
            timeout: Some((timeout_in_ns / 100) as i64),
        }
    }

    pub fn from_duration(duration: Duration) -> Self {
        Self::timeout(duration.as_nanos() as u64)
    }

    pub fn as_ptr(&self) -> *const i64 {
        if let Some(ref timeout) = self.timeout {
            timeout
        } else {
            core::ptr::null()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct SystemTime {
    time: u64,
}

impl TaggedObject for SystemTime {
    fn tag() -> crate::kmalloc::MemoryTag {
        MemoryTag::new_from_bytes(b"syst")
    }
}

impl SystemTime {
    pub fn new() -> Self {
        Self {
            time: Self::get_precise(),
        }
    }

    pub fn update(&mut self) {
        self.time = Self::get_precise();
    }

    pub fn raw_time(&self) -> u64 {
        self.time
    }

    #[inline]
    pub fn elapsed_raw(&self) -> u64 {
        Self::get_precise() - self.time
    }

    #[inline]
    pub fn elapsed_duration(&self) -> Duration {
        Duration::from_nanos(self.elapsed_raw() * 100)
    }

    fn get_precise() -> u64 {
        let mut time: u64 = 0;
        unsafe {
            let ptr_time: *mut u64 = &mut time;
            KeQuerySystemTimePrecise(ptr_time as _);
        }
        return time;
    }
}
