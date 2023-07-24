use std::{sync::Mutex, time::Duration};

use int_enum::IntEnum;

use crate::{ffi::*, ffi_make_async_waker, interfaces::root::FFI::*};

static START: std::sync::Once = std::sync::Once::new();
pub fn init_mock_global_ffi_context() {
    START.call_once(|| {
        let res = Box::new(RustHandle {
            c_ctx: std::ptr::null(),
            fn_gen_cpp_string: Some(ffi_gen_cpp_string),
            fn_gc_raw_cpp_ptr: Some(ffi_gc_raw_cpp_ptr),
            fn_gc_raw_cpp_ptr_carr: Some(ffi_gc_raw_cpp_ptr_carr),
            fn_gc_special_raw_cpp_ptr: Some(ffi_gc_special_raw_cpp_ptr),
        });
        let raw = Box::into_raw(res);
        init_global_ffi_context(raw as *const RustHandle as *const u8)
    });
}

pub extern "C" fn ffi_gen_cpp_string(s: BaseBuffView) -> RawCppPtr {
    let str = Box::new(Vec::from(s.to_slice()));
    let ptr = Box::into_raw(str);
    RawCppPtr {
        ptr: ptr as *mut _,
        type_: RawCppPtrTypeImpl::String.into(),
    }
}

extern "C" fn ffi_gc_special_raw_cpp_ptr(ptr: RawVoidPtr, hint_len: u64, tp: SpecialCppPtrType) {
    match tp {
        SpecialCppPtrType::None => (),
        SpecialCppPtrType::TupleOfRawCppPtr => unsafe {
            let p = Box::from_raw(std::slice::from_raw_parts_mut(
                ptr as *mut RawCppPtr,
                hint_len as usize,
            ));
            drop(p);
        },
        SpecialCppPtrType::ArrayOfRawCppPtr => unsafe {
            let p = Box::from_raw(std::slice::from_raw_parts_mut(
                ptr as *mut RawVoidPtr,
                hint_len as usize,
            ));
            drop(p);
        },
    }
}

extern "C" fn ffi_gc_raw_cpp_ptr_carr(ptr: RawVoidPtr, tp: RawCppPtrType, len: u64) {
    match tp.into() {
        RawCppPtrTypeImpl::String => unsafe {
            let p = Box::from_raw(std::slice::from_raw_parts_mut(
                ptr as *mut RawVoidPtr,
                len as usize,
            ));
            for i in 0..len {
                let i = i as usize;
                if !p[i].is_null() {
                    ffi_gc_raw_cpp_ptr(p[i], RawCppPtrTypeImpl::String.into());
                }
            }
            drop(p);
        },
        _ => todo!(),
    }
}

pub extern "C" fn ffi_gc_raw_cpp_ptr(ptr: RawVoidPtr, tp: RawCppPtrType) {
    match tp.into() {
        RawCppPtrTypeImpl::None => {}
        RawCppPtrTypeImpl::String => unsafe {
            drop(Box::<Vec<u8>>::from_raw(ptr as *mut _));
        },
        RawCppPtrTypeImpl::WakerNotifier => unsafe {
            drop(Box::from_raw(ptr as *mut MockCPeerNotifier));
        },
        _ => todo!(),
    }
}

#[repr(u32)]
#[derive(int_enum::IntEnum, Copy, Clone)]
pub enum RawCppPtrTypeImpl {
    None = 0,
    String = 1,
    WakerNotifier = 12,
}

impl From<RawCppPtrTypeImpl> for RawCppPtrType {
    fn from(value: RawCppPtrTypeImpl) -> Self {
        assert_type_eq::assert_type_eq!(RawCppPtrType, u32);
        value.int_value()
    }
}

impl From<RawCppPtrType> for RawCppPtrTypeImpl {
    fn from(value: RawCppPtrType) -> Self {
        if let Ok(s) = RawCppPtrTypeImpl::from_int(value) {
            s
        } else {
            panic!("unknown RawCppPtrType {:?}", value);
        }
    }
}

pub struct MockCPeerNotifier {
    cv: std::sync::Condvar,
    mutex: Mutex<()>,
    // Multi notifiers single receiver model.
    // Use another flag to avoid waiting until timeout.
    flag: std::sync::atomic::AtomicBool,
}

impl MockCPeerNotifier {
    pub fn blocked_wait_for(&self, timeout: Duration) {
        // if flag from false -> false, wait for notification.
        // if flag from true -> false, do nothing.
        if !self.flag.swap(false, std::sync::atomic::Ordering::AcqRel) {
            {
                let lock = self.mutex.lock().unwrap();
                if !self.flag.load(std::sync::atomic::Ordering::Acquire) {
                    let _cv = self.cv.wait_timeout(lock, timeout);
                }
            }
            self.flag.store(false, std::sync::atomic::Ordering::Release);
        }
    }

    pub fn wake(&self) {
        // if flag from false -> true, then wake up.
        // if flag from true -> true, do nothing.
        if !self.flag.swap(true, std::sync::atomic::Ordering::AcqRel) {
            let _lock = self.mutex.lock().unwrap();
            self.cv.notify_one();
        }
    }

    pub fn new_raw() -> RawCppPtr {
        let notifier = Box::new(Self {
            cv: Default::default(),
            mutex: Mutex::new(()),
            flag: std::sync::atomic::AtomicBool::new(false),
        });

        RawCppPtr {
            ptr: Box::into_raw(notifier) as _,
            type_: RawCppPtrTypeImpl::WakerNotifier.into(),
        }
    }
}

pub struct RawRustPtrWrap(RawRustPtr);

impl RawRustPtrWrap {
    fn new(ptr: RawRustPtr) -> Self {
        Self(ptr)
    }
}

impl Drop for RawRustPtrWrap {
    fn drop(&mut self) {
        ffi_gc_rust_ptr(self.0.ptr, self.0.type_);
    }
}

// Wrap `MockCPeerNotifier` into something that can be made into a NotifyWaker by `ffi_make_async_waker`.
pub struct Waker {
    // The wrapped MockCPeerNotifier by `ffi_make_async_waker`.
    pub rust_waker: RawRustPtrWrap,
    // The MockCPeerNotifier itself.
    pub c_ptr: RawVoidPtr,
}

impl Waker {
    pub fn new() -> Self {
        let notifier_cpp = MockCPeerNotifier::new_raw();
        let ptr = notifier_cpp.ptr;
        let notifier_rust = ffi_make_async_waker(Some(ffi_wake), notifier_cpp);
        Self {
            rust_waker: RawRustPtrWrap::new(notifier_rust),
            c_ptr: ptr,
        }
    }

    pub fn wait_for(&self, timeout: Duration) {
        // Block wait for test
        self.get_cpeer_notifier().blocked_wait_for(timeout)
    }

    pub fn get_cpeer_notifier(&self) -> &MockCPeerNotifier {
        unsafe { &*(self.c_ptr as *mut MockCPeerNotifier) }
    }

    pub fn get_raw_rust_waker(&self) -> RawVoidPtr {
        self.rust_waker.0.ptr
    }
}

pub extern "C" fn ffi_wake(data: RawVoidPtr) {
    let notifier = unsafe { &mut *(data as *mut MockCPeerNotifier) };
    notifier.wake()
}
