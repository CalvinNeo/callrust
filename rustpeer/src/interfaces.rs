#[allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
pub mod root {
    pub mod FFI {
        #[allow(unused_imports)]
        use self::super::super::root;
        pub type ConstRawVoidPtr = *const ::std::os::raw::c_void;
        pub type RawVoidPtr = *mut ::std::os::raw::c_void;
        #[repr(C)]
        #[derive(Debug)]
        pub struct RawCppString {
            _unused: [u8; 0],
        }
        pub type RawCppStringPtr = *mut root::FFI::RawCppString;
        #[repr(C)]
        #[derive(Debug)]
        pub struct BaseBuffView {
            pub data: *const ::std::os::raw::c_char,
            pub len: u64,
        }
        pub type RawCppPtrType = u32;
        pub type RawRustPtrType = u32;
        #[repr(C)]
        #[derive(Debug)]
        pub struct RawRustPtr {
            pub ptr: root::FFI::RawVoidPtr,
            pub type_: root::FFI::RawRustPtrType,
        }
        pub type RustFuture = root::FFI::RawRustPtr;
        #[repr(u32)]
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
        pub enum SpecialCppPtrType {
            None = 0,
            TupleOfRawCppPtr = 1,
            ArrayOfRawCppPtr = 2,
        }
        #[repr(C)]
        #[derive(Debug)]
        pub struct RawCppPtr {
            pub ptr: root::FFI::RawVoidPtr,
            pub type_: root::FFI::RawCppPtrType,
        }
        #[repr(C)]
        #[derive(Debug)]
        pub struct RawCppPtrCarr {
            pub inner: root::FFI::RawVoidPtr,
            pub len: u64,
            pub type_: root::FFI::RawCppPtrType,
        }
        #[repr(C)]
        #[derive(Debug)]
        pub struct RawCppPtrTuple {
            pub inner: *mut root::FFI::RawCppPtr,
            pub len: u64,
        }
        #[repr(C)]
        #[derive(Debug)]
        pub struct RawCppPtrArr {
            pub inner: *mut root::FFI::RawVoidPtr,
            pub len: u64,
            pub type_: root::FFI::RawCppPtrType,
        }
        #[repr(C)]
        #[derive(Debug)]
        pub struct CHandle {
            pub rust_ctx: root::FFI::ConstRawVoidPtr,
            pub fn_gc_rust_ptr: ::std::option::Option<
                unsafe extern "C" fn(arg1: root::FFI::RawVoidPtr, arg2: root::FFI::RawRustPtrType),
            >,
            pub fn_println:
                ::std::option::Option<unsafe extern "C" fn(arg1: root::FFI::BaseBuffView)>,
            pub fn_make_async_waker: ::std::option::Option<
                unsafe extern "C" fn(
                    wake_fn: ::std::option::Option<
                        unsafe extern "C" fn(arg1: root::FFI::RawVoidPtr),
                    >,
                    data: root::FFI::RawCppPtr,
                ) -> root::FFI::RawRustPtr,
            >,
            pub fn_invoke_test: ::std::option::Option<unsafe extern "C" fn()>,
        }
        #[repr(C)]
        #[derive(Debug)]
        pub struct RustHandle {
            pub c_ctx: root::FFI::ConstRawVoidPtr,
            pub fn_gen_cpp_string: ::std::option::Option<
                unsafe extern "C" fn(arg1: root::FFI::BaseBuffView) -> root::FFI::RawCppPtr,
            >,
            pub fn_gc_raw_cpp_ptr: ::std::option::Option<
                unsafe extern "C" fn(arg1: root::FFI::RawVoidPtr, arg2: root::FFI::RawCppPtrType),
            >,
            pub fn_gc_raw_cpp_ptr_carr: ::std::option::Option<
                unsafe extern "C" fn(
                    arg1: root::FFI::RawVoidPtr,
                    arg2: root::FFI::RawCppPtrType,
                    arg3: u64,
                ),
            >,
            pub fn_gc_special_raw_cpp_ptr: ::std::option::Option<
                unsafe extern "C" fn(
                    arg1: root::FFI::RawVoidPtr,
                    arg2: u64,
                    arg3: root::FFI::SpecialCppPtrType,
                ),
            >,
        }
    }
}
