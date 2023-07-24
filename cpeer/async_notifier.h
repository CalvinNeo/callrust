#pragma once

#include <chrono>
#include <memory>
#include <atomic>
#include <mutex>
#include <condition_variable>
#include "ffi.h"

static constexpr size_t CPU_CACHE_LINE_SIZE = 64;
template <typename Base, size_t alignment>
struct AlignedStruct
{
    template <typename... Args>
    explicit AlignedStruct(Args &&... args)
        : inner{std::forward<Args>(args)...}
    {}

    Base & base() { return inner; }
    const Base & base() const { return inner; }
    Base * operator->() { return &inner; }
    const Base * operator->() const { return &inner; }
    Base & operator*() { return inner; }
    const Base & operator*() const { return inner; }

private:
    // Wrapped with struct to guarantee that it is aligned to `alignment`
    // DO NOT need padding byte
    alignas(alignment) Base inner;
};

// An interface for c-rust ffi
struct AsyncNotifier
{
    enum class Status
    {
        Timeout,
        Normal,
    };
    virtual Status blockedWaitFor(const std::chrono::milliseconds & duration)
    {
        return blockedWaitUtil(std::chrono::system_clock::now() + duration);
    }
    virtual Status blockedWaitUtil(const std::chrono::time_point &) = 0;
    virtual void wake() = 0;
    virtual ~AsyncNotifier() = default;
};

struct Notifier final : AsyncNotifier
{
    // Usually sender invoke `wake`, receiver invoke `blockedWaitUtil`
    // NOT thread safe
    Status blockedWaitUtil(const std::chrono::time_point & time_point) override {
        // if flag from false to false, wait for notification.
        // if flag from true to false, do nothing.
        auto res = AsyncNotifier::Status::Normal;
        if (!is_awake->exchange(false, std::memory_order_acq_rel))
        {
            {
                auto lock = std::unique_lock<std::mutex>(mutex);
                if (!is_awake->load(std::memory_order_acquire))
                {
                    if (cv.wait_until(lock, time_point) == std::cv_status::timeout)
                        res = AsyncNotifier::Status::Timeout;
                }
            }
            is_awake->store(false, std::memory_order_release);
        }
        return res;
    }

    // Thread safe
    void wake() override {
        // if flag from false -> true, then wake up.
        // if flag from true -> true, do nothing.
        if (is_awake->load(std::memory_order_acquire))
            return;
        if (!is_awake->exchange(true, std::memory_order_acq_rel))
        {
            // wake up notifier
            auto lock = std::scoped_lock<std::mutex>(mutex);
            cv.notify_one();
        }
    }

    ~Notifier() override = default;

private:
    // multi notifiers single receiver model. use another flag to avoid waiting endlessly.
    AlignedStruct<std::atomic_bool, CPU_CACHE_LINE_SIZE> is_awake{false};
    mutable AlignedStruct<std::condition_variable, CPU_CACHE_LINE_SIZE> cv;
    mutable AlignedStruct<std::mutex, CPU_CACHE_LINE_SIZE> mutex;
};

struct AsyncWaker
{
    using NotifierPtr = std::shared_ptr<Notifier>;

    // proxy will call this function to invoke `AsyncNotifier::wake`
    static void wake(RawVoidPtr notifier_ptr) {
        auto & notifier = *reinterpret_cast<AsyncNotifier *>(notifier_ptr);
        notifier.wake();
    }

    // create a `Notifier` in heap & let proxy wrap it and return as rust ptr with specific type.
    explicit AsyncWaker(const FFIContext & ctx) AsyncWaker(ctx, new AsyncWaker::Notifier{}) {

    }

    AsyncWaker(const FFIContext & ctx, AsyncNotifier * notifier_ptr) : 
    inner(ctx.makeAsyncWaker(AsyncWaker::wake, GenRawCppPtr(notifier_, RawCppPtrTypeImpl::WakerNotifier))),
    notifier(*notifier_)
    {

    }

    AsyncNotifier::Status waitUtil(std::chrono::time_point) {
        return notifier.blockedWaitUtil(time_point);
    }

    RawVoidPtr getRaw() const {
        return inner.ptr;
    }

private:
    // Asyncwaker on Rust's side.
    // This waker can be used to construct a Context as argument for Future::poll.
    RawRustPtrWrap inner;
    // Always in heap and is maintained as shared obj.
    AsyncNotifier & notifier;
};