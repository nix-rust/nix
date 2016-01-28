use {Errno, Result, NixPath};
use libc::{c_int, c_char};

#[cfg(all(target_os = "linux",
          any(target_arch = "x86",
              target_arch = "x86_64",
              target_arch = "arm")),
          )]
pub mod quota {
    use libc::c_int;

    pub struct QuotaCmd(pub QuotaSubCmd, pub QuotaType);
    pub type QuotaSubCmd = c_int;

    impl QuotaCmd {
        pub fn as_int(&self) -> c_int {
            ((self.0 << 8) | (self.1 & 0x00ff)) as c_int
        }
    }

    // linux quota version >= 2
    pub const Q_SYNC:	QuotaSubCmd = 0x800001;
    pub const Q_QUOTAON:	QuotaSubCmd = 0x800002;
    pub const Q_QUOTAOFF:	QuotaSubCmd = 0x800003;
    pub const Q_GETFMT:	QuotaSubCmd = 0x800004;
    pub const Q_GETINFO:	QuotaSubCmd = 0x800005;
    pub const Q_SETINFO:	QuotaSubCmd = 0x800006;
    pub const Q_GETQUOTA:	QuotaSubCmd = 0x800007;
    pub const Q_SETQUOTA:	QuotaSubCmd = 0x800008;

    pub type QuotaType = c_int;

    pub const USRQUOTA:	QuotaType = 0;
    pub const GRPQUOTA:	QuotaType = 1;

    pub type QuotaFmt = c_int;

    pub const QFMT_VFS_OLD:	QuotaFmt = 1;
    pub const QFMT_VFS_V0:	QuotaFmt = 2;
    pub const QFMT_VFS_V1:  QuotaFmt = 4;

    bitflags!(
        #[derive(Default)]
        flags QuotaValidFlags: u32 {
            const QIF_BLIMITS	 = 1,
            const QIF_SPACE		 = 2,
            const QIF_ILIMITS	 = 4,
            const QIF_INODES	 = 8,
            const QIF_BTIME 	 = 16,
            const QIF_ITIME 	 = 32,
            const QIF_LIMITS 	 = QIF_BLIMITS.bits | QIF_ILIMITS.bits,
            const QIF_USAGE 	 = QIF_SPACE.bits | QIF_INODES.bits,
            const QIF_TIMES 	 = QIF_BTIME.bits | QIF_ITIME.bits,
            const QIF_ALL 		 = QIF_LIMITS.bits | QIF_USAGE.bits | QIF_TIMES.bits
        }
    );

    #[repr(C)]
    #[derive(Default,Debug,Copy,Clone)]
    pub struct Dqblk {
        pub bhardlimit: u64,
        pub bsoftlimit: u64,
        pub curspace:   u64,
        pub ihardlimit: u64,
        pub isoftlimit: u64,
        pub curinodes: u64,
        pub btime: u64,
        pub itime: u64,
        pub valid: QuotaValidFlags,
    }
}

mod ffi {
    use libc::{c_int, c_char};

    extern {
        pub fn quotactl(cmd: c_int, special: * const c_char, id: c_int, data: *mut c_char) -> c_int;
    }
}

use std::ptr;

fn quotactl<P: ?Sized + NixPath>(cmd: quota::QuotaCmd, special: Option<&P>, id: c_int, addr: *mut c_char) -> Result<()> {
    unsafe {
        Errno::clear();
        let res = try!(
            match special {
                Some(dev) => dev.with_nix_path(|path| ffi::quotactl(cmd.as_int(), path.as_ptr(), id, addr)),
                None => Ok(ffi::quotactl(cmd.as_int(), ptr::null(), id, addr)),
            }
        );

        Errno::result(res).map(drop)
    }
}

pub fn quotactl_on<P: ?Sized + NixPath>(which: quota::QuotaType, special: &P, format: quota::QuotaFmt, quota_file: &P) -> Result<()> {
    try!(quota_file.with_nix_path(|path| {
        let mut path_copy = path.to_bytes_with_nul().to_owned();
        let p: *mut c_char = path_copy.as_mut_ptr() as *mut c_char;
        quotactl(quota::QuotaCmd(quota::Q_QUOTAON, which), Some(special), format as c_int, p)
    }))
}

pub fn quotactl_off<P: ?Sized + NixPath>(which: quota::QuotaType, special: &P) -> Result<()> {
    quotactl(quota::QuotaCmd(quota::Q_QUOTAOFF, which), Some(special), 0, ptr::null_mut())
}

pub fn quotactl_sync<P: ?Sized + NixPath>(which: quota::QuotaType, special: Option<&P>) -> Result<()> {
    quotactl(quota::QuotaCmd(quota::Q_SYNC, which), special, 0, ptr::null_mut())
}

pub fn quotactl_get<P: ?Sized + NixPath>(which: quota::QuotaType, special: &P, id: c_int, dqblk: &mut quota::Dqblk) -> Result<()> {
    use std::mem;
    unsafe {
        quotactl(quota::QuotaCmd(quota::Q_GETQUOTA, which), Some(special), id, mem::transmute(dqblk))
    }
}

pub fn quotactl_set<P: ?Sized + NixPath>(which: quota::QuotaType, special: &P, id: c_int, dqblk: &quota::Dqblk) -> Result<()> {
    use std::mem;
    let mut dqblk_copy = *dqblk;
    unsafe {
        quotactl(quota::QuotaCmd(quota::Q_SETQUOTA, which), Some(special), id, mem::transmute(&mut dqblk_copy))
    }
}
