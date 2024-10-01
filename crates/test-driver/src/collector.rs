use nt_string::unicode_string::NtUnicodeStr;
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
        let path = process_info
            .image_file_name
            .unwrap_or(nt_string::nt_unicode_str!("Unknown process"));

        let cmd = process_info
            .command_line
            .unwrap_or(nt_string::nt_unicode_str!("Unknown command line"));

        maple::info!("Creating process info for {} Cmd: {}", path, cmd);

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
        let collector =
            ProcessCollector::try_create_with_registry(registry, ProcessCreateFactory {}).unwrap();

        collector.start().unwrap();

        Self { collector }
    }

    pub fn find_by_pid(
        &self,
        pid: HANDLE,
    ) -> Option<Arc<ProcessInfo<(ArcKernelObj<PEPROCESS>, ArcKernelObj<PFILE_OBJECT>)>>> {
        self.collector.find_by_pid(pid)
    }
}
