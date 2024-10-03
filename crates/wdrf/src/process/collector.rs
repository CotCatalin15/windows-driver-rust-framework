use core::{mem::swap, ops::DerefMut};

use wdrf_std::{
    constants::PoolFlags,
    hashbrown::{HashMap, HashMapExt},
    kmalloc::{GlobalKernelAllocator, MemoryTag, TaggedObject},
    sync::{
        arc::{Arc, ArcExt},
        rwlock::ExRwLock,
    },
    NtStatusError,
};
use windows_sys::Win32::Foundation::{HANDLE, NTSTATUS};

use crate::context::ContextRegistry;

use super::{
    notifier::PsNotifierRegistration,
    process_create_notifier::{try_register_process_notifier, PsCreateNotifyCallback},
    ProcessCollectorError,
};

pub enum ItemRegistrationVerdict<Item> {
    Register(Item),
    NoItem,
    NoRegister,
    Deny(NTSTATUS),
}

pub trait ProcessInfoEvent: Send + Sync + 'static {
    /*
    This event is called when a clear/stop is called on the collector
     */
    fn unregistered(&self);

    /*
    This event is called only when a process terminate is received
     */
    fn destroy(&self);
}

pub trait IProcessItemFactory: Send + Sync + 'static {
    type Item: ProcessInfoEvent;

    fn create(
        &self,
        process: wdrf_std::object::ArcKernelObj<wdrf_std::structs::PEPROCESS>,
        pid: HANDLE,
        process_info: &super::PsCreateNotifyInfo,
    ) -> anyhow::Result<ItemRegistrationVerdict<Self::Item>, ProcessCollectorError>;
}

pub struct ProcessCollector<Factory: IProcessItemFactory> {
    inner: PsNotifierRegistration<ProcessCollectorInner<Factory>>,
}

#[allow(dead_code)]
pub struct ProcessInfo<Item: Send + Sync + 'static> {
    pid: HANDLE, //TODO: Maybe usize ?
    item: Option<Item>,
}

impl<Item: Send + Sync + 'static> TaggedObject for ProcessInfo<Item> {
    fn tag() -> MemoryTag {
        MemoryTag::new_from_bytes(b"pinf")
    }
}

impl<Item: Send + Sync + 'static> ProcessInfo<Item> {
    fn try_create(
        item: Option<Item>,
        _process: wdrf_std::object::ArcKernelObj<wdrf_std::structs::PEPROCESS>,
        pid: HANDLE,
        _process_info: &super::PsCreateNotifyInfo,
    ) -> anyhow::Result<Self> {
        Ok(Self { pid: pid, item })
    }

    pub fn pid(&self) -> HANDLE {
        self.pid
    }

    pub fn item(&self) -> Option<&Item> {
        self.item.as_ref()
    }
}

impl<Factory: IProcessItemFactory> ProcessCollector<Factory> {
    pub fn try_create_with_registry<R: ContextRegistry>(
        registry: &'static R,
        factory: Factory,
    ) -> anyhow::Result<Self, ProcessCollectorError> {
        unsafe {
            let collector = ProcessCollectorInner::try_create(factory)?;
            let inner = PsNotifierRegistration::try_create(collector)?;

            try_register_process_notifier(registry)?;

            Ok(Self { inner })
        }
    }

    pub fn find_by_pid(&self, pid: HANDLE) -> Option<Arc<ProcessInfo<Factory::Item>>> {
        self.inner.find_by_pid(pid)
    }

    pub fn start(&self) -> anyhow::Result<(), ProcessCollectorError> {
        self.inner.try_start()
    }

    pub fn stop(&self) -> anyhow::Result<(), ProcessCollectorError> {
        self.inner.try_stop().inspect(|_| self.clear())
    }

    pub fn clear(&self) {
        self.inner.clear();
    }
}

struct ProcessCollectorInner<Factory: IProcessItemFactory> {
    factory: Factory,
    process_map: ExRwLock<HashMap<isize, Arc<ProcessInfo<Factory::Item>>>>,
}

unsafe impl<Factory: IProcessItemFactory> Send for ProcessCollectorInner<Factory> {}
unsafe impl<Factory: IProcessItemFactory> Sync for ProcessCollectorInner<Factory> {}

impl<Factory: IProcessItemFactory> TaggedObject for ProcessCollectorInner<Factory> {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"pcin")
    }
}

impl<Factory: IProcessItemFactory> ProcessCollectorInner<Factory> {
    fn creat_map() -> HashMap<isize, Arc<ProcessInfo<Factory::Item>>> {
        HashMap::create_in(GlobalKernelAllocator::new(
            MemoryTag::new_from_bytes(b"pshs"),
            PoolFlags::POOL_FLAG_NON_PAGED,
        ))
    }

    fn try_create(factory: Factory) -> anyhow::Result<Self, ProcessCollectorError> {
        Ok(Self {
            factory,
            process_map: ExRwLock::new(Self::creat_map()),
        })
    }

    fn find_by_pid(&self, pid: HANDLE) -> Option<Arc<ProcessInfo<Factory::Item>>> {
        let guard = self.process_map.read();

        guard.get(&pid).cloned()
    }

    fn clear(&self) {
        let mut map = Self::creat_map();

        let mut guard = self.process_map.write();
        swap(&mut map, guard.deref_mut());
        drop(guard);

        map.into_iter().for_each(|(_pid, process)| {
            if let Some(ref item) = process.item {
                item.unregistered();
            }
        });
    }
}

impl<Factory: IProcessItemFactory> PsCreateNotifyCallback for ProcessCollectorInner<Factory> {
    fn on_create(
        &self,
        process: wdrf_std::object::ArcKernelObj<wdrf_std::structs::PEPROCESS>,
        pid: HANDLE,
        process_info: &super::PsCreateNotifyInfo,
    ) -> wdrf_std::NtResult<()> {
        let registration = self
            .factory
            .create(process.clone(), pid, process_info)
            .unwrap_or(ItemRegistrationVerdict::NoItem);

        let item = match registration {
            ItemRegistrationVerdict::Register(item) => Some(item),
            ItemRegistrationVerdict::NoItem => None,
            ItemRegistrationVerdict::NoRegister => return Ok(()),
            ItemRegistrationVerdict::Deny(status) => return Err(NtStatusError::Status(status)),
        };

        let proc_info = ProcessInfo::try_create(item, process, pid, process_info);
        if proc_info.is_err() {
            return Ok(());
        }
        let proc_info = proc_info.unwrap();

        let proc_info = Arc::try_create(proc_info);
        if proc_info.is_err() {
            return Ok(());
        }

        let proc_info = proc_info.unwrap();

        let mut guard = self.process_map.write();
        let _ = guard.insert(pid, proc_info);

        Ok(())
    }

    fn on_destroy(&self, pid: HANDLE) {
        let mut guard = self.process_map.write();
        let old_item = guard.remove(&pid);
        drop(guard);

        if let Some(process) = old_item {
            process.item.as_ref().inspect(|item| item.destroy());
        }
    }
}
