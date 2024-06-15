use async_channel as ac;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::state_manager::StateManager;

type ResultOrString<T> = Result<T, &'static str>;
type PrivateReturnCallBack = ResultOrString<Box<dyn Send + FnOnce() -> ResultOrString<()>>>;
type JoinHandleResult<T> = ResultOrString<JoinHandle<T>>;

pub const START_SIGNAL: u8 = 0;
pub const STOP_SIGNAL: u8 = 1;

enum RunnerState {
    Active,
    Inactive,
    Operating,
    Closed,
}

pub struct RunnerHandler {
    pub thread: JoinHandle<()>,
    s: ac::Sender<u8>,
    r: ac::Receiver<u8>,
    // state: Arc<RwLock<RunnerState>>,
    _state: Arc<Mutex<StateManager<RunnerState>>>,
}

impl RunnerHandler {
    pub fn new(s: ac::Sender<u8>, r: ac::Receiver<u8>, thread: JoinHandle<()>) -> Self {
        RunnerHandler {
            s,
            r,
            thread,
            // state: Arc::new(RwLock::new(RunnerState::Inactive)),
            _state: Arc::new(Mutex::new(StateManager::new(RunnerState::Inactive))),
        }
    }

    // start
    fn _start(&mut self) -> PrivateReturnCallBack {
        let mut state = self._state.lock().unwrap();
        let a = state.get();
        match a.deref() {
            RunnerState::Active => Err("[task-controler] Runner already running"),
            RunnerState::Inactive => {
                drop(a);
                let mut h = state.set_first(RunnerState::Operating)?;
                if self.s.send_blocking(START_SIGNAL).is_err() {
                    h.restore();
                    return Err("[task-controler] Error sending start signal");
                }
                h.apply();
                let state = self._state.clone();
                let r = self.r.clone();
                Ok(Box::new(move || {
                    let mut state = state.lock().unwrap();
                    let x = match state.set(RunnerState::Active) {
                        Ok(mut h) => {
                            if r.recv_blocking().is_err() {
                                h.changed_to(RunnerState::Inactive);
                                return Err("[task-controler] Error receiving started signal");
                            }
                            h.apply();
                            Ok(())
                        }
                        Err(e) => Err(e),
                    };
                    x
                }))
            }
            RunnerState::Operating => Err("[task-controler] Runner is starting"),
            RunnerState::Closed => Err(
                "[task-controler] Runner is closed, you should drop this and create another one",
            ),
        }
    }
    pub fn start(&mut self) -> JoinHandleResult<ResultOrString<()>> {
        let res = self._start()?;
        Ok(thread::spawn(res))
    }
    pub fn start_blocking(&mut self) -> Result<(), &str> {
        let res = self._start()?;
        res()
    }

    // close
    fn _close(&self) -> PrivateReturnCallBack {
        let mut state = self._state.lock().unwrap();
        let a = state.get();
        match a.deref() {
            RunnerState::Operating => Err(
                "[task-controler] Runner is starting or closing, please wait using `JoinHandle` then perform operation",
            ),
            RunnerState::Closed => Err("[task-controler] Runner is closed, you should drop this"),
            _ => {
                drop(a);
                match state.set(RunnerState::Closed) {
                    Ok(mut h) => {
                        if self.s.send_blocking(STOP_SIGNAL).is_err() {
                            h.ignore();
                            return Err("[task-controler] error sending stop signal");
                        }
                        h.apply();

                        let state = self._state.clone();
                        let r = self.r.clone();
                        Ok(Box::new(move || {
                            let mut state = state.lock().unwrap();
                            let x = match state.set(RunnerState::Closed) {
                                Ok(mut h) => {
                                    if r.recv_blocking().is_err() {
                                        h.ignore();
                                        return Err("[task-controler] Error receiving stopped signal");
                                    }
                                    h.apply();
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            };
                            x
                        }))
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
    pub fn close(&mut self) -> JoinHandleResult<ResultOrString<()>> {
        let res = self._close()?;
        Ok(thread::spawn(res))
    }
    pub fn close_blocking(&mut self) -> Result<(), &str> {
        let res = self._close()?;
        res()
    }
}
