use ringbuf::Producer;

use crate::error::StreamError;
use crate::StreamMsg;

use super::push_stream_msg;

pub struct JackNotificationHandler {
    to_msg_channel_tx: Producer<StreamMsg>,
    sample_rate: u32,
}

impl JackNotificationHandler {
    pub fn new(to_msg_channel_tx: Producer<StreamMsg>, sample_rate: u32) -> Self {
        Self { to_msg_channel_tx, sample_rate }
    }
}

impl jack::NotificationHandler for JackNotificationHandler {
    fn thread_init(&mut self, _: &jack::Client) {
        log::debug!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        let msg = format!("JACK: shutdown with status {:?} because \"{}\"", status, reason);

        log::error!("{}", msg);

        push_stream_msg(
            &mut self.to_msg_channel_tx,
            StreamMsg::Error(StreamError::AudioServerShutdown { msg: Some(msg) }),
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        log::debug!("JACK: freewheel mode is {}", if is_enabled { "on" } else { "off" });
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        // Why does Jack allow changing the samplerate mid-stream?!
        // Just shut down the audio thread in this case.
        if srate != self.sample_rate {
            log::error!("JACK: sample rate changed to {}", srate);

            push_stream_msg(
                &mut self.to_msg_channel_tx,
                StreamMsg::Error(StreamError::AudioServerChangedSamplerate(srate)),
            );

            return jack::Control::Quit;
        }

        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        log::debug!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        log::debug!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        log::debug!("JACK: port with id {} renamed from {} to {}", port_id, old_name, new_name);
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        log::debug!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected { "connected" } else { "disconnected" }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        log::debug!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        //log::warn!("JACK: xrun occurred");
        jack::Control::Continue
    }
}
