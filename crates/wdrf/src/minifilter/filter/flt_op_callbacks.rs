use core::any::Any;

use windows_sys::{
    Wdk::Storage::FileSystem::Minifilters::{
        FltGetFileNameInformation, FltReleaseFileNameInformation, FLTFL_FILTER_UNLOAD_MANDATORY,
        FLTFL_POST_OPERATION_DRAINING, FLT_CALLBACK_DATA, FLT_FILE_NAME_OPENED, FLT_PARAMETERS,
        FLT_POSTOP_CALLBACK_STATUS, FLT_POSTOP_FINISHED_PROCESSING,
        FLT_POSTOP_MORE_PROCESSING_REQUIRED, FLT_PREOP_CALLBACK_STATUS, FLT_PREOP_COMPLETE,
        FLT_PREOP_DISALLOW_FASTIO, FLT_PREOP_DISALLOW_FSFILTER_IO, FLT_PREOP_PENDING,
        FLT_PREOP_SUCCESS_NO_CALLBACK, FLT_PREOP_SUCCESS_WITH_CALLBACK, FLT_PREOP_SYNCHRONIZE,
        FLT_RELATED_OBJECTS,
    },
    Win32::{
        Foundation::{
            NTSTATUS, STATUS_FLT_DO_NOT_ATTACH, STATUS_FLT_DO_NOT_DETACH, STATUS_SUCCESS,
        },
        Storage::{
            FileSystem::{FILE_DEVICE_CD_ROM, FILE_DEVICE_DISK},
            InstallableFileSystems::FLT_FILESYSTEM_TYPE,
        },
    },
};

use super::{
    framework::GLOBAL_MINIFILTER, params::FltParameters, registration::FltOperationType,
    FilterDataOperation, FilterOperationVisitor, FltCallbackData, FltDeviceType, FltPostOpCallback,
    FltPreOpCallback, FltRelatedObjects, FltVolumeType, InstanceSetupStatus, PostOpContext,
    PostOpStatus, PreOpStatus,
};

pub unsafe extern "system" fn generic_pre_op_callback<'a, V>(
    data: *mut FLT_CALLBACK_DATA,
    fltobjects: *const FLT_RELATED_OBJECTS,
    completioncontext: *mut *mut core::ffi::c_void,
) -> FLT_PREOP_CALLBACK_STATUS
where
    V: FltPreOpCallback + 'a,
{
    let params: *const FLT_PARAMETERS = &(*(*data).Iopb).Parameters;
    let params: *mut FLT_PARAMETERS = params as _;
    #[allow(invalid_reference_casting)]
    let params = &mut *params;
    let params = FltParameters::new(
        FltOperationType::from_irp_mj((*(*data).Iopb).MajorFunction),
        params,
    );

    let pre_op_visitor: *const dyn FltPreOpCallback =
        GLOBAL_MINIFILTER.get().pre_operations.as_ref();
    let pre_op_visitor = &*(pre_op_visitor as *const V);

    let status = pre_op_visitor.callback(
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

pub unsafe extern "system" fn generic_post_op_callback<'a, V>(
    data: *mut FLT_CALLBACK_DATA,
    fltobjects: *const FLT_RELATED_OBJECTS,
    completioncontext: *const core::ffi::c_void,
    flags: u32,
) -> FLT_POSTOP_CALLBACK_STATUS
where
    V: FltPostOpCallback + 'a,
{
    let params: *const FLT_PARAMETERS = &(*(*data).Iopb).Parameters;
    let params: *mut FLT_PARAMETERS = params as _;
    #[allow(invalid_reference_casting)]
    let params = &mut *params;
    let params = FltParameters::new(
        FltOperationType::from_irp_mj((*(*data).Iopb).MajorFunction),
        params,
    );

    let post_op_visitor: *const dyn FltPostOpCallback =
        GLOBAL_MINIFILTER.get().post_operations.as_ref();
    let post_op_visitor = &*(post_op_visitor as *const V);

    let context = if completioncontext.is_null() {
        None
    } else {
        let context: *mut dyn Any = (completioncontext as *mut core::ffi::c_void) as _;
        let context = PostOpContext::from_raw_ptr(context);

        Some(context)
    };

    let draining = (flags & FLTFL_POST_OPERATION_DRAINING) == FLTFL_POST_OPERATION_DRAINING;

    let status = post_op_visitor.callback(
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

pub unsafe extern "system" fn flt_minifilter_instance_setup(
    fltobjects: *const FLT_RELATED_OBJECTS,
    flags: u32,
    volumedevicetype: u32,
    volumefilesystemtype: FLT_FILESYSTEM_TYPE,
) -> NTSTATUS {
    let device_type = match volumedevicetype {
        FILE_DEVICE_CD_ROM => FltDeviceType::CdRom,
        FILE_DEVICE_DISK => FltDeviceType::Disk,
        0x00000014 => FltDeviceType::Network,
        _ => panic!("Unknown device type received: {volumedevicetype}"),
    };

    let fs_type = FltVolumeType::try_from(volumefilesystemtype).expect("Uknown filesystem type");

    let result = GLOBAL_MINIFILTER.get().filter_operations.instance_setup(
        FltRelatedObjects::new(fltobjects),
        flags,
        device_type,
        fs_type,
    );

    match result {
        InstanceSetupStatus::Success => STATUS_SUCCESS,
        InstanceSetupStatus::DoNotAttach => STATUS_FLT_DO_NOT_ATTACH,
    }
}
