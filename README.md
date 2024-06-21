# interval-task

This crate provides a `runner` to simulate what [`setInterval`](https://developer.mozilla.org/en-US/docs/Web/API/setInterval) does in JS which is repeatedly executing a task every given `Duration`.

But since in rust we can't have that kind of flexibility like js, the runner here provides much more usage except for just pass in the function and delay.  
Also provide a `channel` which wraps up `runner`.

# Examples

Examples of executing task 120 times in 1 second

## channel

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

## runner

```rust
// manually call `close`
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

// close inside `Task` by returning `bool`
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
```

# Other

[To what point is thread::sleep accurate? ](https://www.reddit.com/r/rust/comments/15ql2af/to_what_point_is_threadsleep_accurate/)
