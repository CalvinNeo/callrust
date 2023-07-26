#![feature(vec_into_raw_parts)]

mod ffi;
mod interfaces;
mod mock_cpeer;
#[cfg(test)]
mod tests;
mod util;
mod waker;

use std::{future::Future, sync::Arc};

use ffi::*;
use futures_util::future::BoxFuture;
use interfaces::root::FFI::*;
use waker::*;

lazy_static::lazy_static! {
    static ref GLOBAL_RUNTIME : tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
}

pub struct TimerTask {
    future: BoxFuture<'static, ()>,
}

pub extern "C" fn ffi_make_timer_task(millis: u64) -> RustFuture {
    let _g = GLOBAL_RUNTIME.enter();
    let fut = async move { tokio::time::sleep(std::time::Duration::from_millis(millis)).await };
    let task = TimerTask {
        future: Box::pin(fut),
    };
    RustFuture {
        ptr: Box::into_raw(Box::new(task)) as *mut _,
        type_: RawRustPtrTypeEnum::TimerTask.into(),
    }
}

#[allow(clippy::bool_to_int_with_if)]
pub unsafe extern "C" fn ffi_poll_timer_task(task_ptr: RawVoidPtr, waker: RawVoidPtr) -> u8 {
    let _g = GLOBAL_RUNTIME.enter();
    let task = &mut *(task_ptr as *mut TimerTask);
    let mut func = |cx: &mut std::task::Context| {
        let fut = &mut task.future;
        match fut.as_mut().poll(cx) {
            std::task::Poll::Pending => None,
            std::task::Poll::Ready(e) => Some(e),
        }
    };
    let waker = if waker.is_null() {
        None
    } else {
        Some(&*(waker as *mut ArcNotifyWaker))
    };
    let res = if let Some(waker) = waker {
        let waker = futures::task::waker_ref(waker);
        let cx = &mut std::task::Context::from_waker(&*waker);
        func(cx)
    } else {
        let waker = futures::task::noop_waker();
        let cx = &mut std::task::Context::from_waker(&waker);
        func(cx)
    };
    if res.is_some() { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn ffi_println(buff: BaseBuffView) {
    unsafe {
        println!("{}", std::str::from_utf8_unchecked(buff.to_slice()));
    }
}

#[no_mangle]
pub extern "C" fn ffi_invoke_test() {
    unsafe {
        let jh = std::thread::spawn(||{

        });
    }
}

pub fn init_global_c_handle() -> *const CHandle {
    Box::into_raw(Box::new(CHandle {
        rust_ctx: std::ptr::null(),
        fn_gc_rust_ptr: Some(ffi_gc_rust_ptr),
        fn_println: Some(ffi_println),
        fn_make_async_waker: Some(ffi_make_async_waker),
        fn_invoke_test: Some(ffi_invoke_test),
    }))
}

// TODO Avoid Undefined symbols here.
// extern "C" {
//     #[no_mangle]
//     fn set_global_c_handle(ptr: RawVoidPtr);
// }

#[no_mangle]
pub extern "C" fn set_global_rust_handle(ptr: RawVoidPtr) -> RawVoidPtr {
    init_global_rust_handle(ptr as *const u8);
    let c_handle = init_global_c_handle();
    c_handle as RawVoidPtr
}
