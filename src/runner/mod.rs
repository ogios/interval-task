#![doc = include_str!("runner.md")]

use async_channel as ac;
use std::thread::{self, JoinHandle, Thread};
use std::time::{Duration, Instant};

/// this function accepts raw impl FnMut
/// if you have func within box, use [`Runner::new`]
pub fn new_runner<T: 'static>(
    interval: Duration,
    ctx_func: impl Send + 'static + FnOnce() -> T,
    task: impl Send + 'static + FnMut(&mut T) -> bool,
) -> Runner<T> {
    Runner::new(interval, Box::new(task), Box::new(ctx_func))
}

/// return [`true`] to break the loop and stop runner
/// don't forget you can call [`Runner::join`] to wait for runner thread to finish
/// for arg, see [`CtxFunc<T>`]
pub type TaskWithHandle<T> = Box<dyn Send + 'static + FnMut(&mut T) -> bool>;

/// This will be executed inside runner thread when started
/// And will be passed as arg to [`TaskWithHandle<T>`]
pub type CtxFunc<T> = Box<dyn Send + 'static + FnOnce() -> T>;

struct Task<T> {
    task: TaskWithHandle<T>,
    ctx_func: CtxFunc<T>,
    interval: Duration,
}
unsafe impl<T> Send for Task<T> {}
unsafe impl<T> Sync for Task<T> {}

/// the basic runner
/// the runner can only [`Runner::start`] once
pub struct Runner<T> {
    t: Option<Box<Task<T>>>,

    stop_signal_sender: Option<ac::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}
unsafe impl<T> Send for Runner<T> {}
unsafe impl<T> Sync for Runner<T> {}

impl<T: 'static> Runner<T> {
    pub fn new(interval: Duration, task: TaskWithHandle<T>, ctx_func: CtxFunc<T>) -> Self {
        Runner {
            t: Some(Box::new(Task {
                interval,
                task,
                ctx_func,
            })),
            stop_signal_sender: None,
            thread: None,
        }
    }

    fn take_task(&mut self) -> Result<Task<T>, &'static str> {
        if let Some(t) = self.t.take() {
            Ok(*t)
        } else {
            Err("ctx func not set")
        }
    }

    pub fn get_thread_ref(&self) -> &Thread {
        self.thread.as_ref().unwrap().thread()
    }

    pub fn start(&mut self) -> Result<(), &str> {
        if self.thread.is_some() {
            return Ok(());
        }

        let (stop_signal_sender, stop_signal_receiver) = ac::bounded::<()>(1);

        self.stop_signal_sender = Some(stop_signal_sender);

        let task = self.take_task().unwrap();
        self.thread = Some(thread::spawn(move || {
            let interval = task.interval;
            let mut ctx = (task.ctx_func)();
            let mut task = task.task;

            let mut last_cost = Duration::from_nanos(0);

            loop {
                let frame_start = Instant::now() - last_cost;

                if let Err(async_channel::TryRecvError::Empty) = stop_signal_receiver.try_recv() {
                } else {
                    break;
                }

                if task(&mut ctx) {
                    break;
                };

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

    pub fn close(mut self) -> Result<(), &'static str> {
        if let Some(t) = self.thread.take() {
            let _ = self.stop_signal_sender.as_ref().unwrap().send_blocking(());
            t.join().unwrap();
            return Ok(());
        };
        Err("no task running")
    }

    pub fn join(mut self) -> Result<(), &'static str> {
        if let Some(t) = self.thread.take() {
            t.join().unwrap();
            let _ = self.stop_signal_sender.as_ref().unwrap().close();
            return Ok(());
        };
        Err("no task running")
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::*;

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
        runner.join().unwrap();
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
