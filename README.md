# task-control

This crate provides a `runner` to simulate what [`setInterval`](https://developer.mozilla.org/en-US/docs/Web/API/setInterval) does in JS which is repeatedly executing a task every given `Duration`.

But since in rust we can't have that kind of flexibility like js, the runner here provides much more usage except for just pass in the function and delay. Also provide a `channel` which wraps up `runner`

The exact time interval can not be guaranteed(around 0-2% extra time needed to execute logic on my laptop: [linux plateform with AMD r7-4800h, 3200mhz Ram], see also [To what point is thread::sleep accurate? ](https://www.reddit.com/r/rust/comments/15ql2af/to_what_point_is_threadsleep_accurate/))

PS: _Purely thread with no async support_.
