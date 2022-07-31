//! Process management syscalls

use crate::config::MAX_SYSCALL_NUM;
use crate::mm::convert_to_physical_addr;
use crate::task::{
    current_user_token, exit_current_and_run_next, get_first_sched_time, list_syscall_counts, mmap,
    munmap, suspend_current_and_run_next, TaskStatus,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

impl From<TimeVal> for usize {
    fn from(tv: TimeVal) -> Self {
        tv.sec * 1_000_000 + tv.usec
    }
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    // 将虚拟地址转换为物理地址
    let token = current_user_token();
    let ts = convert_to_physical_addr(token, _ts as usize) as *mut TimeVal;
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    mmap(_start, _len, _port)
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    munmap(_start, _len)
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    let first_sched_ts = get_first_sched_time();
    let syscall_times = list_syscall_counts();
    let curr_ts = get_time_us();
    let token = current_user_token();
    let ti = convert_to_physical_addr(token, _ti as usize) as *mut TaskInfo;
    unsafe {
        *ti = TaskInfo {
            status: TaskStatus::Running,
            syscall_times: syscall_times,
            time: (usize::from(curr_ts) - usize::from(first_sched_ts)) / 1000,
        }
    }
    0
}
