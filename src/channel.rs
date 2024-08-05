#![doc = include_str!("channel.md")]

use super::runner::Runner;
use super::runner::{self};
use async_channel as achannel;
use std::time::Duration;

pub const TASK_SIGNAL: () = ();

/// Create a [`runner::Runner`].
/// Signals every `interval`.
pub fn new(interval: Duration, block: bool) -> (achannel::Receiver<()>, Runner<()>) {
    let (sub_sender, main_receiver) = achannel::bounded::<()>(1);
    let runner = if block {
        runner::Runner::new(
            interval,
            || (),
            move |_| {
                if let Err(e) = sub_sender.send_blocking(TASK_SIGNAL) {
                    panic!("[task-controler] error sending start signal: {}", e);
                }
                false
            },
        )
    } else {
        runner::Runner::new(
            interval,
            || (),
            move |_| {
                if let Err(e) = sub_sender.force_send(TASK_SIGNAL) {
                    panic!("[task-controler] error sending start signal: {}", e);
                }
                false
            },
        )
    };
    (main_receiver, runner)
}

pub fn new_unbound(interval: Duration) -> (achannel::Receiver<()>, Runner<()>) {
    let (sub_sender, main_receiver) = achannel::unbounded::<()>();
    let runner = runner::Runner::new(
        interval,
        || (),
        move |_| {
            if let Err(e) = sub_sender.send_blocking(TASK_SIGNAL) {
                panic!("[task-controler] error sending start signal: {}", e);
            }
            false
        },
    );
    (main_receiver, runner)
}
