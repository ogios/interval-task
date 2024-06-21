#![doc = include_str!("runner.md")]

use async_channel as ac;
use std::thread::{self, JoinHandle, Thread};
use std::time::{Duration, Instant};

pub trait ExternalRunnerExt {
    fn set_task(&mut self, t: Task);
    fn start(&mut self) -> Result<(), &str>;
    fn close(self) -> Result<(), &'static str>;
}
pub trait InternalRunnerExt {
    fn set_task(&mut self, t: TaskWithHandle);
    fn start(&mut self) -> Result<(), &str>;
    fn join(self) -> Result<(), &'static str>;
}

pub type Task = Box<dyn FnMut() + Send + 'static>;
pub type TaskWithHandle = Box<dyn Send + 'static + FnMut() -> bool>;

pub struct Runner<T> {
    task: Option<T>,
    interval: Duration,
    s: Option<ac::Sender<u8>>,
    r: Option<ac::Receiver<u8>>,
    thread: Option<JoinHandle<()>>,
}

impl<T> Runner<T> {
    pub fn new(interval: Duration) -> Runner<T> {
        Runner {
            interval,
            task: None,
            s: None,
            r: None,
            thread: None,
        }
    }

    fn take_task(&mut self) -> Result<T, &'static str> {
        self.task.take().ok_or("task not set")
    }

    fn _start_runner(&mut self, task: Task) {
        let (sub_sender, main_receiver) = ac::bounded::<u8>(1);
        let (main_sender, sub_receiver) = ac::bounded::<u8>(1);

        self.s = Some(main_sender);
        self.r = Some(main_receiver);

        let interval = self.interval;
        let mut task = task;
        self.thread = Some(thread::spawn(move || {
            sub_sender.send_blocking(0).unwrap();
            loop {
                if sub_receiver.try_recv().is_ok() {
                    break;
                }

                let frame_start = Instant::now();

                task();

                if let Some(gap) = interval.checked_sub(frame_start.elapsed()) {
                    spin_sleep::sleep(gap);
                }
            }

            if sub_sender.send_blocking(0).is_err() {
                panic!("[task-controler] Error sending stopped signal");
            }
        }));
    }

    pub fn get_thread_ref(&self) -> &Thread {
        self.thread.as_ref().unwrap().thread()
    }
}

impl ExternalRunnerExt for Runner<Task> {
    fn set_task(&mut self, t: Task) {
        self.task = Some(t);
    }

    fn start(&mut self) -> Result<(), &str> {
        let task = self.take_task().unwrap();

        let (sub_sender, main_receiver) = ac::bounded::<u8>(1);
        let (main_sender, sub_receiver) = ac::bounded::<u8>(1);

        self.s = Some(main_sender);
        self.r = Some(main_receiver);

        let interval = self.interval;
        let mut task = task;
        self.thread = Some(thread::spawn(move || {
            sub_sender.send_blocking(0).unwrap();
            loop {
                if sub_receiver.try_recv().is_ok() {
                    break;
                }

                let frame_start = Instant::now();

                task();

                if let Some(gap) = interval.checked_sub(frame_start.elapsed()) {
                    spin_sleep::sleep(gap);
                }
            }

            if sub_sender.send_blocking(0).is_err() {
                panic!("[task-controler] Error sending stopped signal");
            }
        }));
        Ok(())
    }
    /// !!! DO NOT USE THIS IF `true` WILL BE RETURNED FROM `Task`, INSTEAD, USE `join()`.
    /// Send `signal` to runner, wait for response and join thread.
    /// Gets dropped after call
    fn close(mut self) -> Result<(), &'static str> {
        if let Some(t) = self.thread.take() {
            self.s.as_ref().unwrap().send_blocking(0).unwrap();
            self.r.as_ref().unwrap().recv_blocking().unwrap();
            t.join().unwrap();
            return Ok(());
        };
        Err("no task running")
    }
}

impl InternalRunnerExt for Runner<TaskWithHandle> {
    fn start(&mut self) -> Result<(), &'static str> {
        let task = self.take_task().unwrap();

        let (sub_sender, main_receiver) = ac::bounded::<u8>(1);

        self.s = None;
        self.r = Some(main_receiver);

        let interval = self.interval;
        let mut task = task;
        self.thread = Some(thread::spawn(move || {
            sub_sender.send_blocking(0).unwrap();
            loop {
                let frame_start = Instant::now();

                if task() {
                    return;
                };

                if let Some(gap) = interval.checked_sub(frame_start.elapsed()) {
                    spin_sleep::sleep(gap);
                }
            }
        }));
        Ok(())
    }

    /// object moved and gets dropped after call
    fn join(mut self) -> Result<(), &'static str> {
        if let Some(t) = self.thread.take() {
            t.join().unwrap();
            return Ok(());
        };
        Err("no task running")
    }

    fn set_task(&mut self, t: TaskWithHandle) {
        self.task = Some(t);
    }
}

pub fn new_external_close_runner(interval: Duration) -> Runner<Task> {
    Runner::new(interval)
}

pub fn new_internal_close_runner(interval: Duration) -> Runner<TaskWithHandle> {
    Runner::new(interval)
}
