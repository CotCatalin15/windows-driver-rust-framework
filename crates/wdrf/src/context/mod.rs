use core::{
    cell::SyncUnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

use wdrf_std::sync::mutex::SpinMutex;

///
/// Usefull for storing static global data
///

struct FixedRegistryInternal<const SIZE: usize> {
    array: [MaybeUninit<&'static dyn ContextDrop>; SIZE],
    size: usize,
}

pub struct FixedGlobalContextRegistry<const SIZE: usize> {
    internal: SyncUnsafeCell<MaybeUninit<SpinMutex<FixedRegistryInternal<SIZE>>>>,
}

unsafe impl<const SIZE: usize> Send for FixedGlobalContextRegistry<SIZE> {}
unsafe impl<const SIZE: usize> Sync for FixedGlobalContextRegistry<SIZE> {}

pub trait ContextRegistry {
    fn register<T>(&self, context: &'static Context<T>) -> anyhow::Result<()>;
    fn drop_self(&self);
}

impl<const SIZE: usize> FixedGlobalContextRegistry<SIZE> {
    pub const fn new() -> Self {
        Self {
            internal: SyncUnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn init(&self) {
        unsafe {
            let s = SpinMutex::new(FixedRegistryInternal::<SIZE> {
                array: MaybeUninit::uninit().assume_init(),
                size: 0,
            });
            let internal = &mut *self.internal.get();
            *internal = MaybeUninit::new(s);
        }
    }
}

impl<const SIZE: usize> Default for FixedGlobalContextRegistry<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> ContextRegistry for FixedGlobalContextRegistry<SIZE> {
    fn register<T: Sized>(&self, context: &'static Context<T>) -> anyhow::Result<()> {
        let guard = unsafe { &mut *self.internal.get() };
        let mut guard = unsafe { guard.assume_init_mut() }.lock();

        if guard.size + 1 >= SIZE {
            Err(anyhow::Error::msg("Fixed context registry is full"))
        } else {
            unsafe {
                let val: &'static dyn ContextDrop = context;
                let pos = guard.size;

                *guard.array.get_unchecked_mut(pos) = MaybeUninit::new(val);
            }
            guard.size += 1;

            Ok(())
        }
    }

    fn drop_self(&self) {
        let guard = unsafe { &mut *self.internal.get() };
        let guard = unsafe { guard.assume_init_mut() }.lock();

        let s = guard.size.clone();
        for elem in &mut guard.array[0..s].iter().rev() {
            unsafe {
                elem.assume_init().context_drop();
            }
        }
    }
}

trait ContextDrop {
    fn context_drop(&self);
}

pub struct Context<T: Sized> {
    is_init: AtomicBool,
    data: SyncUnsafeCell<MaybeUninit<T>>,
}

impl<T> Context<T> {
    pub const fn uninit() -> Self {
        Self {
            is_init: AtomicBool::new(false),
            data: SyncUnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub const fn new(data: T) -> Self {
        Self {
            is_init: AtomicBool::new(true),
            data: SyncUnsafeCell::new(MaybeUninit::new(data)),
        }
    }

    pub fn init<R: ContextRegistry, F>(
        &'static self,
        registry: &'static R,
        init_function: F,
    ) -> anyhow::Result<()>
    where
        F: FnOnce() -> T,
    {
        let result = self
            .is_init
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);

        match result {
            Ok(false) => {
                unsafe {
                    *self.data.get() = MaybeUninit::new(init_function());
                }
                registry.register(self)
            }
            Err(true) => Ok(()),
            _ => panic!("Should not have gotten here"),
        }
    }

    pub fn get(&self) -> &'static T {
        unsafe { (*self.data.get()).assume_init_ref() }
    }

    pub fn get_mut(&self) -> &'static mut T {
        unsafe { (*self.data.get()).assume_init_mut() }
    }
}

impl<T> ContextDrop for Context<T> {
    fn context_drop(&self) {
        unsafe {
            if self.is_init.load(Ordering::SeqCst) {
                (*self.data.get()).assume_init_drop();
            }
        }
    }
}
