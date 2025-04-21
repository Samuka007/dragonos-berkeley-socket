// To make compatible
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pid(usize);

pub struct ProcessManager {}

impl ProcessManager {
    pub fn current_pid() -> Pid {
        Pid(std::thread::current().id().as_u64().get() as usize)
    }
}
