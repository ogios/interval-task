#![doc = include_str!("channel.md")]

use super::runner::{self, Task};
use super::runner::{ExternalRunnerExt, Runner};
use async_channel as achannel;
use std::time::Duration;

pub const TASK_SIGNAL: u8 = 0;
pub const TASK_DONE: u8 = TASK_SIGNAL;

/// Create a [`runner::Runner`].
/// Signals every `interval`.
pub fn new(interval: Duration) -> (achannel::Receiver<u8>, Runner<Task>) {
    let (sub_sender, main_receiver) = achannel::bounded::<u8>(1);
    let f = move || {
        if let Err(e) = sub_sender.send_blocking(TASK_SIGNAL) {
            panic!("[task-controler] error sending start signal: {}", e);
        }
    };

    let mut runner = runner::new_external_close_runner(interval);
    runner.set_task(Box::new(f));
    (main_receiver, runner)
}

/// Create a [`runner::Runner`].
/// Signals every `interval` and waits for [`TASK_DONE`] signal.
pub fn new_blocking(
    interval: Duration,
) -> (achannel::Sender<u8>, achannel::Receiver<u8>, Runner<Task>) {
    let (sub_sender, main_receiver) = achannel::bounded::<u8>(1);
    let (main_sender, sub_receiver) = achannel::bounded::<u8>(1);

    let f = move || {
        if let Err(e) = sub_sender.send_blocking(TASK_SIGNAL) {
            panic!("[task-controler] error sending start signal: {}", e);
        }
        if let Err(e) = sub_receiver.recv_blocking() {
            panic!("[task-controler] error receiving done signal: {}", e)
        }
    };

    let mut runner = runner::new_external_close_runner(interval);
    runner.set_task(Box::new(f));
    (main_sender, main_receiver, runner)
}
