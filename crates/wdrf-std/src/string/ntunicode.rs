use wdk_sys::ntddk::RtlCompareUnicodeString;
use wdk_sys::UNICODE_STRING;

#[derive(Clone, Copy)]
pub struct NtUnicode<'a> {
    data: UNICODE_STRING,
    pub str: &'a [u16],
}

impl<'a> NtUnicode<'a> {
    pub fn new(unicode: &'a UNICODE_STRING) -> Self {
        Self {
            data: *unicode,
            str: unsafe { core::slice::from_raw_parts_mut(unicode.Buffer, unicode.Length as _) },
        }
    }

    pub fn new_from_slice(slice: &'a [u16]) -> Self {
        Self {
            data: UNICODE_STRING {
                Length: slice.len() as _,
                MaximumLength: slice.len() as _,
                Buffer: slice.as_ptr() as _,
            },
            str: slice,
        }
    }

    pub fn starts_with(&self, prefix: &NtUnicode) -> bool {
        self.str.starts_with(prefix.str)
    }

    pub fn ends_with(&self, sufix: &NtUnicode) -> bool {
        self.str.ends_with(sufix.str)
    }
}

impl<'a, 'b> PartialEq<NtUnicode<'b>> for NtUnicode<'a> {
    fn eq(&self, other: &NtUnicode<'b>) -> bool {
        unsafe { RtlCompareUnicodeString(&self.data, &other.data, 0) == 0 }
    }
}

impl<'a> Eq for NtUnicode<'a> {}

impl<'a> PartialOrd for NtUnicode<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.str.partial_cmp(&other.str)
    }
}

impl<'a> Ord for NtUnicode<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.str.cmp(&other.str)
    }
}
