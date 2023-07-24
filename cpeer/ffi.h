#pragma once

#include "../interfaces/interfaces.h"

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


struct FFIContext : public CHandle {
    void gcRustPtr(RawVoidPtr ptr, RawRustPtrType type) const
    {
        fn_gc_rust_ptr(ptr, type);
    }
};

struct GlobalFFIContext : public FFIContext, Singleton<GlobalFFIContext> {

}

struct RawRustPtrWrap;

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
