#![doc = include_str!("runner.md")]

use educe::Educe;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub fn new_runner<T: 'static>(
    interval: Duration,
    ctx_func: impl Send + 'static + FnOnce() -> T,
    task: impl Send + 'static + FnMut(&mut T) -> bool,
) -> Runner<T> {
    Runner::new(interval, ctx_func, task)
}

struct Task<T> {
    /// return [`true`] to break the loop and stop runner
    /// don't forget you can call [`Runner::join`] to wait for runner thread to finish
    /// for arg, see [`Task<T>::ctx_func`]
    task: Box<dyn Send + 'static + FnMut(&mut T) -> bool>,

    /// This will be executed inside runner thread when started
    /// And will be passed as arg to [`Task<T>::task`]
    ctx_func: Box<dyn Send + 'static + FnOnce() -> T>,

    interval: Duration,
}
unsafe impl<T> Send for Task<T> {}
unsafe impl<T> Sync for Task<T> {}

/// the basic runner
///
/// the runner can only [`Runner::start`] once.
/// drop the runner or return `true` in task to stop it.
/// you can take the thread join handle to wait until it stopped.
#[derive(Educe, Default)]
#[educe(Debug)]
pub struct Runner<T> {
    pub thread: Option<JoinHandle<()>>,
    guard: Option<Arc<()>>,

    #[educe(Debug(ignore))]
    t: Option<Task<T>>,
}
unsafe impl<T> Send for Runner<T> {}
unsafe impl<T> Sync for Runner<T> {}

impl<T: 'static> Runner<T> {
    pub fn new(
        interval: Duration,
        ctx_func: impl Send + 'static + FnOnce() -> T,
        task: impl Send + 'static + FnMut(&mut T) -> bool,
    ) -> Self {
        Runner {
            t: Some(Task {
                interval,
                task: Box::new(task),
                ctx_func: Box::new(ctx_func),
            }),
            guard: None,
            thread: None,
        }
    }

    pub fn start(&mut self) -> Result<(), &str> {
        if self.thread.is_some() {
            return Ok(());
        }

        let handle = Arc::new(());
        let handle_weak = Arc::downgrade(&handle);

        self.guard = Some(handle);

        let task = self.t.take().unwrap();
        self.thread = Some(thread::spawn(move || {
            if handle_weak.upgrade().is_none() {
                return;
            }

            let interval = task.interval;
            let mut ctx = (task.ctx_func)();
            let mut task = task.task;

            let mut last_cost = Duration::from_nanos(0);

            loop {
                let frame_start = Instant::now() - last_cost;

                if task(&mut ctx) {
                    break;
                };

                if handle_weak.upgrade().is_none() {
                    return;
                }

                let last_cost_start = Instant::now();
                if let Some(gap) = interval.checked_sub(frame_start.elapsed()) {
                    spin_sleep::sleep(gap);
                    last_cost = last_cost_start.elapsed() - gap;
                } else {
                    last_cost = last_cost_start.elapsed();
                }
            }
        }));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc, sync::Mutex};

    use super::*;

    #[test]
    fn arc_test() {
        let interval = Duration::from_micros(1_000_000 / 60);
        let count = Arc::new(Mutex::new(0));
        let count_c = count.clone();
        let mut runner = new_runner(
            interval,
            || (),
            move |_| {
                let mut c = count.lock().unwrap();
                *c += 1;
                false
            },
        );

        let start = Instant::now();
        runner.start().unwrap();
        std::thread::sleep(Duration::from_millis(1000));
        drop(runner);
        println!("Elapsed: {:?}", start.elapsed());

        let c = *count_c.lock().unwrap();
        println!("count: {}", c);
        assert!(c >= 60);
    }

    fn normal_internal(fps: u64) {
        let interval = Duration::from_micros(1_000_000 / fps);
        let mut runner = new_runner(
            interval,
            || Rc::new(Cell::new(0)),
            move |count| {
                count.set(count.get() + 1);
                count.get() == fps
            },
        );

        let start = Instant::now();
        runner.start().unwrap();
        runner.thread.map(|j| j.join());
        println!("Elapsed: {:?}", start.elapsed());
    }

    #[test]
    fn runner_test_60_fps() {
        normal_internal(60)
    }

    #[test]
    fn runner_test_120_fps() {
        normal_internal(120)
    }

    #[test]
    fn runner_test_144_fps() {
        normal_internal(144)
    }

    #[test]
    fn runner_test_240_fps() {
        normal_internal(240)
    }

    #[test]
    fn runner_test_1000_fps() {
        normal_internal(1000)
    }
}
