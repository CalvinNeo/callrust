use std::sync::Arc;

use futures_util::future::BoxFuture;

use crate::{ffi::*, interfaces::root::FFI::*, GLOBAL_RUNTIME};

pub type ArcNotifyWaker = std::sync::Arc<NotifyWaker>;

pub struct NotifyWaker {
    pub inner: Box<dyn Fn() + Send + Sync>,
}

impl futures::task::ArcWake for NotifyWaker {
    fn wake_by_ref(arc_self: &std::sync::Arc<Self>) {
        (arc_self.inner)();
    }
}

// Given a C++ waker func and context, create a rust NotifyWaker.
#[allow(clippy::redundant_closure_call)]
pub extern "C" fn ffi_make_async_waker(
    wake_fn: Option<unsafe extern "C" fn(RawVoidPtr)>,
    data: RawCppPtr,
) -> RawRustPtr {
    let _g = GLOBAL_RUNTIME.enter();
    debug_assert!(wake_fn.is_some());

    struct RawCppPtrWrap(RawCppPtr);
    // This pointer should be thread safe, just wrap it.
    unsafe impl Sync for RawCppPtrWrap {}

    let data = RawCppPtrWrap(data);
    let res: ArcNotifyWaker = Arc::new(NotifyWaker {
        inner: Box::new(move || unsafe {
            let _ = &data;
            wake_fn.into_inner()(data.0.ptr);
        }),
    });
    RawRustPtr {
        ptr: Box::into_raw(Box::new(res)) as _,
        type_: RawRustPtrTypeEnum::ArcFutureWaker.into(),
    }
}
