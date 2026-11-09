//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple task scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        // For stride
        let min_pass = self.ready_queue
            .iter()
            .map(|task| task.inner_exclusive_access().pass)
            .min()
            .unwrap_or(0);

        {
            let mut inner = task.inner_exclusive_access();
            if inner.pass < min_pass {
                inner.pass = min_pass;
            }
        }

        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // FIFO
        // self.ready_queue.pop_front()
        
        // Stride
        let index = self.ready_queue
            .iter()
            .enumerate()
            .min_by_key(|(_, task)| {
                task.inner_exclusive_access().pass
            })
            .map(|(index, _)| index)?;

        let task = self.ready_queue.remove(index)?;

        let overflow = {
            let inner = task.inner_exclusive_access();
            inner.pass.checked_add(inner.stride).is_none()
        };

        if overflow {
            self.normalize_with(&task);
        }

        {
            let mut inner = task.inner_exclusive_access();

            // This task consumes one scheduling time slice.
            inner.pass = inner.pass.checked_add(inner.stride)
                .expect("pass overflow after normalization");
        }

        Some(task)
    }

    /// Normaliza passes in case of overflow
    fn normalize_with(&mut self, selected: &Arc<TaskControlBlock>) {
        let selected_pass = selected.inner_exclusive_access().pass;

        let ready_min = self.ready_queue
            .iter()
            .map(|task| task.inner_exclusive_access().pass)
            .min()
            .unwrap_or(selected_pass);

        let base = if selected_pass < ready_min {
            selected_pass
        }
        else {
            ready_min
        };

        if base == 0 {
            return;
        }

        for task in self.ready_queue.iter() {
            let mut inner = task.inner_exclusive_access();
            inner.pass -= base;
        }
        
        let mut inner = selected.inner_exclusive_access();
        inner.pass -= base;
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
