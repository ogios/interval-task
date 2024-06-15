#[allow(unused, unused_mut, dead_code)]
mod s {
    use std::{
        borrow::BorrowMut,
        clone::Clone,
        sync::{Arc, Mutex, RwLock},
    };

    pub struct StateManager<F> {
        state: Arc<Mutex<Arc<F>>>,
        pub in_change: Arc<RwLock<bool>>,
    }
    impl<F: 'static> StateManager<F> {
        pub fn new(initial_state: F) -> Self {
            StateManager {
                state: Arc::new(Mutex::new(Arc::new(initial_state))),
                in_change: Arc::new(RwLock::new(false)),
            }
        }

        fn _set(&mut self, new_state: Arc<F>) -> StateChangeHandler<F> {
            *self.in_change.write().unwrap() = true;
            let state = self.state.clone();
            let in_change = self.in_change.clone();
            StateChangeHandler {
                replace_state: Some(new_state),
                state,
                in_change,
            }
        }

        pub fn set(&mut self, new_state: F) -> Result<Box<dyn SetAfter<F> + '_>, &'static str> {
            if self.is_in_change() {
                return Err("state is under change");
            }
            Ok(Box::new(self._set(Arc::new(new_state))))
        }

        pub fn is_in_change(&self) -> bool {
            self.in_change.read().unwrap().to_owned()
        }

        pub fn set_first(&mut self, new_state: F) -> Result<Box<dyn SetFirst<F>>, &'static str> {
            if self.is_in_change() {
                return Err("state is under change");
            }
            let old_state = self.state.lock().unwrap().clone();
            self.state
                .lock()
                .unwrap()
                .borrow_mut()
                .clone_from(&Arc::new(new_state));
            Ok(Box::new(self._set(old_state)))
        }

        pub fn get(&self) -> Arc<F> {
            let a = self.state.lock().unwrap();
            // let b = a.clone_into(c);
            let b = a.clone();
            drop(a);
            b
        }
    }

    pub struct StateChangeHandler<F> {
        replace_state: Option<Arc<F>>,
        state: Arc<Mutex<Arc<F>>>,
        in_change: Arc<RwLock<bool>>,
    }
    pub trait SetAfter<F> {
        fn ignore(&mut self);
        fn apply(&mut self);
        fn changed_to(&self, new_state: F);
    }
    pub trait SetFirst<F> {
        fn restore(&mut self);
        fn apply(&mut self);
        fn changed_to(&self, new_state: F);
    }
    impl<F> StateChangeHandler<F> {
        fn dispose(&self) {
            *self.in_change.write().unwrap() = false;
        }
    }
    impl<F> SetAfter<F> for StateChangeHandler<F> {
        fn ignore(&mut self) {
            self.dispose()
        }
        fn apply(&mut self) {
            if let Some(f) = self.replace_state.take() {
                self.state.lock().unwrap().borrow_mut().clone_from(&f);
            }
            self.dispose()
        }
        fn changed_to(&self, new_state: F) {
            self.state
                .lock()
                .unwrap()
                .borrow_mut()
                .clone_from(&Arc::new(new_state));
            self.dispose()
        }
    }
    impl<F> SetFirst<F> for StateChangeHandler<F> {
        fn restore(&mut self) {
            if let Some(f) = self.replace_state.take() {
                self.state.lock().unwrap().borrow_mut().clone_from(&f);
            }
            self.dispose()
        }
        fn apply(&mut self) {
            self.dispose()
        }
        fn changed_to(&self, new_state: F) {
            self.state
                .lock()
                .unwrap()
                .borrow_mut()
                .clone_from(&Arc::new(new_state));
            self.dispose()
        }
    }
}

pub use s::*;
