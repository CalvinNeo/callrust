#include "async_notifier.h"
#include "ffi.h"

extern "C" {
    void * set_global_rust_handle(void * ptr);
}

int main() {
    auto * rust_handle = FFI::init_global_rust_handle();
    auto * c_handle_raw = set_global_rust_handle(rust_handle);
    FFI::CHandle * c_handle = reinterpret_cast<FFI::CHandle *>(c_handle_raw);

    std::string s = "hello";
    c_handle->fn_println(FFI::BaseBuffView {
        .data = s.data(),
        .len = s.size()
    });

    delete rust_handle;
}