pub unsafe trait DispatchSafe {}

pub trait WriteLock {
    fn lock(&self);
    fn unlock(&self);
}

pub trait ReadLock {
    fn lock_shared(&self);
    fn unlock_shared(&self);
}
