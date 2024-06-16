#![allow(dead_code)]
use std::time::{Duration, Instant};
extern crate interval_task;

fn main() {
    println!("internal close:");
    internal_close_example();
    println!("external close:");
    external_close_example();
}

fn external_close_example() {
    use interval_task::runner::{self, ExternalRunnerExt, FnMutTask, Task};

    struct TestTask(u32, async_channel::Sender<u8>);
    impl FnMutTask for TestTask {
        fn call_mut(&mut self) {
            if self.0 == 119 {
                self.1.send_blocking(0).unwrap();
            } else {
                self.0 += 1
            }
        }
    }

    let mut runner = runner::new_external_close_runner(Duration::from_micros(1_000_000 / 120));
    let (s, r) = async_channel::bounded(1);
    runner.set_task(Task::new_fn_mut_task(TestTask(0, s)));
    runner.start().unwrap();
    let start = Instant::now();
    r.recv_blocking().unwrap();
    println!("Elapsed: {:?}", start.elapsed());
    runner.close().unwrap();
}

fn internal_close_example() {
    use interval_task::runner::{self, InternalRunnerExt, TaskWithHandle};
    use std::time::{Duration, Instant};

    struct RunnerTask(u32, Instant);
    impl runner::FnMutTaskWithHandle for RunnerTask {
        fn call_mut(&mut self) -> bool {
            if self.0 == 119 {
                println!("{}", self.1.elapsed().as_secs_f64());
                true
            } else {
                if self.0 == 0 {
                    self.1 = Instant::now();
                }
                self.0 += 1;
                false
            }
        }
    }
    let mut runner = runner::new_internal_close_runner(Duration::from_micros(1_000_000 / 120));
    runner.set_task(TaskWithHandle::new_fn_mut_task(RunnerTask(
        0,
        Instant::now(),
    )));
    runner.start().unwrap();
    runner.join().unwrap();
}
