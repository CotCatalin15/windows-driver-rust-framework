use wdrf::{
    context::ContextRegistry,
    process::collector::{
        IProcessItemFactory, ItemRegistrationVerdict, ProcessCollector, ProcessInfo,
    },
};
use wdrf_std::{
    object::ArcKernelObj,
    structs::{PEPROCESS, PFILE_OBJECT},
    sync::arc::Arc,
};
use windows_sys::Win32::Foundation::HANDLE;

struct ProcessCreateFactory {}

impl IProcessItemFactory for ProcessCreateFactory {
    type Item = (ArcKernelObj<PEPROCESS>, ArcKernelObj<PFILE_OBJECT>);

    fn create(
        &self,
        process: ArcKernelObj<wdrf_std::structs::PEPROCESS>,
        _pid: windows_sys::Win32::Foundation::HANDLE,
        process_info: &wdrf::process::PsCreateNotifyInfo,
    ) -> anyhow::Result<
        wdrf::process::collector::ItemRegistrationVerdict<Self::Item>,
        wdrf::process::ProcessCollectorError,
    > {
        Ok(ItemRegistrationVerdict::Register((
            process,
            ArcKernelObj::new(*process_info.file_object.as_ref().unwrap(), true),
        )))
    }
}

pub struct TestCollector {
    collector: ProcessCollector<ProcessCreateFactory>,
}

impl TestCollector {
    pub fn new<R: ContextRegistry>(registry: &'static R) -> Self {
        Self {
            collector: ProcessCollector::try_create_with_registry(
                registry,
                ProcessCreateFactory {},
            )
            .unwrap(),
        }
    }

    pub fn find_by_pid(
        &self,
        pid: HANDLE,
    ) -> Option<Arc<ProcessInfo<(ArcKernelObj<PEPROCESS>, ArcKernelObj<PFILE_OBJECT>)>>> {
        self.collector.find_by_pid(pid)
    }
}
