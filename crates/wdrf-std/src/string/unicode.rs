use wdk_sys::POOL_FLAG_NON_PAGED;

use crate::{
    kmalloc::{GlobalKernelAllocator, MemoryTag},
    vec::{Vec, VecExt},
};

use super::ntunicode::NtUnicode;

pub struct UnicodeString {
    vec: Vec<u16>,
}

const UNICODE_STRING_TAG: MemoryTag = MemoryTag::new_from_bytes(b"ustr");

impl UnicodeString {
    pub fn create() -> Self {
        Self::create_in(GlobalKernelAllocator::new(
            UNICODE_STRING_TAG,
            POOL_FLAG_NON_PAGED,
        ))
    }

    pub fn create_in(alloc: GlobalKernelAllocator) -> Self {
        Self {
            vec: Vec::new_in(alloc),
        }
    }

    pub fn from_u16(str: &[u16]) -> anyhow::Result<Self> {
        let mut buffer: Vec<u16, _> = Vec::new_in(GlobalKernelAllocator::new(
            UNICODE_STRING_TAG,
            POOL_FLAG_NON_PAGED,
        ));

        buffer
            .try_reserve_exact(str.len())
            .map_err(|_| anyhow::Error::msg("Failed to reserver exact for unicode string"))?;

        //PANIC: Can panic if src.len != self.len
        buffer.copy_from_slice(str);

        Ok(Self { vec: buffer })
    }

    pub fn from_unicode(unicode: &NtUnicode) -> anyhow::Result<Self> {
        Self::from_u16(unicode.str)
    }

    #[inline]
    pub fn try_push(&mut self, c: u16) -> anyhow::Result<()> {
        self.vec.try_push(c)
    }

    #[inline]
    pub fn try_push_str(&mut self, str: &[u16]) -> anyhow::Result<()> {
        self.vec
            .try_reserve(str.len())
            .map_err(|_| anyhow::Error::msg("Failed to reserver string for push_str"))?;

        //PANIC: If slice.len != self.len
        self.vec.extend_from_slice(str);

        Ok(())
    }

    pub fn as_nt_unicode(&mut self) -> NtUnicode<'_> {
        NtUnicode::new_from_slice(self.vec.as_mut_slice())
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u16] {
        &self.vec
    }

    #[inline]
    pub fn clear(&mut self) {
        self.vec.clear();
    }

    #[inline]
    pub fn pop(&mut self) -> Option<u16> {
        self.vec.pop()
    }

    #[inline]
    pub fn remove(&mut self, idx: usize) -> Option<u16> {
        if idx >= self.len() {
            None
        } else {
            //PANICS: If index is out of bounds
            Some(self.vec.remove(idx))
        }
    }

    #[inline]
    pub fn try_insert(&mut self, idx: usize, ch: u16) -> anyhow::Result<()> {
        self.vec.try_insert(idx, ch)
    }

    #[inline]
    pub fn chars(&mut self) -> UnicodeChars<'_> {
        UnicodeChars::new(self.as_bytes())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }
}

impl PartialEq<UnicodeString> for UnicodeString {
    #[inline]
    fn eq(&self, other: &UnicodeString) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for UnicodeString {}

impl PartialOrd<UnicodeString> for UnicodeString {
    #[inline]
    fn partial_cmp(&self, other: &UnicodeString) -> Option<core::cmp::Ordering> {
        self.vec.partial_cmp(&other.vec)
    }
}

impl Ord for UnicodeString {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.vec.cmp(&other.vec)
    }
}

impl PartialEq<NtUnicode<'_>> for UnicodeString {
    #[inline]
    fn eq(&self, other: &NtUnicode<'_>) -> bool {
        self.as_bytes() == other.str
    }
}

pub struct UnicodeChars<'a> {
    pub(super) iter: core::slice::Iter<'a, u16>,
}

impl<'a> Iterator for UnicodeChars<'a> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().cloned()
    }
}

impl<'a> DoubleEndedIterator for UnicodeChars<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().cloned()
    }
}
impl<'a> UnicodeChars<'a> {
    pub fn new(slice: &'a [u16]) -> Self {
        Self { iter: slice.iter() }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use crate::string::ntunicode::NtUnicode;

    use super::UnicodeString;

    #[test]
    fn test_basic_functions() -> anyhow::Result<()> {
        let mut basic_str = UnicodeString::create();

        basic_str.try_push('t' as u16)?;
        basic_str.try_push_str(wchar::wch!("est"))?;

        assert_eq!(basic_str.as_bytes(), wchar::wch!("test"));

        Ok(())
    }

    #[test]
    fn test_nt_string() -> anyhow::Result<()> {
        let mut basic_str = UnicodeString::create();

        basic_str.try_push_str(wchar::wch!("Hello World"))?;

        let nt_str = basic_str.as_nt_unicode();
        let hello_str = &NtUnicode::new_from_slice(wchar::wch!("Hello"));
        let world_str = &NtUnicode::new_from_slice(wchar::wch!("World"));

        assert!(nt_str.starts_with(hello_str));
        assert!(nt_str.ends_with(world_str));

        Ok(())
    }

    #[test]
    fn test_iterators() -> anyhow::Result<()> {
        let mut basic_str = UnicodeString::create();
        basic_str.try_push_str(wchar::wch!("Hello World"))?;

        assert_eq!(
            basic_str.chars().collect::<std::vec::Vec<u16>>(),
            wchar::wch!("Hello World")
        );

        Ok(())
    }
}
