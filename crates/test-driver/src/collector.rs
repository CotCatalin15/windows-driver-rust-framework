use wdk_sys::_PS_CREATE_NOTIFY_INFO;
use wdrf::process::collector::{ProcessDescriptor, ProcessHook};
use wdrf_std::kmalloc::TaggedObject;

pub struct TestProcessCollector {}

pub struct ProcessItem {
    pid: u64,
}

impl TaggedObject for ProcessItem {}

impl ProcessDescriptor for ProcessItem {
    fn pid(&self) -> u64 {
        self.pid
    }
}

impl ProcessItem {
    pub fn new(pid: u64) -> Self {
        Self { pid }
    }
}

impl TestProcessCollector {
    pub fn new() -> Self {
        Self {}
    }
}

impl ProcessHook for TestProcessCollector {
    type Item = ProcessItem;

    fn on_process_create(
        &mut self,
        _process: wdk_sys::PEPROCESS,
        process_id: wdk_sys::HANDLE,
        _create_info: &_PS_CREATE_NOTIFY_INFO,
    ) -> anyhow::Result<Self::Item> {
        Ok(ProcessItem::new(process_id as _))
    }

    fn on_process_destroy(&mut self, _item: &Self::Item) {}
}
