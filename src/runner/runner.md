# Runner

[Runner][`runner::Runner`] have 2 usage:

- externally call [`runner::ExternalRunnerExt::close()`].
- return [`true`] in [`task::FnTaskWithHandle::call`] or [`task::FnMutTaskWithHandle::call_mut`] and run [`runner::InternalRunnerExt::join()`] to wait until runner thread exit.

## Examples

```rust
// example 1
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
    runner.start().unwrap();
    let start = Instant::now();
    r.recv_blocking().unwrap();
    println!("Elapsed: {:?}", start.elapsed());
    runner.close().unwrap();
}

// example 2
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
    runner.join().unwrap();
}
```
