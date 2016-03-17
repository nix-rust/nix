use libc;
use {Errno, Result};
use std::mem;

#[derive(Clone, Copy)]
pub struct UContext {
    context: libc::ucontext_t,
}

impl UContext {
    pub fn get() -> Result<UContext> {
        let mut context: libc::ucontext_t = unsafe { mem::uninitialized() };
        let res = unsafe {
            libc::getcontext(&mut context as *mut libc::ucontext_t)
        };
        Errno::result(res).map(|_| UContext { context: context })
    }

    pub fn set(&self) -> Result<()> {
        let res = unsafe {
            libc::setcontext(&self.context as *const libc::ucontext_t)
        };
        Errno::result(res).map(drop)
    }
}
