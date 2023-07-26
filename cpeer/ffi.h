#pragma once
#include <cstdlib>
#include <string>
#include "../interfaces/interface.h"

#define DISALLOW_COPY(ClassName)           \
    ClassName(const ClassName &) = delete; \
    ClassName & operator=(const ClassName &) = delete

template <typename T>
class Singleton
{
public:
    static T & instance()
    {
        static T instance;
        return instance;
    }

protected:
    Singleton() = default;

public:
    Singleton(const Singleton &) = delete;
    Singleton & operator=(const Singleton &) = delete;
};


namespace FFI {

struct RawCppString : std::string
{
    using Base = std::string;
    using Base::Base;
    RawCppString() = delete;
    RawCppString(Base && src)
        : Base(std::move(src))
    {}
    RawCppString(const Base & src)
        : Base(src)
    {}
    DISALLOW_COPY(RawCppString);

    template <class... Args>
    static RawCppString * New(Args &&... _args)
    {
        return new RawCppString{std::forward<Args>(_args)...};
    }
};

struct RawRustPtrWrap;
struct GlobalFFIContext;

enum class RawCppPtrTypeImpl : RawCppPtrType
{
    None = 0,
    String,
    WakerNotifier,
};

void ffi_gc_raw_cpp_ptr(RawVoidPtr ptr, RawCppPtrType type);
void ffi_gc_raw_cpp_ptr_carr(RawVoidPtr ptr, RawCppPtrType type, uint64_t len);
void ffi_gc_special_raw_cpp_ptr(RawVoidPtr ptr, uint64_t hint_size, SpecialCppPtrType type);
RawCppPtr ffi_raw_cpp_ptr(RawVoidPtr ptr_, RawCppPtrTypeImpl type_);
RawCppPtr ffi_gen_cpp_string(BaseBuffView view);
RustHandle * init_global_rust_handle();

struct FFIContext : public CHandle {
    void gcRustPtr(RawVoidPtr ptr, RawRustPtrType type) const
    {
        fn_gc_rust_ptr(ptr, type);
    }
    RawRustPtr makeAsyncWaker(void (*wake_fn)(RawVoidPtr), RawCppPtr data) const {
        return fn_make_async_waker(wake_fn, data);
    }
};

struct GlobalFFIContext : public FFIContext, Singleton<GlobalFFIContext> {

};

struct RawRustPtrWrap : RawRustPtr
{
    DISALLOW_COPY(RawRustPtrWrap);

    explicit RawRustPtrWrap(RawRustPtr inner) : RawRustPtr(inner) {

    }
    ~RawRustPtrWrap() {
        if (ptr == nullptr)
            return;
        GlobalFFIContext::instance().gcRustPtr(ptr, type);
    }
    RawRustPtrWrap(RawRustPtrWrap &&) {
    }
};


CHandle * get_global_c_handle();
void set_global_c_handle(CHandle *);
}
