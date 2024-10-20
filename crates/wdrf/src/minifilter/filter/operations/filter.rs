pub enum UnloadStatus {
    Unload,
    NoDetach,
}

#[allow(unused_variables)]
pub trait FilterOperationVisitor: Send + Sync + 'static {
    fn unload(&self, mandatory: bool) -> UnloadStatus {
        UnloadStatus::Unload
    }
}
