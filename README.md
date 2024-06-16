# interval-task

This crate provides a `runner` to simulate what [`setInterval`](https://developer.mozilla.org/en-US/docs/Web/API/setInterval) does in JS which is repeatedly executing a task every given `Duration`.

But since in rust we can't have that kind of flexibility like js, the runner here provides much more usage except for just pass in the function and delay.  
Also provide a `channel` which wraps up `runner`.

# Examples

Examples of executing task 120 times in 1 second

<details>

  <summary>

    ## channel

  </summary>

```rust
use interval_task::{
    channel::{self, TASK_DONE},
    runner::ExternalRunnerExt,
};

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
```

</details>

<details>

  <summary>

    ## runner

  </summary>

```rust
// manually call `close`
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

    let (s, r) = async_channel::bounded(1);
    let mut runner = runner::new_external_close_runner(Duration::from_micros(1_000_000 / 120));
    runner.set_task(Task::new_fn_mut_task(TestTask(0, s)));

    runner.start().unwrap();    // start runner
    let start = Instant::now(); // start count
    r.recv_blocking().unwrap(); // wait for signal from `Task`
    println!("Elapsed: {:?}", start.elapsed());
    runner.close().unwrap();
}

// close inside `Task` by returning `bool`
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
    runner.join().unwrap(); // wait for `Task` close inside
}
```

</details>

# Other

The exact time interval can not be guaranteed(around 0-2% extra time needed to execute logic on my laptop: [linux plateform with AMD r7-4800h, 3200mhz Ram], see also [To what point is thread::sleep accurate? ](https://www.reddit.com/r/rust/comments/15ql2af/to_what_point_is_threadsleep_accurate/))
