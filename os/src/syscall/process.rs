//! Process management syscalls
use crate::{mm::{translated_byte_buffer, PTEFlags, PageTable, VirtAddr}, task::{change_program_brk, current_user_token, exit_current_and_run_next, get_current_syscall_count, suspend_current_and_run_next}, timer::get_time_us};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    if ts.is_null() {
        return -1;
    }

    let buffers = translated_byte_buffer(
        current_user_token(), 
        ts as *const u8, 
        core::mem::size_of::<TimeVal>(),
    );
    
    let us = get_time_us();
    let value = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    let src = unsafe {
        core::slice::from_raw_parts(
            &value as *const TimeVal as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };

    let mut copied = 0;
    for buffer in buffers {
        let len = buffer.len();
        buffer.copy_from_slice(&src[copied..(copied+len)]);
        copied += len;
    }
    
    0
}

/// support function: write user byte with check
fn write_user_byte(token: usize, addr: usize, value: u8) -> bool {
    let va = VirtAddr::from(addr);
    let page_table = PageTable::from_token(token);

    let pte = match page_table.translate(va.floor()) {
        Some(pte) => pte,
        None => return false,
    };

    let flags =pte.flags();
    if !pte.is_valid()
        || !flags.contains(PTEFlags::U)
        || !flags.contains(PTEFlags::W) {
        return false;
    }

    let page = pte.ppn().get_bytes_array();
    page[va.page_offset()] = value;
    true
}

/// support function: read user byte with check
fn read_user_byte(token: usize, addr: usize) -> Option<u8> {
    let va = VirtAddr::from(addr);
    let page_table = PageTable::from_token(token);

    let pte = page_table.translate(va.floor())?;

    let flags = pte.flags();
    if !pte.is_valid()
        || !flags.contains(PTEFlags::U)
        || !flags.contains(PTEFlags::R) {
        return None;
    }

    let page = pte.ppn().get_bytes_array();
    Some(page[va.page_offset()])
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    
    match trace_request {
        0 => read_user_byte(current_user_token(), id)
            .map(|value| value as isize)
            .unwrap_or(-1),
        1 => if write_user_byte(current_user_token(), id, data as u8) { 0 } else { -1 },
        2 => get_current_syscall_count(id)
            .map(|count| count as isize)
            .unwrap_or(-1),
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
