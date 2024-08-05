# Runner

[Runner][`runner::Runner`] have 2 usage:

- externally call [`runner::ExternalRunnerExt::close()`].
- return [`true`] in [`runner::TaskWithHandle<T>`] and run [`runner::Runner::join`] to wait until runner thread exit.

Runner is run inside another thread, but don't feel bad if you have a [`!Send`] or [`!Sync`] object needs to be used inside task, you can provide a [`runner::CtxFunc<T>`] function, it will be called in runner thread on start, and be passed in task each loop.

## Examples

Checkout [`runner::tests`]
