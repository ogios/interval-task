# interval-task

This crate provides a [`runner`][`crate::runner`] to simulate what [`setInterval`](https://developer.mozilla.org/en-US/docs/Web/API/setInterval) does in JS which is repeatedly executing a task every given [`Duration`][`std::time::Duration`].

But since in rust we can't have that kind of flexibility like js, the runner here provides much more usage except for just pass in the function and delay. Also provide a [channel][`channel`] which wraps up [runner][`crate::runner`]

[To what point is thread::sleep accurate? ](https://www.reddit.com/r/rust/comments/15ql2af/to_what_point_is_threadsleep_accurate/)

_Purely thread with no async support_.
