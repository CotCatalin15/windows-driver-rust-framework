use core::cell::UnsafeCell;

use wdrf_std::{
    hashbrown::{HashMap, HashMapExt, OccupiedError},
    kmalloc::TaggedObject,
    structs::{PEPROCESS, PKPROCESS},
    sync::{
        arc::{Arc, ArcExt},
        rwlock::ExRwLock,
    },
    traits::DispatchSafe,
    NtResult, NtResultEx,
};
use windows_sys::{
    Wdk::System::SystemServices::{PsSetCreateProcessNotifyRoutineEx, PS_CREATE_NOTIFY_INFO},
    Win32::Foundation::HANDLE,
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
        create_info: &PS_CREATE_NOTIFY_INFO,
    ) -> anyhow::Result<Self::Item>;

    fn on_process_destroy(&mut self, item: &Self::Item);
}

pub struct ProcessRegistry<H: ProcessHook + DispatchSafe + Send> {
    started: UnsafeCell<bool>,
    hook: H,

    processes: ExRwLock<HashMap<u64, Arc<H::Item>>>,
}

unsafe impl<H: ProcessHook + DispatchSafe + Send> DispatchSafe for ProcessRegistry<H> {}

impl<H: ProcessHook + DispatchSafe + Send> ProcessRegistry<H> {
    pub fn new(hook: H) -> Self {
        Self {
            started: UnsafeCell::new(false),
            hook,
            processes: ExRwLock::new(HashMap::create()),
        }
    }

    pub fn start_collector(&'static mut self) -> NtResult<()> {
        unsafe {
            if *self.started.get() {
                return Ok(());
            }

            let status = PsSetCreateProcessNotifyRoutineEx(
                Some(create_process_notify_implementation),
                false as _,
            );
            NtResult::from_status(status, || ())?;
            *self.started.get() = true;
            GLOBAL_PROCESS_COLLECTOR = Some(self);
        }

        Ok(())
    }

    pub fn stop_collector(&self) -> NtResult<()> {
        unsafe {
            if *self.started.get() {
                GLOBAL_PROCESS_COLLECTOR = None;
                let status = PsSetCreateProcessNotifyRoutineEx(
                    Some(create_process_notify_implementation),
                    true as _,
                );
                NtResult::from_status(status, || ())?;
            }
            *self.started.get() = false;
        }

        Ok(())
    }

    pub fn find_by_pid(&self, process_id: u64) -> Option<Arc<H::Item>> {
        let guard = self.processes.read();

        guard.get(&process_id).cloned()
    }
}

impl<H: ProcessHook + Send + DispatchSafe> Drop for ProcessRegistry<H> {
    fn drop(&mut self) {
        let _ = self.stop_collector();
    }
}

trait ProcessCollectorCallbacks {
    fn on_process_create(
        &mut self,
        process: PEPROCESS,
        process_id: HANDLE,
        create_info: &PS_CREATE_NOTIFY_INFO,
    ) -> anyhow::Result<()>;

    fn on_process_destroy(&mut self, process: PEPROCESS, process_id: HANDLE);
}

impl<H: ProcessHook + Send + DispatchSafe> ProcessCollectorCallbacks for ProcessRegistry<H> {
    fn on_process_create(
        &mut self,
        process: PEPROCESS,
        process_id: HANDLE,
        create_info: &PS_CREATE_NOTIFY_INFO,
    ) -> anyhow::Result<()> {
        let item = self
            .hook
            .on_process_create(process, process_id, create_info)?;

        let item = Arc::try_create(item)?;

        let mut guard = self.processes.write();
        let occupation = guard.try_insert(process_id as _, item);

        if let Err(OccupiedError { mut entry, value }) = occupation {
            entry.insert(value);
        }

        Ok(())
    }

    fn on_process_destroy(&mut self, _process: PEPROCESS, process_id: HANDLE) {
        let mut guard = self.processes.write();

        let process_id: u64 = process_id as _;
        guard.remove(&process_id);
    }
}

static mut GLOBAL_PROCESS_COLLECTOR: Option<&'static mut dyn ProcessCollectorCallbacks> = None;

unsafe extern "system" fn create_process_notify_implementation(
    process: HANDLE,
    process_id: HANDLE,
    create_info: *mut PS_CREATE_NOTIFY_INFO,
) {
    let process: PKPROCESS = process as _;
    if let Some(ref mut callbacks) = GLOBAL_PROCESS_COLLECTOR {
        if create_info.is_null() {
            callbacks.on_process_destroy(process, process_id);
        } else {
            let _ = callbacks.on_process_create(process, process_id, &*create_info);
        }
    }
}
