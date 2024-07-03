#![no_std]

use core::{mem::MaybeUninit, panic::PanicInfo, ptr::NonNull};

use collector::TestProcessCollector;
use wdk::{dbg_break, println};
#[cfg(not(test))]
use wdk_alloc::WDKAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WDKAllocator = WDKAllocator;

use wdk_sys::{
    ntddk::KeBugCheckEx, DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING, STATUS_SUCCESS,
    STATUS_UNSUCCESSFUL,
};
use wdrf::{
    context::FixedGlobalContextRegistry,
    framework::{
        flt_communication::{FltCommunicationDispatch, FltSingleClientCommunication},
        minifilter::{MinifilterFramework, MinifilterFrameworkBuilder},
    },
    process::collector::ProcessRegistry,
    Framework,
};
use wdrf_std::{
    boxed::{Box, BoxExt},
    io::Write,
    kmalloc::TaggedObject,
    slice::tracked_slice::TrackedSlice,
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
    communication: Option<FltSingleClientCommunication<CommunicationDispatch>>,
}

impl TaggedObject for TestDriverContext {}

static CONTEXT_REGISTRY: MaybeUninit<FixedGlobalContextRegistry<usize>> = MaybeUninit::uninit();

///# Safety
///
/// Driver entry point
///
///
#[export_name = "DriverEntry"] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    dbg_break();

    CONTEXT_REGISTRY = MaybeUninit::new(FixedGlobalContextRegistry::new());

    match driver_main(driver, registry_path) {
        Ok(_) => STATUS_SUCCESS,
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}

fn driver_main(driver: &mut DRIVER_OBJECT, registry_path: PCUNICODE_STRING) -> anyhow::Result<()> {
    Ok(())
}

struct CommunicationDispatch {}

impl TaggedObject for CommunicationDispatch {}

impl FltCommunicationDispatch for CommunicationDispatch {
    fn on_connect(
        &mut self,
        _client_port: &mut wdrf::framework::flt_communication::FltClient,
        _context: &[u8],
    ) -> anyhow::Result<()> {
        println!("Client connected");
        dbg_break();
        Ok(())
    }

    fn on_disconnect(&mut self, _client: wdrf::framework::flt_communication::FltClient) {
        println!("Client disconnected");
        dbg_break();
    }

    fn on_notify(&mut self, input: &[u8], output: &mut TrackedSlice) -> anyhow::Result<()> {
        println!("Client received message");
        dbg_break();

        output.write(b"Hello from the driver");

        Ok(())
    }
}

fn framework_main(builder: &mut MinifilterFrameworkBuilder) -> anyhow::Result<()> {
    let context = Box::try_create(TestDriverContext {
        collector: ProcessRegistry::new(TestProcessCollector::new()),
        communication: None,
    })?;

    println!("Building the minifilter");

    let minifilter = builder.unload(unload_callback).context(context).build()?;

    let port_name = widestring::u16str!("\\IGNISCOMMPORT\0");

    let context = MinifilterFramework::context::<TestDriverContext>().unwrap();

    let communication = FltSingleClientCommunication::try_create(
        minifilter.filter(),
        &mut NtUnicode::new_from_slice(port_name.as_slice()),
        CommunicationDispatch {},
    )?;
    context.communication = Some(communication);

    minifilter.start_filtering()?;

    Ok(())
}

pub fn unload_callback(framework: &mut MinifilterFramework) -> anyhow::Result<()> {
    dbg_break();
    let context = MinifilterFramework::context::<TestDriverContext>().unwrap();

    //Communication must be stopped before unloading the filter
    context.communication = None;

    Ok(())
}
