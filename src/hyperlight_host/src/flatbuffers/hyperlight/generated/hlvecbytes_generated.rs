// automatically generated by the FlatBuffers compiler, do not modify
// @generated
extern crate alloc;
extern crate flatbuffers;
use self::flatbuffers::{EndianScalar, Follow};
use super::*;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::mem;
pub enum hlvecbytesOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct hlvecbytes<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for hlvecbytes<'a> {
    type Inner = hlvecbytes<'a>;
    #[inline]
    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table::new(buf, loc),
        }
    }
}

impl<'a> hlvecbytes<'a> {
    pub const VT_VALUE: flatbuffers::VOffsetT = 4;

    #[inline]
    pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        hlvecbytes { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args hlvecbytesArgs<'args>,
    ) -> flatbuffers::WIPOffset<hlvecbytes<'bldr>> {
        let mut builder = hlvecbytesBuilder::new(_fbb);
        if let Some(x) = args.value {
            builder.add_value(x);
        }
        builder.finish()
    }

    #[inline]
    pub fn value(&self) -> Option<flatbuffers::Vector<'a, u8>> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(
                    hlvecbytes::VT_VALUE,
                    None,
                )
        }
    }
}

impl flatbuffers::Verifiable for hlvecbytes<'_> {
    #[inline]
    fn run_verifier(
        v: &mut flatbuffers::Verifier,
        pos: usize,
    ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
        use self::flatbuffers::Verifiable;
        v.visit_table(pos)?
            .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>(
                "value",
                Self::VT_VALUE,
                false,
            )?
            .finish();
        Ok(())
    }
}
pub struct hlvecbytesArgs<'a> {
    pub value: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
}
impl<'a> Default for hlvecbytesArgs<'a> {
    #[inline]
    fn default() -> Self {
        hlvecbytesArgs { value: None }
    }
}

pub struct hlvecbytesBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> hlvecbytesBuilder<'a, 'b> {
    #[inline]
    pub fn add_value(&mut self, value: flatbuffers::WIPOffset<flatbuffers::Vector<'b, u8>>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(hlvecbytes::VT_VALUE, value);
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> hlvecbytesBuilder<'a, 'b> {
        let start = _fbb.start_table();
        hlvecbytesBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<hlvecbytes<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

impl core::fmt::Debug for hlvecbytes<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("hlvecbytes");
        ds.field("value", &self.value());
        ds.finish()
    }
}
