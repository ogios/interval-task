# Runner

[`Runner`] have 2 usage:

- externally call [`Runner::close()`].
- return [`true`] in [`TaskWithHandle<T>`] and run [`Runner::join`] to wait until runner thread exit.

Runner is run inside another thread, but don't feel bad if you have a [`!Send`][`std::marker::Send`] or [`!Sync`][`std::marker::Sync`] object needs to be used inside task, you can provide a [`CtxFunc<T>`] function, it will be called in runner thread on start, and be passed in task each loop.

## Examples

Checkout test mod under runner
