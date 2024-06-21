use interval_task::{
    channel::{self, TASK_DONE},
    runner::ExternalRunnerExt,
};
use std::time::{Duration, Instant};

fn main() {
    println!("blocking:");
    blocking();
    println!("no blocking:");
    no_blocking();
}

fn blocking() {
    let (s, r, mut runner) = channel::new_blocking(Duration::from_micros(1_000_000 / 120));
    runner.start().unwrap();
    let start = Instant::now();
    for _ in 0..120 {
        r.recv_blocking().unwrap();
        s.send_blocking(TASK_DONE).unwrap();
    }
    println!("Elapsed: {:?}", start.elapsed());
    runner.close().unwrap();
}

fn no_blocking() {
    let (r, mut runner) = channel::new(Duration::from_micros(1_000_000 / 120));
    runner.start().unwrap();
    let start = Instant::now();
    for _ in 0..120 {
        r.recv_blocking().unwrap();
    }
    println!("Elapsed: {:?}", start.elapsed());
    runner.close().unwrap();
}
