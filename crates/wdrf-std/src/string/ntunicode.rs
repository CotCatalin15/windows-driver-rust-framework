use core::marker::PhantomData;

use wdk_sys::{
    ntddk::{
        RtlCompareUnicodeString, RtlDowncaseUnicodeChar, RtlInitUnicodeString,
        RtlPrefixUnicodeString, RtlSuffixUnicodeString, RtlUpcaseUnicodeString, RtlUpperChar,
    },
    UNICODE_STRING,
};

pub struct NtUnicode<'a> {
    data: UNICODE_STRING,
    str: &'a mut [u16],
}

impl<'a> NtUnicode<'a> {
    pub fn new(unicode: &'a mut UNICODE_STRING) -> Self {
        Self {
            data: *unicode,
            str: unsafe { core::slice::from_raw_parts_mut(unicode.Buffer, unicode.Length as _) },
        }
    }

    pub fn lower(&mut self) {
        self.str
            .iter_mut()
            .for_each(|c| unsafe { *c = RtlDowncaseUnicodeChar(*c) });
    }

    pub fn starts_with(&self, other: &NtUnicode, case_sensitive: bool) -> anyhow::Result<()> {
        unsafe {
            if 1 == RtlPrefixUnicodeString(&self.data, &other.data, (!case_sensitive) as _) {
                Ok(())
            } else {
                Err(anyhow::Error::msg("test"))
            }
        }
    }

    pub fn ends_with(&self, other: &NtUnicode, case_sensitive: bool) -> anyhow::Result<()> {
        unsafe {
            if 1 == RtlSuffixUnicodeString(&self.data, &other.data, (!case_sensitive) as _) {
                Ok(())
            } else {
                Err(anyhow::Error::msg("test"))
            }
        }
    }

    pub fn equals(&self, other: &NtUnicode, case_sensitive: bool) -> bool {}
}

impl<'a, 'b> PartialEq<NtUnicode<'b>> for NtUnicode<'a> {
    fn eq(&self, other: &NtUnicode<'b>) -> bool {
        unsafe { RtlCompareUnicodeString(&self.data, &other.data, 1) == 0 }
    }
}

impl<'a> Eq for NtUnicode<'a> {}

impl<'a> Ord for NtUnicode<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        todo!()
    }
}
