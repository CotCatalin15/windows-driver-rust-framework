use nt_string::unicode_string::NtUnicodeStr;
use wdrf::minifilter::filter::{PreOpStatus, PreOperationVisitor};
use wdrf_std::kmalloc::TaggedObject;

pub struct MinifilterPreOperation {}

impl PreOperationVisitor for MinifilterPreOperation {
    fn create<'a>(
        &self,
        data: wdrf::minifilter::filter::FltCallbackData<'a>,
        related_obj: wdrf::minifilter::filter::FltRelatedObjects<'a>,
        create: wdrf::minifilter::filter::params::FltCreateRequest<'a>,
    ) -> wdrf::minifilter::filter::PreOpStatus {
        let file_object = data.io_params().file_object();

        if let Some(file_object) = file_object {
            let name = unsafe { &(*file_object.as_raw_obj()).FileName };

            if name.Length > 0 && name.Buffer != core::ptr::null_mut() {
                let name = unsafe {
                    NtUnicodeStr::from_raw_parts(name.Buffer, name.Length, name.MaximumLength)
                };

                maple::info!("Create new file name: {name}");
            }
        }

        PreOpStatus::SuccessNoCallback
    }
}

impl TaggedObject for MinifilterPreOperation {}
