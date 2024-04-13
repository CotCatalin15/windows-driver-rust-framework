use core::cell::UnsafeCell;

use wdk_sys::{
    ntddk::PsSetCreateProcessNotifyRoutineEx, HANDLE, PEPROCESS, PPS_CREATE_NOTIFY_INFO,
    _PS_CREATE_NOTIFY_INFO,
};
use wdrf_std::{
    hashbrown::{HashMap, HashMapExt, OccupiedError},
    kmalloc::TaggedObject,
    sync::mutex::GuardedMutex,
    NtResultEx, Result,
};

pub trait ProcessDescriptor {
    fn pid(&self) -> u64;
}

pub trait ProcessHook {
    type Item: ProcessDescriptor + TaggedObject;

    fn on_process_create(
        &mut self,
        process: PEPROCESS,
        process_id: HANDLE,
        create_info: &mut _PS_CREATE_NOTIFY_INFO,
    ) -> anyhow::Result<Self::Item>;

    fn on_process_destroy(&mut self, item: &Self::Item);
}

pub struct ProcessRegistry<H: ProcessHook + Send> {
    started: UnsafeCell<bool>,
    hook: H,

    ///TODO: Maybe make it a spin lock so it works at Dispatch ?
    processes: GuardedMutex<HashMap<u64, H::Item>>,
}

impl<H: ProcessHook + Send> ProcessRegistry<H> {
    pub fn new(hook: H) -> Self {
        Self {
            started: UnsafeCell::new(false),
            hook,
            processes: GuardedMutex::new(HashMap::create()),
        }
    }

    pub fn start_collector(&'static mut self) -> Result {
        unsafe {
            if *self.started.get() {
                return Ok(());
            }

            let status = PsSetCreateProcessNotifyRoutineEx(
                Some(create_process_notify_implementation),
                false as _,
            );
            Result::from_status(status)?;
            *self.started.get() = true;
            GLOBAL_PROCESS_COLLECTOR = Some(self);
        }

        Ok(())
    }

    pub fn stop_collector(&self) -> Result {
        unsafe {
            if *self.started.get() {
                GLOBAL_PROCESS_COLLECTOR = None;
                let status = PsSetCreateProcessNotifyRoutineEx(
                    Some(create_process_notify_implementation),
                    true as _,
                );
                Result::from_status(status)?;
            }
            *self.started.get() = false;
        }

        Ok(())
    }
}

impl<H: ProcessHook + Send> Drop for ProcessRegistry<H> {
    fn drop(&mut self) {
        let _ = self.stop_collector();
    }
}

trait ProcessCollectorCallbacks {
    fn on_process_create(
        &mut self,
        process: PEPROCESS,
        process_id: HANDLE,
        create_info: PPS_CREATE_NOTIFY_INFO,
    ) -> anyhow::Result<()>;

    fn on_process_destroy(&mut self, process: PEPROCESS, process_id: HANDLE);
}

impl<H: ProcessHook + Send> ProcessCollectorCallbacks for ProcessRegistry<H> {
    fn on_process_create(
        &mut self,
        process: PEPROCESS,
        process_id: HANDLE,
        create_info: PPS_CREATE_NOTIFY_INFO,
    ) -> anyhow::Result<()> {
        let create_info = unsafe { &mut *create_info };

        let item = self
            .hook
            .on_process_create(process, process_id, create_info)?;

        let mut guard = self.processes.lock();
        let occupation = guard.try_insert(process_id as _, item);

        if let Err(OccupiedError { mut entry, value }) = occupation {
            entry.insert(value);
        }

        Ok(())
    }

    fn on_process_destroy(&mut self, _process: PEPROCESS, process_id: HANDLE) {
        let mut guard = self.processes.lock();

        let process_id: u64 = process_id as _;
        guard.remove(&process_id);
    }
}

static mut GLOBAL_PROCESS_COLLECTOR: Option<&'static mut dyn ProcessCollectorCallbacks> = None;

unsafe extern "C" fn create_process_notify_implementation(
    process: PEPROCESS,
    process_id: HANDLE,
    create_info: PPS_CREATE_NOTIFY_INFO,
) {
    if let Some(ref mut callbacks) = GLOBAL_PROCESS_COLLECTOR {
        if create_info.is_null() {
            callbacks.on_process_destroy(process, process_id);
        } else {
            let _ = callbacks.on_process_create(process, process_id, create_info);
        }
    }
}
