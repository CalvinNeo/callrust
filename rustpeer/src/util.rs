pub fn set_panic_hook() {
    use std::{panic, thread};

    // HACK! New a backtrace ahead for caching necessary elf sections of this
    // tikv-server, in case it can not open more files during panicking
    // which leads to no stack info (0x5648bdfe4ff2 - <no info>).
    //
    // Crate backtrace caches debug info in a static variable `STATE`,
    // and the `STATE` lives forever once it has been created.
    // See more: https://github.com/alexcrichton/backtrace-rs/blob/\
    //           597ad44b131132f17ed76bf94ac489274dd16c7f/\
    //           src/symbolize/libbacktrace.rs#L126-L159
    // Caching is slow, spawn it in another thread to speed up.

    panic::set_hook(Box::new(move |info: &panic::PanicInfo<'_>| {
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let thread = thread::current();
        let name = thread.name().unwrap_or("<unnamed>");
        let loc = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()));
        let bt = std::backtrace::Backtrace::capture();

        println!(
            "{} thread_name {} location {} backtrace {}",
            msg,
            name,
            loc.unwrap_or_else(|| "<unknown>".to_owned()),
            format_args!("{}", bt),
        );

        // Calling process::exit would trigger global static to destroy, like C++
        // static variables of RocksDB, which may cause other threads encounter
        // pure virtual method call. So calling libc::_exit() instead to skip the
        // cleanup process.
        unsafe {
            libc::_exit(1);
        }
    }))
}

pub fn get_tag_from_thread_name() -> Option<String> {
    std::thread::current()
        .name()
        .and_then(|name| name.split("::").skip(1).last())
        .map(From::from)
}

/// Makes a thread name with an additional tag inherited from the current
/// thread.
#[macro_export]
macro_rules! thd_name {
    ($name:expr) => {{
        $crate::get_tag_from_thread_name()
            .map(|tag| format!("{}::{}", $name, tag))
            .unwrap_or_else(|| $name.to_owned())
    }};
}
