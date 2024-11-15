use core::any::Any;

use wdrf_std::boxed::Box;
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
    framework::GLOBAL_MINIFILTER,
    params::{FltCreateRequest, FltQueryFileRequest, FltReadFileRequest},
    FilterDataOperation, FilterOperationVisitor, FltCallbackData, FltRelatedObjects, PostOpContext,
    PostOpStatus, PostOperationVisitor, PreOpStatus, PreOperationVisitor,
};

#[inline]
unsafe fn generic_pre_op_callback<'a, V, F>(
    data: *mut FLT_CALLBACK_DATA,
    fltobjects: *const FLT_RELATED_OBJECTS,
    completioncontext: *mut *mut core::ffi::c_void,
    f: F,
) -> FLT_PREOP_CALLBACK_STATUS
where
    V: PreOperationVisitor + 'a,
    F: FnOnce(
        &'a V,
        FltCallbackData<'a>,
        FltRelatedObjects<'a>,
        &'a mut FLT_PARAMETERS,
    ) -> PreOpStatus,
{
    let params: *const FLT_PARAMETERS = &(*(*data).Iopb).Parameters;
    let params: *mut FLT_PARAMETERS = params as _;
    #[allow(invalid_reference_casting)]
    let params = &mut *params;

    let pre_op_visitor: *const dyn PreOperationVisitor =
        GLOBAL_MINIFILTER.get().pre_operations.as_ref();
    let pre_op_visitor = &*(pre_op_visitor as *const V);

    let status = f(
        pre_op_visitor,
        FltCallbackData::new(data),
        FltRelatedObjects::new(fltobjects),
        params,
    );

    let mut data = FltCallbackData::new(data);

    match status {
        PreOpStatus::Complete(status) => {
            data.set_status(status, 0);
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
                let leak: *mut dyn Any = PostOpContext::leak(context);
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

#[inline]
unsafe fn generic_post_op_callback<'a, V, F>(
    data: *mut FLT_CALLBACK_DATA,
    fltobjects: *const FLT_RELATED_OBJECTS,
    completioncontext: *mut core::ffi::c_void,
    flags: u32,
    f: F,
) -> FLT_PREOP_CALLBACK_STATUS
where
    V: PostOperationVisitor + 'a,
    F: FnOnce(
        &'a V,
        FltCallbackData<'a>,
        FltRelatedObjects<'a>,
        Option<PostOpContext<dyn Any>>,
        &'a mut FLT_PARAMETERS,
        bool,
    ) -> PostOpStatus,
{
    let params: *const FLT_PARAMETERS = &(*(*data).Iopb).Parameters;
    let params: *mut FLT_PARAMETERS = params as _;
    #[allow(invalid_reference_casting)]
    let params = &mut *params;

    let post_op_visitor: *const dyn PostOperationVisitor =
        GLOBAL_MINIFILTER.get().post_operations.as_ref();
    let post_op_visitor = &*(post_op_visitor as *const V);

    let context = if completioncontext.is_null() {
        None
    } else {
        let context: *mut dyn Any = completioncontext as _;
        let context = PostOpContext::from_raw_ptr(context);

        Some(context)
    };

    let draining = (flags & FLTFL_POST_OPERATION_DRAINING) == FLTFL_POST_OPERATION_DRAINING;

    let status = f(
        post_op_visitor,
        FltCallbackData::new(data),
        FltRelatedObjects::new(fltobjects),
        context,
        params,
        draining,
    );

    match status {
        PostOpStatus::FinishProcessing => FLT_POSTOP_FINISHED_PROCESSING,
        PostOpStatus::PendProcessing => FLT_POSTOP_MORE_PROCESSING_REQUIRED,
    }
}

macro_rules! generate_preop_cb {
    ($func_name:ident, $visitor_fn:ident, $request_type:ty) => {
        pub unsafe extern "system" fn $func_name<V>(
            data: *mut FLT_CALLBACK_DATA,
            fltobjects: *const FLT_RELATED_OBJECTS,
            completioncontext: *mut *mut core::ffi::c_void,
        ) -> FLT_PREOP_CALLBACK_STATUS
        where
            V: PreOperationVisitor,
        {
            generic_pre_op_callback(
                data,
                fltobjects,
                completioncontext,
                |visitor: &V, data, fltobjects, params| {
                    visitor.$visitor_fn(data, fltobjects, <$request_type>::new(params))
                },
            )
        }
    };
}

macro_rules! generate_postop_cb {
    ($func_name:ident, $visitor_fn:ident, $request_type:ty) => {
        pub unsafe extern "system" fn $func_name<V>(
            data: *mut FLT_CALLBACK_DATA,
            fltobjects: *const FLT_RELATED_OBJECTS,
            completioncontext: *const core::ffi::c_void,
            flags: u32,
        ) -> FLT_POSTOP_CALLBACK_STATUS
        where
            V: PostOperationVisitor,
        {
            generic_post_op_callback(
                data,
                fltobjects,
                completioncontext as _,
                flags,
                |visitor: &V, data, related_obj, context, params, draining| {
                    visitor.$visitor_fn(
                        data,
                        related_obj,
                        <$request_type>::new(params),
                        context,
                        draining,
                    )
                },
            )
        }
    };
}

// ################ BEGIN PREOP CALLBACKS
generate_preop_cb!(flt_create_pre_op_implementation, create, FltCreateRequest);
generate_preop_cb!(
    flt_query_information_pre_op_implementation,
    query_file_information,
    FltQueryFileRequest
);
generate_preop_cb!(flt_read_pre_op_implementation, read, FltReadFileRequest);

// ################# END PREOP CALLBACKS
//
//
//
// Use commencts since the autoformater deleted these mepty lines :(
//
//
//
// ################ BEGIN POST CALLBACKS
generate_postop_cb!(flt_create_post_op_implementation, create, FltCreateRequest);
generate_postop_cb!(
    flt_query_information_post_op_implementation,
    create,
    FltQueryFileRequest
);
/// ################# END PREOP CALLBACKS
//
//
//
//
//
//
//
//

pub unsafe extern "system" fn flt_minifilter_unload_implementation<V>(flags: u32) -> NTSTATUS
where
    V: FilterOperationVisitor,
{
    let mandatory = (flags & FLTFL_FILTER_UNLOAD_MANDATORY) == FLTFL_FILTER_UNLOAD_MANDATORY;
    let result = GLOBAL_MINIFILTER.get().filter_operations.unload(mandatory);

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
