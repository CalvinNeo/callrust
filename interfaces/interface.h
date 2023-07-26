#pragma once

#include <cstdint>

// TODO Consider add an extra namespace field.

// We need this FFI to filter outside mod std.
namespace FFI {

using ConstRawVoidPtr = const void *;
using RawVoidPtr = void *;

struct RawCppString;
using RawCppStringPtr = RawCppString *;

struct BaseBuffView {
    const char *data;
    const uint64_t len;
};

using RawCppPtrType = uint32_t;
using RawRustPtrType = uint32_t;

struct RawRustPtr {
    RawVoidPtr ptr;
    RawRustPtrType type;
};

using RustFuture = RawRustPtr;

// struct RustFuture : RawRustPtr {

// };

enum class SpecialCppPtrType : uint32_t {
  None = 0,
  TupleOfRawCppPtr = 1,
  ArrayOfRawCppPtr = 2,
};

struct RawCppPtr {
    RawVoidPtr ptr;
    RawCppPtrType type;
};

struct RawCppPtrCarr {
    RawVoidPtr inner;
    const uint64_t len;
    RawCppPtrType type;
};

// An tuple of pointers, like `void **`,
// Can be used to represent structures.
struct RawCppPtrTuple {
    RawCppPtr *inner;
    const uint64_t len;
};

// An array of pointers(same type), like `T **`,
// Can be used to represent arrays.
struct RawCppPtrArr {
    RawVoidPtr *inner;
    const uint64_t len;
    RawCppPtrType type;
};

struct CHandle {
    ConstRawVoidPtr rust_ctx;

    void (*fn_gc_rust_ptr)(RawVoidPtr, RawRustPtrType);
    void (*fn_println)(BaseBuffView);
    RawRustPtr (*fn_make_async_waker)(void (*wake_fn)(RawVoidPtr),
                                      RawCppPtr data);
    void (*fn_invoke_test)();
};

struct RustHandle {
    ConstRawVoidPtr c_ctx;

    RawCppPtr (*fn_gen_cpp_string)(BaseBuffView);
    void (*fn_gc_raw_cpp_ptr)(RawVoidPtr, RawCppPtrType);
    void (*fn_gc_raw_cpp_ptr_carr)(RawVoidPtr, RawCppPtrType, uint64_t);
    void (*fn_gc_special_raw_cpp_ptr)(RawVoidPtr, uint64_t, SpecialCppPtrType);
};

extern "C" {
}
}
