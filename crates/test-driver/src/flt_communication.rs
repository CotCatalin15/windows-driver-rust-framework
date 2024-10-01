use wdrf::minifilter::{
    communication::client_communication::{FltClientCommunication, FltCommunicationCallback},
    FltFilter,
};
use wdrf_std::{dbg_break, io::Write, slice::tracked_slice::TrackedSlice, NtResult};

use maple::info;

pub struct FltCallbackImpl {}

impl FltCommunicationCallback for FltCallbackImpl {
    fn connect(&self, _buffer: Option<&[u8]>) -> anyhow::Result<()> {
        dbg_break();

        Ok(())
    }

    fn message(&self, input: &[u8], output: Option<&mut TrackedSlice>) -> anyhow::Result<()> {
        dbg_break();

        if input.len() > 0 {
            let s = core::str::from_utf8(input)
                .map_err(|_| anyhow::Error::msg("Failed to convert message into utf8 string"))?;
            info!("Test {s}");
        }

        if let Some(output) = output {
            output.write(input)?;
        }

        Ok(())
    }

    fn disconnect(&self) {
        dbg_break();
    }
}

pub fn create_communication(
    filter: FltFilter,
) -> NtResult<FltClientCommunication<FltCallbackImpl>> {
    let port_name = nt_string::nt_unicode_str!("\\TESTPORT");

    FltClientCommunication::new(FltCallbackImpl {}, filter, port_name)
}
