#include "ffi.h"
#include "async_notifier.h"

namespace FFI {
void ffi_gc_raw_cpp_ptr(RawVoidPtr ptr, RawCppPtrType type) {
    if (ptr)
    {
        switch (static_cast<RawCppPtrTypeImpl>(type))
        {
        case RawCppPtrTypeImpl::String:
            delete reinterpret_cast<RawCppStringPtr>(ptr);
            break;
        case RawCppPtrTypeImpl::WakerNotifier:
            delete reinterpret_cast<AsyncNotifier *>(ptr);
            break;
        default:
            exit(-1);
        }
    }
}

void ffi_gc_raw_cpp_ptr_carr(RawVoidPtr ptr, RawCppPtrType type, uint64_t len) {
    if (ptr)
    {
        switch (static_cast<RawCppPtrTypeImpl>(type))
        {
        default:
            exit(-1);
        }
    }
}

void ffi_gc_special_raw_cpp_ptr(RawVoidPtr ptr, uint64_t hint_size, SpecialCppPtrType type) {
    if (ptr)
    {
        switch (static_cast<SpecialCppPtrType>(type))
        {
        case SpecialCppPtrType::None:
            // Do nothing.
            break;
        case SpecialCppPtrType::TupleOfRawCppPtr:
        {
            auto * special_ptr = reinterpret_cast<RawCppPtrTuple *>(ptr);
            delete special_ptr->inner;
            delete special_ptr;
            break;
        }
        case SpecialCppPtrType::ArrayOfRawCppPtr:
        {
            auto * special_ptr = reinterpret_cast<RawCppPtrArr *>(ptr);
            delete special_ptr->inner;
            delete special_ptr;
            break;
        }
        default:
            exit(-1);
        }
    }
}

RawCppPtr ffi_raw_cpp_ptr(RawVoidPtr ptr_, RawCppPtrTypeImpl type_)
{
    return RawCppPtr{ptr_, static_cast<RawCppPtrType>(type_)};
}

RawCppPtr ffi_gen_cpp_string(BaseBuffView view)
{
    return ffi_raw_cpp_ptr(view.len ? RawCppString::New(view.data, view.len) : nullptr, RawCppPtrTypeImpl::String);
}

// This interface is provided, so we can call from rust part.
RustHandle * init_global_rust_handle() {
    return new RustHandle {
        .c_ctx = nullptr,
        .fn_gen_cpp_string = ffi_gen_cpp_string,
        .fn_gc_raw_cpp_ptr = ffi_gc_raw_cpp_ptr,
        .fn_gc_raw_cpp_ptr_carr = ffi_gc_raw_cpp_ptr_carr,
        .fn_gc_special_raw_cpp_ptr = ffi_gc_special_raw_cpp_ptr
    };
}

CHandle * global_c_handle = nullptr;

CHandle * get_global_c_handle() {
    return global_c_handle;
}

}