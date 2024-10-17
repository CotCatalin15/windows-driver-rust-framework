use wdrf_std::{boxed::Box, thread::spawn};
use windows_sys::Win32::Foundation::NTSTATUS;

use super::{
    params::{FltCreateRequest, FltQueryFileRequest},
    FltCallbackData, FltRelatedObjects,
};

/*
FLT_PREOP_COMPLETE	The minifilter is completing the I/O operation. See Remarks for details.
FLT_PREOP_DISALLOW_FASTIO	The operation is a fast I/O operation, and the minifilter is not allowing the fast I/O path to be used for this operation. See Remarks for details.
FLT_PREOP_PENDING	The minifilter has pended the I/O operation, and the operation is still pending. See Remarks for details.
FLT_PREOP_SUCCESS_NO_CALLBACK	The minifilter is returning the I/O operation to FltMgr for further processing. In this case, FltMgr won't call the minifilter driver's post-operation callback, if one exists, during I/O completion.
FLT_PREOP_SUCCESS_WITH_CALLBACK	The minifilter is returning the I/O operation to FltMgr for further processing. In this case, FltMgr calls the minifilter's post-operation callback during I/O completion.
FLT_PREOP_SYNCHRONIZE	The minifilter is returning the I/O operation to FltMgr for further processing, but it is not completing the operation. See Remarks for details.
FLT_PREOP_DISALLOW_FSFILTER_IO	The minifilter is disallowing a fast QueryOpen operation and forcing the operation down the slow path. Doing so causes the I/O manager to service the request by performing an open/query/close of the file. Minifilter drivers should only return this status for QueryOpen.
 */

pub enum PreOpStatus {
    Complete(NTSTATUS),
    DisalowFastIO,
    Pending,
    SuccessNoCallback,
    SuccessWithCallback,
    Sync,
    DisallowFsFilterIo,
}
pub enum PostOpStatus {}

pub trait PreOperationVisitor {
    fn create<'a>(
        &self,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        create: FltCreateRequest<'a>,
    ) -> PreOpStatus;

    fn query_file_information<'a>(
        &self,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        query: FltQueryFileRequest<'a>,
    ) -> PreOpStatus;
}

pub trait PostOperationVisitor {}

fn test() {
    let b: Option<Box<dyn PreOperationVisitor>> = None;

    if let Some(ref b) = b {
        b.create(
            FltCallbackData::new(core::ptr::null_mut()),
            FltRelatedObjects::new(core::ptr::null_mut()),
        );
    }
}
