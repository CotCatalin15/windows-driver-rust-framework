use core::num::NonZeroU32;

use anyhow::bail;
use wdrf::minifilter::{
    communication::client_communication::{
        FltClient, FltClientCommunication, FltCommunicationCallback,
    },
    FltFilter,
};
use wdrf_std::{io::Write, slice::tracked_slice::TrackedSlice, sync::arc::Arc};

pub struct FltCallbackImpl {}

impl FltCommunicationCallback for FltCallbackImpl {
    type ClientCookie = ();

    fn on_connect(
        &self,
        client: &Arc<FltClient<Self::ClientCookie>>,
        connection_buf: &[u8],
    ) -> anyhow::Result<Option<Self::ClientCookie>> {
        Ok(())
    }

    fn on_disconnect(&self, client: &FltClient<Self::ClientCookie>) {}

    fn on_message(
        &self,
        cookie: Option<&Self::ClientCookie>,
        input: &[u8],
        output: &mut TrackedSlice,
    ) -> anyhow::Result<()> {
        output.write(input)?;

        Ok(())
    }
}

pub fn create_communication(
    filter: Arc<FltFilter>,
) -> anyhow::Result<FltClientCommunication<FltCallbackImpl>> {
    let port_name = nt_string::nt_unicode_str!("\\TESTPORT");

    FltClientCommunication::new(
        FltCallbackImpl {},
        filter,
        port_name,
        NonZeroU32::new(1).unwrap(),
    )
}