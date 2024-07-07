pub mod tracked_slice;

///
/// # Safety
///
/// In case of a null or 0 size pointer it returns an empty array
///
pub unsafe fn slice_from_raw_parts_or_empty<'a, T>(data: *const T, len: usize) -> &'a [T] {
    if data.is_null() && len == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(data, len)
    }
}

///
/// # Safety
///
/// In case of a null or 0 size pointer it returns an empty array
///
pub unsafe fn slice_from_raw_parts_mut_or_empty<'a, T>(data: *mut T, len: usize) -> &'a mut [T] {
    if data.is_null() && len == 0 {
        &mut []
    } else {
        core::slice::from_raw_parts_mut(data, len)
    }
}
