///
/// # Safety
/// If a structure can be safety called at dispatch it must
/// implement this trait
///
pub unsafe trait DispatchSafe {}

unsafe impl DispatchSafe for i8 {}
unsafe impl DispatchSafe for u8 {}

unsafe impl DispatchSafe for i16 {}
unsafe impl DispatchSafe for u16 {}

unsafe impl DispatchSafe for i32 {}
unsafe impl DispatchSafe for u32 {}

unsafe impl DispatchSafe for i64 {}
unsafe impl DispatchSafe for u64 {}

unsafe impl DispatchSafe for f32 {}
unsafe impl DispatchSafe for f64 {}

unsafe impl DispatchSafe for bool {}
unsafe impl DispatchSafe for char {}

unsafe impl DispatchSafe for isize {}
unsafe impl DispatchSafe for usize {}

//TODO: Add
