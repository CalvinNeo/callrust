mod wrappers;

use std::time::Duration;

use tokio::runtime::Runtime;

use crate::{
    ffi_make_timer_task, ffi_poll_timer_task,
    mock_cpeer::{init_mock_global_rust_handle, Waker},
};

#[test]
fn test_timer_task() {
    crate::util::set_panic_hook();
    init_mock_global_rust_handle();
    let now = std::time::Instant::now();
    let task = ffi_make_timer_task(1000);

    let waker = Waker::new();
    assert_eq!(0, unsafe {
        ffi_poll_timer_task(task.ptr, waker.get_raw_rust_waker())
    });
    let now = std::time::Instant::now();
    waker.wait_for(Duration::from_secs(3));
    assert_ne!(0, unsafe {
        ffi_poll_timer_task(task.ptr, waker.get_raw_rust_waker())
    });
    assert!(now.elapsed() < Duration::from_secs(3));
}

#[test]
fn test_await_future() {
    crate::util::set_panic_hook();
    init_mock_global_rust_handle();
}
