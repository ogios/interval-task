# channel

This is a wrapper of [runner][`crate::runner`].

It returns sender & receiver created by [`async_channel`], and a [`Runner`][`crate::runner::Runner`] instance that implements [`ExternalRunnerExt`][`crate::runner::ExternalRunnerExt`].

It will send [`TASK_SIGNAL`] signal and wait for [`TASK_DONE`] signal repeatedly every fixed [`Duration`] .

## Example

```rust
let (s, r, mut runner) = channel::new(Duration::from_micros(1_000_000 / 120));
runner.start().unwrap();
let start = Instant::now();
for _ in 0..120 {
    r.recv_blocking().unwrap();
    s.send_blocking(TASK_DONE).unwrap();
}
println!("Elapsed: {:?}", start.elapsed());
runner.close().unwrap();
```
