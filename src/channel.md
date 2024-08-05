# channel

This is a wrapper of [runner][`crate::runner`].

- block: use `send_blocking` and blocked in the next loop if you don't `recv`.
- non-block: use `force_send` to replace the last signal.
- unbounded: use [`async_channel::unbounded`] which would be `non-block`.
