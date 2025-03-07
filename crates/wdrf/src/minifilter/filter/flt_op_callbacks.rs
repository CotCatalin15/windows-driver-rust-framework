use windows_sys::{
    Wdk::Storage::FileSystem::Minifilters::{
        FLTFL_FILTER_UNLOAD_MANDATORY, FLTFL_POST_OPERATION_DRAINING, FLT_CALLBACK_DATA,
        FLT_PARAMETERS, FLT_POSTOP_CALLBACK_STATUS, FLT_POSTOP_FINISHED_PROCESSING,
        FLT_POSTOP_MORE_PROCESSING_REQUIRED, FLT_PREOP_CALLBACK_STATUS, FLT_PREOP_COMPLETE,
        FLT_PREOP_DISALLOW_FASTIO, FLT_PREOP_DISALLOW_FSFILTER_IO, FLT_PREOP_PENDING,
        FLT_PREOP_SUCCESS_NO_CALLBACK, FLT_PREOP_SUCCESS_WITH_CALLBACK, FLT_PREOP_SYNCHRONIZE,
        FLT_RELATED_OBJECTS,
    },
    Win32::Foundation::{NTSTATUS, STATUS_FLT_DO_NOT_DETACH, STATUS_SUCCESS},
};

use super::{
    framework::MinifilterFramework, params::FltParameters, registration::FltOperationType,
    FilterDataOperation, FilterUnload, FltCallbackData, FltPostOpCallback, FltPreOpCallback,
    FltRelatedObjects, PostOpContext, PostOpStatus, PreOpStatus,
};

pub unsafe extern "system" fn generic_pre_op_callback<'a, Pre>(
    data: *mut FLT_CALLBACK_DATA,
    fltobjects: *const FLT_RELATED_OBJECTS,
    completioncontext: *mut *mut core::ffi::c_void,
) -> FLT_PREOP_CALLBACK_STATUS
where
    Pre: FltPreOpCallback<'a>,
{
    let params: *const FLT_PARAMETERS = &(*(*data).Iopb).Parameters;
    let params: *mut FLT_PARAMETERS = params as _;

    let irp_mj = (*(*data).Iopb).MajorFunction;
    let irp_mj = core::mem::transmute::<u8, FltOperationType>(irp_mj);

    #[allow(invalid_reference_casting)]
    let params = &mut *params;
    let params = FltParameters::new(irp_mj, params);

    let status = Pre::call_pre(
        MinifilterFramework::context(),
        FltCallbackData::new(data),
        FltRelatedObjects::new(fltobjects),
        params,
    );

    let mut data = FltCallbackData::new(data);

    match status {
        PreOpStatus::Complete(status, size) => {
            data.set_status(status, size);
            FLT_PREOP_COMPLETE
        }
        PreOpStatus::DisalowFastIO => FLT_PREOP_DISALLOW_FASTIO,
        PreOpStatus::Pending => {
            if data.data_operation() != FilterDataOperation::Irp {
                panic!("Cannot pend a non-irp based operation");
            }
            FLT_PREOP_PENDING
        }
        PreOpStatus::SuccessNoCallback => FLT_PREOP_SUCCESS_NO_CALLBACK,
        PreOpStatus::SuccessWithCallback(any) => {
            if let Some(context) = any {
                let leak: *mut Pre::PostContext = PostOpContext::leak(context);
                *completioncontext = leak as _;
            } else {
                *completioncontext = core::ptr::null_mut();
            }

            FLT_PREOP_SUCCESS_WITH_CALLBACK
        }
        PreOpStatus::Sync => FLT_PREOP_SYNCHRONIZE,
        PreOpStatus::DisallowFsFilterIo => FLT_PREOP_DISALLOW_FSFILTER_IO,
    }
}

pub unsafe extern "system" fn generic_post_op_callback<'a, Post>(
    data: *mut FLT_CALLBACK_DATA,
    fltobjects: *const FLT_RELATED_OBJECTS,
    completioncontext: *const core::ffi::c_void,
    flags: u32,
) -> FLT_POSTOP_CALLBACK_STATUS
where
    Post: FltPostOpCallback<'a>,
{
    let params: *const FLT_PARAMETERS = &(*(*data).Iopb).Parameters;
    let params: *mut FLT_PARAMETERS = params as _;

    let irp_mj = (*(*data).Iopb).MajorFunction;
    let irp_mj = core::mem::transmute::<u8, FltOperationType>(irp_mj);

    #[allow(invalid_reference_casting)]
    let params = &mut *params;
    let params = FltParameters::new(irp_mj, params);

    let context = if completioncontext.is_null() {
        None
    } else {
        let context: *mut Post::PostContext = (completioncontext as *mut core::ffi::c_void) as _;
        let context = PostOpContext::from_raw_ptr(context);

        Some(context)
    };

    let draining = (flags & FLTFL_POST_OPERATION_DRAINING) == FLTFL_POST_OPERATION_DRAINING;

    let status = Post::call_post(
        MinifilterFramework::context(),
        FltCallbackData::new(data),
        FltRelatedObjects::new(fltobjects),
        params,
        context,
        draining,
    );

    match status {
        PostOpStatus::FinishProcessing => FLT_POSTOP_FINISHED_PROCESSING,
        PostOpStatus::PendProcessing => FLT_POSTOP_MORE_PROCESSING_REQUIRED,
    }
}

pub unsafe extern "system" fn flt_minifilter_unload_implementation<F>(flags: u32) -> NTSTATUS
where
    F: FilterUnload,
{
    let mandatory = (flags & FLTFL_FILTER_UNLOAD_MANDATORY) == FLTFL_FILTER_UNLOAD_MANDATORY;

    let context = MinifilterFramework::context();
    let result = F::call(context, mandatory);

    //let result = GLOBAL_MINIFILTER.get().filter_operations.unload(mandatory);
    match result {
        super::UnloadStatus::Unload => STATUS_SUCCESS,
        super::UnloadStatus::NoDetach => {
            if mandatory {
                panic!("Mandatory unload but NoDetach was returned from unload");
            } else {
                STATUS_FLT_DO_NOT_DETACH
            }
        }
    }
}
