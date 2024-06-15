#![doc = include_str!("channel.md")]

use crate::runner::Runner;
use crate::runner::{self, Task};
use async_channel as achannel;
use std::time::Duration;

pub const TASK_SIGNAL: u8 = 0;
pub const TASK_DONE: u8 = TASK_SIGNAL;

struct Test {
    s: achannel::Sender<u8>,
    r: achannel::Receiver<u8>,
}
impl runner::FnTask for Test {
    fn call(&self) {
        if let Err(e) = self.s.send_blocking(TASK_SIGNAL) {
            panic!("[task-controler] error sending start signal: {}", e);
        }
        if let Err(e) = self.r.recv_blocking() {
            panic!("[task-controler] error receiving done signal: {}", e)
        }
    }
}

/// Create a [`runner::Runner`].
/// Signals every `interval`.
pub fn new(interval: Duration) -> (achannel::Sender<u8>, achannel::Receiver<u8>, Runner<Task>) {
    let (sub_sender, main_receiver) = achannel::bounded::<u8>(1);
    let (main_sender, sub_receiver) = achannel::bounded::<u8>(1);

    let mut runner = runner::new_external_close_runner(interval);
    runner.set_task(runner::Task::TaskFn(Box::new(Test {
        s: sub_sender,
        r: sub_receiver,
    })));
    (main_sender, main_receiver, runner)
}
