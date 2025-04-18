#[repr(align(8))]
pub struct AsAligned<T> {
    pub ptr: T,
}

impl<T: Copy> AsAligned<T> {
    pub fn get(&self) -> T {
        self.ptr
    }
}
