#[repr(align(8))]
pub struct AsAligned<T> {
    pub ptr: T,
}
