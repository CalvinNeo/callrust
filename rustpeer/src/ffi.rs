use crate::interfaces::root::FFI::*;

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum RawRustPtrTypeEnum {
    None = 0,
    TimerTask = 1,
    ArcFutureWaker = 2,
}

impl From<u32> for RawRustPtrTypeEnum {
    fn from(x: u32) -> Self {
        unsafe { std::mem::transmute(x) }
    }
}

// TODO remove this warn.
#[allow(clippy::from_over_into)]
impl Into<u32> for RawRustPtrTypeEnum {
    fn into(self) -> u32 {
        unsafe { std::mem::transmute(self) }
    }
}

#[allow(clippy::wrong_self_convention)]
pub trait UnwrapExternCFunc<T> {
    unsafe fn into_inner(&self) -> &T;
}

impl<T> UnwrapExternCFunc<T> for std::option::Option<T> {
    unsafe fn into_inner(&self) -> &T {
        std::mem::transmute::<&Self, &T>(self)
    }
}

impl From<&[u8]> for BaseBuffView {
    fn from(s: &[u8]) -> Self {
        let ptr = s.as_ptr() as *const _;
        Self {
            data: ptr,
            len: s.len() as u64,
        }
    }
}

#[allow(clippy::clone_on_copy)]
impl Clone for BaseBuffView {
    fn clone(&self) -> BaseBuffView {
        BaseBuffView {
            data: self.data.clone(),
            len: self.len.clone(),
        }
    }
}

impl BaseBuffView {
    pub fn to_slice(&self) -> &[u8] {
        if self.len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data as *const _, self.len as usize) }
        }
    }
}

impl RawCppPtr {
    pub fn into_raw(mut self) -> RawVoidPtr {
        let ptr = self.ptr;
        self.ptr = std::ptr::null_mut();
        ptr
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

static mut GLOBAL_FFI_CONTEXT_PTR: isize = 0;

pub fn init_global_ffi_context(p: *const u8) {
    unsafe {
        let ptr = &GLOBAL_FFI_CONTEXT_PTR as *const _ as *mut _;
        *ptr = p;
    }
}

pub fn gen_global_ffi_context(ptr: isize) -> &'static RustHandle {
    debug_assert!(ptr != 0);
    unsafe { &(*(ptr as *const RustHandle)) }
}

pub fn get_global_ffi_context() -> &'static RustHandle {
    gen_global_ffi_context(unsafe { GLOBAL_FFI_CONTEXT_PTR })
}

#[macro_export]
macro_rules! call_by_ffi_name {
    ($func:expr, $($args:expr),*) => {
        // debug_assert!($ffi.$func.is_some());
        unsafe {
            ($func.into_inner())
            (
                $($args,)*
            )
        }
    };
}

impl Drop for RawCppPtr {
    fn drop(&mut self) {
        if !self.is_null() {
            let ctx = get_global_ffi_context();
            call_by_ffi_name!(ctx.fn_gc_raw_cpp_ptr, self.ptr, self.type_)
        }
    }
}

// Do not guarantee raw pointer could be accessed between threads safely
// unsafe impl Sync for RawCppPtr {}
unsafe impl Send for RawCppPtr {}

pub extern "C" fn ffi_gc_rust_ptr(data: RawVoidPtr, type_: RawRustPtrType) {
    if data.is_null() {
        return;
    }
    let type_: RawRustPtrTypeEnum = type_.into();
    match type_ {
        RawRustPtrTypeEnum::ArcFutureWaker => unsafe {
            drop(Box::from_raw(data as *mut crate::ArcNotifyWaker));
        },
        RawRustPtrTypeEnum::TimerTask => unsafe {
            drop(Box::from_raw(data as *mut crate::TimerTask));
        },
        _ => unreachable!(),
    }
}

impl RawCppPtrTuple {
    pub fn is_null(&self) -> bool {
        unsafe { (*self.inner).ptr.is_null() }
    }
}

unsafe impl Send for RawCppPtrTuple {}

impl Drop for RawCppPtrTuple {
    fn drop(&mut self) {
        // Note the layout is:
        // [0] RawCppPtr to T
        // [1] RawCppPtr to R
        // ...
        // [len-1] RawCppPtr to S
        unsafe {
            if !self.is_null() {
                let ctx = get_global_ffi_context();
                let len = self.len;
                // Delete all `void *`.
                for i in 0..len {
                    let i = i as usize;
                    let inner_i = self.inner.add(i);
                    // Will not fire even without the if in tests,
                    // since type must be 0 which is None.
                    if !inner_i.is_null() {
                        call_by_ffi_name!(ctx.fn_gc_raw_cpp_ptr, (*inner_i).ptr, (*inner_i).type_);
                        // We still set to nullptr, even though we will immediately delete it.
                        (*inner_i).ptr = std::ptr::null_mut();
                    }
                }
                // Delete `void **`.
                call_by_ffi_name!(
                    ctx.fn_gc_special_raw_cpp_ptr,
                    self.inner as RawVoidPtr,
                    self.len,
                    SpecialCppPtrType::TupleOfRawCppPtr
                );
                self.inner = std::ptr::null_mut();
                self.len = 0;
            }
        }
    }
}

impl RawCppPtrArr {
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }
}

unsafe impl Send for RawCppPtrArr {}

impl Drop for RawCppPtrArr {
    fn drop(&mut self) {
        // Note the layout is:
        // [0] RawVoidPtr to T
        // [1] RawVoidPtr
        // ...
        // [len-1] RawVoidPtr
        unsafe {
            if !self.is_null() {
                let ctx = get_global_ffi_context();
                let len = self.len;
                // Delete all `T *`
                for i in 0..len {
                    let i = i as usize;
                    let inner_i = self.inner.add(i);
                    // Will fire even without the if in tests, since type is not 0.
                    if !(*inner_i).is_null() {
                        call_by_ffi_name!(ctx.fn_gc_raw_cpp_ptr, *inner_i, self.type_);
                        // We still set to nullptr, even though we will immediately delete it.
                        *inner_i = std::ptr::null_mut();
                    }
                }
                // Delete `T **`
                call_by_ffi_name!(
                    ctx.fn_gc_special_raw_cpp_ptr,
                    self.inner as RawVoidPtr,
                    self.len,
                    SpecialCppPtrType::ArrayOfRawCppPtr
                );
                self.inner = std::ptr::null_mut();
                self.len = 0;
            }
        }
    }
}

impl Drop for RawCppPtrCarr {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            let ctx = get_global_ffi_context();
            call_by_ffi_name!(
                ctx.fn_gc_raw_cpp_ptr_carr,
                self.inner as RawVoidPtr,
                self.type_,
                self.len
            );
            self.inner = std::ptr::null_mut();
            self.len = 0;
        }
    }
}
