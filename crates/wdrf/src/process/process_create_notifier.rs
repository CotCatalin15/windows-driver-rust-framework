use core::any::Any;

use wdrf_std::object::{ArcKernelObj, NonNullKrnlResource};
use wdrf_std::NtResultEx;
use wdrf_std::{structs::PEPROCESS, NtResult};
use windows_sys::Wdk::System::SystemServices::{
    PsSetCreateProcessNotifyRoutineEx, PCREATE_PROCESS_NOTIFY_ROUTINE_EX, PS_CREATE_NOTIFY_INFO,
};
use windows_sys::Win32::Foundation::HANDLE;

use crate::context::{Context, ContextRegistry};

use super::{ProcessCollectorError, PsCreateNotifyInfo};

struct ProcessCreateNotifier {
    routine: PCREATE_PROCESS_NOTIFY_ROUTINE_EX,
    callback: Option<&'static dyn Any>,
}

unsafe impl Send for ProcessCreateNotifier {}
unsafe impl Sync for ProcessCreateNotifier {}

pub trait PsCreateNotifyCallback: Any + Send + Sync + 'static {
    //Return the create result for the process
    fn on_create(
        &self,
        process: ArcKernelObj<PEPROCESS>,
        pid: HANDLE,
        process_info: &PsCreateNotifyInfo,
    ) -> NtResult<()>;

    fn on_destroy(&self, pid: HANDLE);
}

static GLOBAL_PROCESS_COLLECTOR: Context<ProcessCreateNotifier> = Context::uninit();

///
/// # Safety
///
/// Only register it once
/// Multiple registers will cause a panic!
///
pub unsafe fn try_register_process_notifier<R: ContextRegistry>(
    registry: &'static R,
) -> anyhow::Result<(), ProcessCollectorError> {
    GLOBAL_PROCESS_COLLECTOR
        .init(registry, || ProcessCreateNotifier {
            routine: None,
            callback: None,
        })
        .map_err(|_| ProcessCollectorError::ContextRegisterError)?;

    Ok(())
}

/**

# Safety

Starts the collector
Collector must have been previosuly registered using `try_register_process_notifier`.

Callback must live as long as the registry or until stop_collector is called.

NOT THREAD SAFE

*/
pub unsafe fn start_collector<CB: PsCreateNotifyCallback>(callback: &'static CB) -> NtResult<()> {
    GLOBAL_PROCESS_COLLECTOR
        .get_mut()
        .callback
        .inspect(|_| panic!("Process collector already initialized"));

    GLOBAL_PROCESS_COLLECTOR.get_mut().callback = Some(callback as &'static dyn Any);
    GLOBAL_PROCESS_COLLECTOR.get_mut().routine = Some(process_notify_routine::<CB>);

    let status = PsSetCreateProcessNotifyRoutineEx(Some(process_notify_routine::<CB>), false as _);

    NtResult::from_status(status, || ())
}

/**

# Safety

Stops the collector
Collector must have been previosuly registered using `try_register_process_notifier`.

Safe to call if start_collector was not called previously

NOT THREAD SAFE

*/
pub unsafe fn stop_collector() -> NtResult<()> {
    let status = unsafe {
        PsSetCreateProcessNotifyRoutineEx(GLOBAL_PROCESS_COLLECTOR.get().routine, true as _)
    };

    NtResult::from_status(status, || {
        //Only unregister on success
        GLOBAL_PROCESS_COLLECTOR.get_mut().callback = None;
        ()
    })
}

impl Drop for ProcessCreateNotifier {
    fn drop(&mut self) {
        if self.callback.is_some() {
            unsafe {
                let _ = stop_collector();
            }
        }
    }
}

unsafe extern "system" fn process_notify_routine<CB: PsCreateNotifyCallback>(
    process_as_isize: isize,
    processid: HANDLE,
    createinfo: *mut PS_CREATE_NOTIFY_INFO,
) {
    let process: PEPROCESS = process_as_isize as *mut _;
    let process = NonNullKrnlResource::new(process);

    if process.is_none() {
        maple::error!("Received null eprocess in process_notify_routine");
        return;
    }
    let eprocess = process.unwrap();

    GLOBAL_PROCESS_COLLECTOR.get().callback.inspect(|cb| {
        let cb: &CB = cb.downcast_ref_unchecked();

        if createinfo.is_null() {
            //Process delete
            cb.on_destroy(processid);
        } else {
            //Process create
            let ke_process = ArcKernelObj::new(eprocess, true);

            let _ = cb
                .on_create(
                    ke_process,
                    processid,
                    &PsCreateNotifyInfo::new(&mut unsafe { *createinfo }),
                )
                .inspect_err(|e| match e {
                    wdrf_std::NtStatusError::Status(status) => {
                        (*createinfo).CreationStatus = *status
                    }
                });
        }
    });
}
