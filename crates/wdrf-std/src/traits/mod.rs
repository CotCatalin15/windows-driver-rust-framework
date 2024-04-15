///
/// # Safety
/// If a structure can be safety called at dispatch it must
/// implement this trait
///
pub unsafe trait DispatchSafe {}

pub trait WriteLock {
    fn lock(&self);
    fn unlock(&self);
}

pub trait ReadLock {
    fn lock_shared(&self);
    fn unlock_shared(&self);
}
