use core::ops::{Deref, DerefMut};

pub trait Unlockable {
    type Item;

    fn unlock(&self);
}

pub struct MutexGuard<'a, U: Unlockable> {
    unlockable: U,
    data: &'a mut U::Item,
}

impl<'a, U: Unlockable> !Send for MutexGuard<'a, U> {}
unsafe impl<'a, U> Sync for MutexGuard<'a, U>
where
    U: Unlockable,
    U::Item: Sync,
{
}

impl<'a, U> MutexGuard<'a, U>
where
    U: Unlockable,
{
    pub(super) fn new(unlockable: U, data: &'a mut U::Item) -> Self {
        Self { unlockable, data }
    }
}

impl<'a, U> Deref for MutexGuard<'_, U>
where
    U: Unlockable,
{
    type Target = U::Item;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, U> DerefMut for MutexGuard<'_, U>
where
    U: Unlockable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, U> Drop for MutexGuard<'_, U>
where
    U: Unlockable,
{
    fn drop(&mut self) {
        self.unlockable.unlock();
    }
}

pub struct ReadMutexGuard<'a, U: Unlockable> {
    unlockable: U,
    data: &'a U::Item,
}

impl<'a, U: Unlockable> !Send for ReadMutexGuard<'a, U> {}
unsafe impl<'a, U> Sync for ReadMutexGuard<'a, U>
where
    U: Unlockable,
    U::Item: Sync,
{
}

impl<'a, U> ReadMutexGuard<'a, U>
where
    U: Unlockable,
{
    pub(super) fn new(unlockable: U, data: &'a U::Item) -> Self {
        Self { unlockable, data }
    }
}

impl<'a, U> Deref for ReadMutexGuard<'_, U>
where
    U: Unlockable,
{
    type Target = U::Item;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, U> Drop for ReadMutexGuard<'_, U>
where
    U: Unlockable,
{
    fn drop(&mut self) {
        self.unlockable.unlock();
    }
}
