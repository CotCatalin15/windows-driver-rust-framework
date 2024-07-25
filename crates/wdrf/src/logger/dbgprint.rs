use core::{fmt::Write as FmtWrite, num::NonZeroU32};
use maple::consumer::EventConsumer;
use wdk::{dbg_break, println};
use wdk_sys::ntddk::DbgPrint;
use wdrf_std::{
    fmt::Wrapper,
    kmalloc::{lookaside::LookasideAllocator, MemoryTag},
    vec::Vec,
    vec::VecExt,
    NtResult,
};

const MAX_PRINT_SIZE: u32 = 512;

pub struct DbgPrintLogger {
    allocator: LookasideAllocator,
}

impl DbgPrintLogger {
    pub fn new() -> NtResult<Self> {
        let allocator = LookasideAllocator::new(
            NonZeroU32::new(MAX_PRINT_SIZE).unwrap(),
            MemoryTag::new_from_bytes(b"dbgp"),
        )?;

        Ok(Self { allocator })
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
        let allocator = self.allocator.clone();
        let buffer = Vec::try_with_capacity_in(MAX_PRINT_SIZE as _, allocator);
        if buffer.is_err() {
            dbg_break();
            return;
        }
        let mut buffer: Vec<u8, _> = buffer.unwrap();
        let r = buffer.try_resize(MAX_PRINT_SIZE as _, 0);
        if r.is_err() {
            dbg_break();
            return;
        }

        let mut wrapper = Wrapper::new(buffer.as_mut_slice());
        let meta = event.meta();
        let r = wrapper.write_fmt(format_args!(
            "[{}:{}:{}][{}] {:#?}\n\0",
            meta.module,
            meta.file,
            meta.line,
            meta.name.unwrap_or(""),
            event.args()
        ));
        if r.is_err() {
            //Oversize
            println!(
                "[{}:{}:{}][{}] {:#?}",
                meta.module,
                meta.file,
                meta.line,
                meta.name.unwrap_or(""),
                event.args()
            );
        } else {
            unsafe {
                DbgPrint(buffer.as_ptr() as _);
            }
        }
    }
}
