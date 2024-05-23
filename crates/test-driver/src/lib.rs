#![no_std]

use core::{mem::MaybeUninit, panic::PanicInfo, ptr::NonNull};

use collector::TestProcessCollector;
use wdk::{dbg_break, println};
#[cfg(not(test))]
use wdk_alloc::WDKAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WDKAllocator = WDKAllocator;

use wdk_sys::{ntddk::KeBugCheckEx, DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING};
use wdrf::{
    framework::minifilter::{MinifilterFramework, MinifilterFrameworkBuilder},
    process::collector::ProcessRegistry,
};
use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
    string::ntunicode::NtUnicode,
};

pub mod collector;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        println!("[PANIC] called: {:#?}", info);
        KeBugCheckEx(0x1234, 0, 0, 0, 0);
    }
}

struct TestDriverContext {
    collector: ProcessRegistry<TestProcessCollector>,
}

impl TaggedObject for TestDriverContext {}

///# Safety
///
/// Its safe its just for testing
///
#[export_name = "DriverEntry"] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    dbg_break();

    MinifilterFrameworkBuilder::start_builder(
        NonNull::new(driver).unwrap(),
        NtUnicode::new(&*registry_path),
        framework_main,
    )
}

fn framework_main(builder: &mut MinifilterFrameworkBuilder) -> anyhow::Result<()> {
    dbg_break();

    let context = Box::try_create(TestDriverContext {
        collector: ProcessRegistry::new(TestProcessCollector::new()),
    })?;

    println!("Building the minifilter");

    let _minifilter = builder.unload(unload_callback).context(context).build()?;

    Ok(())
}

pub fn unload_callback(_framework: &mut MinifilterFramework) -> anyhow::Result<()> {
    Ok(())
}
