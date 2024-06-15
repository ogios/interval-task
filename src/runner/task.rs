pub trait TaskExt<T> {
    fn get_call<'a>(&'a mut self) -> Box<dyn FnMut() -> T + Send + Sync + 'a>;
}
pub enum Task {
    TaskFn(Box<dyn FnTask + Send + Sync>),
    TaskFnMut(Box<dyn FnMutTask + Send + Sync>),
}
unsafe impl Send for Task {}
unsafe impl Sync for Task {}
pub trait FnTask {
    fn call(&self);
}
pub trait FnMutTask {
    fn call_mut(&mut self);
}
impl TaskExt<()> for Task {
    fn get_call<'a>(&'a mut self) -> Box<dyn 'a + FnMut() + Send + Sync> {
        match self {
            Task::TaskFn(t) => Box::new(|| t.call()),
            Task::TaskFnMut(t) => Box::new(|| t.call_mut()),
        }
    }
}

impl Task {
    pub fn new_fn_task(t: impl FnTask + Send + Sync + 'static) -> Task {
        Task::TaskFn(Box::new(t))
    }
    pub fn new_fn_mut_task(t: impl FnMutTask + Send + Sync + 'static) -> Task {
        Task::TaskFnMut(Box::new(t))
    }
}

pub enum TaskWithHandle {
    TaskFnHanle(Box<dyn FnTaskWithHandle + Send + Sync>),
    TaskFnMutHanle(Box<dyn FnMutTaskWithHandle + Send + Sync>),
}
unsafe impl Send for TaskWithHandle {}
unsafe impl Sync for TaskWithHandle {}
pub trait FnTaskWithHandle {
    fn call(&self) -> bool;
}
pub trait FnMutTaskWithHandle {
    fn call_mut(&mut self) -> bool;
}
impl TaskExt<bool> for TaskWithHandle {
    fn get_call<'a>(&'a mut self) -> std::boxed::Box<(dyn FnMut() -> bool + Send + Sync + 'a)> {
        match self {
            TaskWithHandle::TaskFnHanle(t) => Box::new(|| t.call()),
            TaskWithHandle::TaskFnMutHanle(t) => Box::new(|| t.call_mut()),
        }
    }
}
impl TaskWithHandle {
    pub fn new_fn_task(t: impl FnTaskWithHandle + Send + Sync + 'static) -> TaskWithHandle {
        TaskWithHandle::TaskFnHanle(Box::new(t))
    }
    pub fn new_fn_mut_task(t: impl FnMutTaskWithHandle + Send + Sync + 'static) -> TaskWithHandle {
        TaskWithHandle::TaskFnMutHanle(Box::new(t))
    }
}
