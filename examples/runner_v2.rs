#![allow(dead_code)]
use std::time::{Duration, Instant};

fn main() {
    println!("internal close:");
    internal_close_example();
    println!("external close:");
    external_close_example();
}

fn external_close_example() {
    use interval_task::runner::{self, ExternalRunnerExt};

    let mut count = 0;
    let (s, r) = async_channel::bounded(1);
    let f = move || {
        if count == 119 {
            s.send_blocking(0).unwrap();
        } else {
            count += 1
        }
    };

    let mut runner = runner::new_external_close_runner(Duration::from_micros(1_000_000 / 120));
    runner.set_task(Box::new(f));

    runner.start().unwrap(); // start runner
    let start = Instant::now(); // start count
    r.recv_blocking().unwrap(); // wait for signal from `Task`
    println!("Elapsed: {:?}", start.elapsed());
    runner.close().unwrap();
}

fn internal_close_example() {
    use interval_task::runner::{self, InternalRunnerExt};
    use std::time::{Duration, Instant};

    let mut count = 0;
    let mut ins = Instant::now();
    let f = move || {
        if count == 119 {
            println!("{}", ins.elapsed().as_secs_f64());
            true
        } else {
            if count == 0 {
                ins = Instant::now();
            }
            count += 1;
            false
        }
    };

    let mut runner = runner::new_internal_close_runner(Duration::from_micros(1_000_000 / 120));
    runner.set_task(Box::new(f));
    runner.start().unwrap();
    runner.join().unwrap(); // wait for `Task` close inside
}
