use sys::time::TimeSpec;
use std::mem::MaybeUninit;
use {Result, Errno};
use libc;

libc_enum! {
    #[cfg_attr(any(
                target_env = "uclibc",
                target_os = "fuchsia",
                target_os = "redox",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
                target_os = "haiku",
                all(not(target_env = "newlib"),
                    any(target_os = "linux",
                        target_os = "android",
                        target_os = "emscripten",
                        target_os = "solaris",
                        target_os = "illumos")),
               ), repr(i32))]
    #[cfg_attr(any(
            all(target_env = "newlib", target_arch = "arm"),
            target_os = "macos",
            target_os = "ios",
            ), repr(u32))]
    #[cfg_attr(any(
                all(target_env = "newlib", target_arch = "aarch64"),
                all(not(any(target_env = "newlib", target_env = "uclibc")), target_os = "hermit"),
                target_os = "dragonfly",
              ), repr(u64))]
    pub enum ClockId {
        #[cfg(any(target_os = "fuchsia",
                  all(not(any(target_env = "uclibc",
                              target_env = "newlib")),
                      any(target_os = "linux",
                          target_os = "android",
                          target_os = "emscripten"),
                      )
                  ))]
        CLOCK_BOOTTIME,
        #[cfg(any(target_os = "fuchsia",
                  all(not(any(target_env = "uclibc",
                              target_env = "newlib")),
                      any(target_os = "linux",
                          target_os = "android",
                          target_os = "emscripten"))))]
        CLOCK_BOOTTIME_ALARM,
        CLOCK_MONOTONIC,
        #[cfg(any(target_os = "fuchsia",
                  all(not(any(target_env = "uclibc",
                              target_env = "newlib")),
                      any(target_os = "linux",
                          target_os = "android",
                          target_os = "emscripten"))))]
        CLOCK_MONOTONIC_COARSE,
        #[cfg(any(target_os = "freebsd",
                  target_os = "dragonfly"))]
        CLOCK_MONOTONIC_FAST,
        #[cfg(any(target_os = "freebsd",
                  target_os = "dragonfly"))]
        CLOCK_MONOTONIC_PRECISE,
        #[cfg(any(target_os = "fuchsia",
                  all(not(any(target_env = "uclibc",
                              target_env = "newlib")),
                      any(target_os = "linux",
                          target_os = "android",
                          target_os = "emscripten"))))]
        CLOCK_MONOTONIC_RAW,
        #[cfg(any(target_os = "fuchsia",
                  target_env = "uclibc",
                  target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly",
                  all(not(target_env = "newlib"),
                      any(target_os = "linux",
                          target_os = "android",
                          target_os = "emscripten"))))]
        CLOCK_PROCESS_CPUTIME_ID,
        #[cfg(any(target_os = "freebsd",
                  target_os = "dragonfly"))]
        CLOCK_PROF,
        CLOCK_REALTIME,
        #[cfg(any(target_os = "fuchsia",
                  all(not(any(target_env = "uclibc",
                              target_env = "newlib")),
                      any(target_os = "linux",
                          target_os = "android",
                          target_os = "emscripten"))))]
        CLOCK_REALTIME_ALARM,
        #[cfg(any(target_os = "fuchsia",
                  all(not(any(target_env = "uclibc",
                              target_env = "newlib")),
                      any(target_os = "linux",
                          target_os = "android",
                          target_os = "emscripten"))))]
        CLOCK_REALTIME_COARSE,
        #[cfg(any(target_os = "freebsd",
                  target_os = "dragonfly"))]
        CLOCK_REALTIME_FAST,
        #[cfg(any(target_os = "freebsd",
                  target_os = "dragonfly"))]
        CLOCK_REALTIME_PRECISE,
        #[cfg(any(target_os = "freebsd",
                  target_os = "dragonfly"))]
        CLOCK_SECOND,
        #[cfg(any(target_os = "fuchsia",
                  all(not(any(target_env = "uclibc", target_env = "newlib")),
                      any(target_os = "emscripten",
                          all(target_os = "linux", target_env = "musl")))))]
        CLOCK_SGI_CYCLE,
        #[cfg(any(target_os = "fuchsia",
                  all(not(any(target_env = "uclibc", target_env = "newlib")),
                      any(target_os = "emscripten",
                          all(target_os = "linux", target_env = "musl")))))]
        CLOCK_TAI,
        #[cfg(any(
                  target_env = "uclibc",
                  target_os = "fuchsia",
                  target_os = "ios",
                  target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly",
                  all(
                      not(target_env = "newlib"),
                      any(
                          target_os = "linux",
                          target_os = "android",
                          target_os = "emscripten",
                      ),
                  ),
                )
            )]
        CLOCK_THREAD_CPUTIME_ID,
        #[cfg(any(target_os = "freebsd",
                  target_os = "dragonfly"))]
        CLOCK_UPTIME,
        #[cfg(any(target_os = "freebsd",
                  target_os = "dragonfly"))]
        CLOCK_UPTIME_FAST,
        #[cfg(any(target_os = "freebsd",
                  target_os = "dragonfly"))]
        CLOCK_UPTIME_PRECISE,
        #[cfg(any(target_os = "freebsd",
                  target_os = "dragonfly"))]
        CLOCK_VIRTUAL,
    }
}

pub fn clock_getres(clk_id: ClockId) -> Result<TimeSpec> {
    let mut c_time: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
    let errno = unsafe { libc::clock_getres(clk_id as libc::clockid_t, c_time.as_mut_ptr()) };
    Errno::result(errno)?;
    let res = unsafe { c_time.assume_init() };
    Ok(TimeSpec::from(res))
}

pub fn clock_gettime(clk_id: ClockId) -> Result<TimeSpec> {
    let mut c_time: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
    let errno = unsafe { libc::clock_gettime(clk_id as libc::clockid_t, c_time.as_mut_ptr()) };
    Errno::result(errno)?;
    let res = unsafe { c_time.assume_init() };
    Ok(TimeSpec::from(res))
}

#[cfg(not(
        any(
            target_os = "macos",
            target_os = "ios",
            all(
                not(any(target_env = "uclibc", target_env = "newlibc")),
                any(
                    target_os = "redox",
                    target_os = "hermit",
                ),
            ),
        )
    )
 )]
pub fn clock_settime(clk_id: ClockId, timespec: TimeSpec) -> Result<()> {
    let res = unsafe { libc::clock_settime(clk_id as libc::clockid_t, timespec.as_ref()) };
    Errno::result(res).map(drop)
}
