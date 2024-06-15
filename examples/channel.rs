use std::{
    env::args,
    time::{Duration, Instant},
};
use task_control::{
    channel::{self, TASK_DONE},
    runner::ExternalRunnerExt,
};
extern crate task_control;

fn main() {
    let mut arg = args();
    arg.next().unwrap();
    if let Some(i) = arg.next() {
        if i == *"1" {
            return a();
        }
    }
    normal()
}

fn normal() {
    let (s, r, mut runner) = channel::new(Duration::from_micros(1_000_000 / 120));
    runner.start().unwrap();
    let start = Instant::now();
    for _ in 0..120 {
        r.recv_blocking().unwrap();
        s.send_blocking(TASK_DONE).unwrap();
    }
    println!("Elapsed: {:?}", start.elapsed());
    runner.close().unwrap();
}

#[allow(unused, dead_code)]
fn a() {
    let (s, r, mut handler) = channel::new(Duration::from_micros(1_000_000 / 120));
    handler.start().unwrap();
    handler.start().unwrap();
}
