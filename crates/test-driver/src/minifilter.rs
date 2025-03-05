use wdrf::minifilter::filter::{
    params::FltParameters, FileNameInformation, FltPreOpCallback, PreOpStatus,
};
use wdrf_std::{kmalloc::TaggedObject, nt_success};
use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;

pub struct MinifilterOperations {}

impl FltPreOpCallback for MinifilterOperations {
    fn callback<'a>(
        &self,
        data: wdrf::minifilter::filter::FltCallbackData<'a>,
        related_obj: wdrf::minifilter::filter::FltRelatedObjects<'a>,
        params: FltParameters<'a>,
    ) -> PreOpStatus {
        match params {
            FltParameters::Create(create) => {
                let name = FileNameInformation::create(&data);

                if let Ok(name) = name {
                    let slice = name.name().as_u16str().as_slice();
                    let a_txt = widestring::u16str!("a.txt").as_slice();
                    if slice.windows(a_txt.len()).any(|window| window == a_txt) {
                        PreOpStatus::Complete(STATUS_ACCESS_DENIED, 0)
                    } else {
                        PreOpStatus::SuccessNoCallback
                    }
                } else {
                    PreOpStatus::SuccessNoCallback
                }
            }
            _ => PreOpStatus::SuccessNoCallback,
        }
    }
}

impl TaggedObject for MinifilterOperations {}
