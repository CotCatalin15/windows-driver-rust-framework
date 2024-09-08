use core::time::Duration;

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
