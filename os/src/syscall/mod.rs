//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// sbrk syscall
const SYSCALL_SBRK: usize = 214;
/// munmap syscall
const SYSCALL_MUNMAP: usize = 215;
/// mmap syscall
const SYSCALL_MMAP: usize = 222;
/// trace syscall
const SYSCALL_TRACE: usize = 410;

mod fs;
mod process;

use crate::task::count_current_syscall;
use fs::*;
use process::*;

/// syscall count struct
#[derive(Clone, Copy)]
pub struct SyscallCount {
    /// write syscall count
    write: usize,
    /// exit syscall count
    exit: usize,
    /// yield syscall count
    yield_: usize,
    /// get_time syscall count
    get_time: usize,
    /// sbrk syscall count
    sbrk: usize,
    /// munmap syscall count
    munmap: usize,
    /// mmap syscall count
    mmap: usize,
    /// trace syscall count
    trace: usize,
}

impl SyscallCount {
    /// new
    pub fn new() -> Self {
        Self {
            write: 0,
            exit: 0,
            yield_: 0,
            get_time: 0,
            sbrk: 0,
            munmap: 0,
            mmap: 0,
            trace: 0,
        }
    }

    /// count
    pub fn count(&mut self, syscall_id: usize) {
        match syscall_id {
            SYSCALL_WRITE => self.write += 1,
            SYSCALL_EXIT => self.exit += 1,
            SYSCALL_YIELD => self.yield_ += 1,
            SYSCALL_GET_TIME => self.get_time += 1,
            SYSCALL_SBRK => self.sbrk += 1,
            SYSCALL_MUNMAP => self.munmap += 1,
            SYSCALL_MMAP => self.mmap += 1,
            SYSCALL_TRACE => self.trace += 1,
            _ => panic!("Unsupported syscall_id: {}", syscall_id),
        }
    }

    /// get
    pub fn get(&self, syscall_id: usize) -> Option<usize> {
        match syscall_id {
            SYSCALL_WRITE => Some(self.write),
            SYSCALL_EXIT => Some(self.exit),
            SYSCALL_YIELD => Some(self.yield_),
            SYSCALL_GET_TIME => Some(self.get_time),
            SYSCALL_SBRK => Some(self.sbrk),
            SYSCALL_MUNMAP => Some(self.munmap),
            SYSCALL_MMAP => Some(self.mmap),
            SYSCALL_TRACE => Some(self.trace),
            _ => None,
        }
    }
}

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    count_current_syscall(syscall_id);
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TRACE => sys_trace(args[0], args[1], args[2]),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_SBRK => sys_sbrk(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
