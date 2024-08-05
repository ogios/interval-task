# interval-task

This crate provides a [`runner`][`crate::runner`] to simulate what [`setInterval`](https://developer.mozilla.org/en-US/docs/Web/API/setInterval) does in JS which is repeatedly executing a task every given [`Duration`][`std::time::Duration`].

But since in rust we can't have that kind of flexibility like js, the runner here provides much more usage except for just pass in the function and delay. Also provide a [channel][`channel`] which wraps up [runner][`crate::runner`]

Please be aware that you have read [`runner`][`create::runner`] doc.

This crate uses [`spin-sleep`] which provides accurate sleep. and i optimized to make the loop more accurate.

_Purely thread with no async support_.
