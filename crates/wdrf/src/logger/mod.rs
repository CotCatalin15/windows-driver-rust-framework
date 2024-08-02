use core::{
    fmt::Write,
    ops::DerefMut,
    panic,
    sync::atomic::{AtomicBool, Ordering},
};

use maple::consumer::EventConsumer;
use wdrf_std::{
    constants::PoolFlags,
    kmalloc::{GlobalKernelAllocator, MemoryTag, TaggedObject},
    sync::{
        arc::{Arc, ArcExt},
        mutex::SpinMutex,
    },
    sys::{
        event::{EventType, KeEvent},
        WaitResponse, WaitableObject,
    },
    thread::{spawn, JoinHandle},
    vec::{Vec, VecCreate, VecExt},
};
use windows_sys::Wdk::System::SystemServices::DbgPrint;

const VEC_U8_TAG: MemoryTag = MemoryTag::new_from_bytes(b"logv");

struct LoggerInner {
    log_event: KeEvent,
    pending_events: SpinMutex<Vec<Vec<u8>>>,
    stop: AtomicBool,
}

unsafe impl Send for LoggerInner {}
unsafe impl Sync for LoggerInner {}

impl TaggedObject for LoggerInner {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"logd")
    }
}

pub struct DbgPrintLogger {
    inner: Arc<LoggerInner>,
    log_thread: JoinHandle<()>,
}

pub struct DbgWrittable {
    offset: usize,
    buffer: Vec<u8>,
}

impl DbgWrittable {
    pub fn try_create() -> anyhow::Result<Self> {
        let mut buffer = Vec::create();
        buffer.try_resize(512, 0 as u8)?;

        Ok(Self { offset: 0, buffer })
    }
}

impl core::fmt::Write for DbgWrittable {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.offset + s.len() + 1 >= self.buffer.len() {
            //for 0 terminated string
            self.buffer
                .try_resize(s.len() + 1, 0)
                .map_err(|_| core::fmt::Error)?;
        }

        self.buffer[self.offset..][..s.len()].copy_from_slice(s.as_bytes());
        self.offset += s.len();

        Ok(())
    }
}

impl DbgPrintLogger {
    pub fn new() -> anyhow::Result<Self> {
        let buffer = Vec::new_in(GlobalKernelAllocator::new(
            VEC_U8_TAG,
            PoolFlags::POOL_FLAG_NON_PAGED,
        ));

        let inner = LoggerInner {
            log_event: unsafe { KeEvent::new() },
            pending_events: SpinMutex::new(buffer),
            stop: AtomicBool::new(false),
        };

        let inner = Arc::try_create(inner)?;
        inner.log_event.init(EventType::Notification, false);

        let inner_clone = inner.clone();
        let th = spawn(move || Self::worker_routine(inner_clone))
            .map_err(|_| anyhow::Error::msg("Failed to create logger thread"))?;

        Ok(Self {
            inner: inner,
            log_thread: th,
        })
    }

    pub fn log_event(&self, writtable: DbgWrittable) {
        {
            let mut guard = self.inner.pending_events.lock();

            if let Err(_) = guard.try_push(writtable.buffer) {
                return;
            }
        }

        self.inner.log_event.signal();
    }

    fn worker_routine(inner: Arc<LoggerInner>) {
        let logger = inner.as_ref();

        let mut event_buffer = Vec::new_in(GlobalKernelAllocator::new(
            VEC_U8_TAG,
            PoolFlags::POOL_FLAG_NON_PAGED,
        ));

        loop {
            let status = logger.log_event.wait();
            if status != WaitResponse::Success {
                panic!("AAA");
            }

            if logger.stop.load(Ordering::Relaxed) {
                break;
            }

            {
                let mut guard = logger.pending_events.lock();
                core::mem::swap(guard.deref_mut(), &mut event_buffer);
                inner.log_event.clear();
            }

            for event in &event_buffer {
                unsafe {
                    DbgPrint(event.as_slice().as_ptr());
                }
            }

            event_buffer.clear();
        }
    }
}

impl EventConsumer for DbgPrintLogger {
    fn enabled(&self) -> bool {
        true
    }

    fn disable(&self) {}

    fn filter(&self, _meta: &maple::fields::Metadata) -> maple::consumer::FilterResult {
        maple::consumer::FilterResult::Allow
    }

    fn event(&self, event: &maple::fields::Event) {
        if let Ok(mut writable) = DbgWrittable::try_create() {
            let meta = event.meta();
            let args = event.args();

            let _ = writable.write_fmt(format_args!(
                "[{}:{}:{}][{}] {:#?}\n\0",
                meta.module,
                meta.file,
                meta.line,
                meta.name.unwrap_or(""),
                args
            ));
        }
    }
}
