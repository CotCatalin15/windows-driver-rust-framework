use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use crate::{
    kmalloc::TaggedObject,
    sys,
    traits::{DispatchSafe, WriteLock},
};

pub type GuardedMutex<T> = Mutex<T, sys::mutex::GuardedMutex>;
pub type SpinMutex<T> = Mutex<T, sys::mutex::SpinLock>;

/// Its implemented using a guard mutex
/// It can be used at IRQL <= APC
///
pub struct Mutex<T: ?Sized, L: WriteLock> {
    inner: L,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send, L: WriteLock> Send for Mutex<T, L> {}
unsafe impl<T: ?Sized + Send, L: WriteLock> Sync for Mutex<T, L> {}
unsafe impl<T: ?Sized + Send, L: WriteLock + DispatchSafe> DispatchSafe for Mutex<T, L> {}

impl<T: ?Sized + Send + TaggedObject, L: WriteLock> TaggedObject for Mutex<T, L> {
    fn tag() -> crate::kmalloc::MemoryTag {
        T::tag()
    }

    fn flags() -> crate::constants::PoolFlags {
        T::flags()
    }
}

pub struct MutexGuard<'a, T: ?Sized, L: WriteLock> {
    lock: &'a Mutex<T, L>,
}

impl<'a, T: ?Sized, L> !Send for MutexGuard<'a, T, L> {}
unsafe impl<'a, T: ?Sized, L: WriteLock> Sync for MutexGuard<'a, T, L> {}

impl<T, L> Mutex<T, L>
where
    L: WriteLock,
{
    pub const fn new_in(data: T, lock: L) -> Self {
        Self {
            inner: lock,
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T, L> {
        self.inner.lock();
        MutexGuard::new(self)
    }
}

impl<T, L> Mutex<T, L>
where
    L: WriteLock + Default,
{
    pub fn new(data: T) -> Self {
        Self {
            inner: L::default(),
            data: UnsafeCell::new(data),
        }
    }
}

impl<'a, T, L> MutexGuard<'a, T, L>
where
    L: WriteLock,
{
    fn new(lock: &'a Mutex<T, L>) -> Self {
        Self { lock }
    }
}

impl<T: ?Sized, L> Deref for MutexGuard<'_, T, L>
where
    L: WriteLock,
{
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized, L> DerefMut for MutexGuard<'_, T, L>
where
    L: WriteLock,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized, L> Drop for MutexGuard<'_, T, L>
where
    L: WriteLock,
{
    fn drop(&mut self) {
        self.lock.inner.unlock();
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::traits::WriteLock;

    use super::Mutex;

    struct TestLock {
        is_locked: *mut bool,
    }

    impl TestLock {
        pub fn new(is_locked: *mut bool) -> Self {
            Self {
                is_locked: is_locked,
            }
        }
    }

    impl WriteLock for TestLock {
        fn lock(&self) {
            unsafe {
                if *self.is_locked {
                    panic!("This test mutex must not be used from multiple threads")
                }
                *self.is_locked = true;
            }
        }

        fn unlock(&self) {
            unsafe {
                if !*self.is_locked {
                    panic!("Unlocked called without locking it first");
                }

                *self.is_locked = false;
            }
        }
    }

    type TestMutex<T> = Mutex<T, TestLock>;

    ///A simple test to check if locking locks the mutex and dropping the guard unlocks the mutex
    ///
    #[test]
    pub fn test_lock_unlock() {
        let mut b = std::boxed::Box::new(false);
        let lock = TestLock::new(b.as_mut());
        let m = TestMutex::new_in(10, lock);

        assert_eq!(*b, false);
        let mut guard = m.lock();

        //Its locked
        assert!(*b);
        assert_eq!(*guard, 10);

        *guard = 20;
        assert_eq!(*guard, 20);

        drop(guard);
        assert_eq!(*b, false);

        guard = m.lock();
        assert!(*b);
        assert_eq!(*guard, 20);

        drop(guard);
        assert_eq!(*b, false);
    }
}
