

#[cfg(any(target_arch = "x86_64"))]
use libc::{c_ushort, c_uint, c_char, c_int, c_longlong, c_ulonglong};
#[cfg(any(target_arch = "x86"))]
use libc::{c_ushort, c_uint, c_char, c_int, c_long, c_ulong};


libc_bitflags! {
    pub flags ExtendedFlags: libc::size_t{
        EFLAG_CARRY = 1,
        EFLAG_PARITY = 3,
        EFLAG_AUXILIARY_CARRY = 5,
        EFLAG_ZERO = 7,
        EFLAG_SIGN = 8,
        EFLAG_TRAP = 9,
        EFLAG_INTERRUPT_ENABLE = 10,
        EFLAG_DIRECTION = 11,
        EFLAG_OVERFLOW = 12,
        EFLAG_IOPRIVILEGE_1 = 13,
        EFLAG_IOPRIVILEGE_2 = 14,
        EFLAG_NESTEDTASK = 15,
        EFLAG_RESUME = 17,
        EFLAG_VIRTUAL_80086_MODE = 18,
        EFLAG_ALIGNMENT_CHECK = 19,
        EFLAG_VIRTUAL_INTERRUPT = 20,
        EFLAG_VIRTUAL_INTERRUPT_PENDING = 21,
        EFLAG_CPUID = 22
    }
}
#[repr(C)]
#[cfg(any(target_arch = "x86_64"))]
pub struct FpRegs {
    pub cwd: c_ushort,
    pub swd: c_ushort,
    pub ftw: c_ushort,
    pub fop: c_ushort,
    pub rip: c_ulonglong,
    pub rdp: c_ulonglong,
    pub mxcsr: c_uint,
    pub mxcr_mask: c_uint,
    pub st_space: [c_uint; 32],
    pub xmm_space: [c_uint; 64],
    pub padding: [c_uint; 24],
}


#[repr(C)]
#[cfg(any(target_arch = "x86_64"))]
pub struct Regs {
    pub r15: c_ulonglong,
    pub r14: c_ulonglong,
    pub r13: c_ulonglong,
    pub r12: c_ulonglong,
    pub rbp: c_ulonglong,
    pub rbx: c_ulonglong,
    pub r11: c_ulonglong,
    pub r10: c_ulonglong,
    pub r9: c_ulonglong,
    pub r8: c_ulonglong,
    pub rax: c_ulonglong,
    pub rcx: c_ulonglong,
    pub rdx: c_ulonglong,
    pub rsi: c_ulonglong,
    pub rdi: c_ulonglong,
    pub orig_rax: c_ulonglong,
    pub rip: c_ulonglong,
    pub cs: c_ulonglong,
    pub eflags: ExtendedFlags,
    pub rsp: c_ulonglong,
    pub ss: c_ulonglong,
    pub fs_base: c_ulonglong,
    pub gs_base: c_ulonglong,
    pub ds: c_ulonglong,
    pub es: c_ulonglong,
    pub fs: c_ulonglong,
    pub gs: c_ulonglong,
}


#[repr(C)]
#[cfg(any(target_arch = "x86_64"))]
pub struct User {
    regs: Regs,
    u_fpvalid: c_int,
    i387: FpRegs,
    u_tsize: c_ulonglong,
    u_dsize: c_ulonglong,
    u_ssize: c_ulonglong,
    start_code: c_ulonglong,
    start_stack: c_ulonglong,
    signal: c_longlong,
    reserved: c_int,
    u_ar0: *mut Regs,
    u_fpstate: *mut FpRegs,
    magic: c_ulonglong,
    u_comm: [c_char; 32],
    u_debugreg: [c_ulonglong; 8]
}



#[repr(C)]
#[cfg(not(any(target_arch = "x86_64")))]
pub struct FpRegs {
    pub cwd: c_long,
    pub swd: c_long,
    pub ftw: c_long,
    pub fip: c_long,
    pub fcs: c_long,
    pub foo: c_long,
    pub fos: c_long,
    pub st_space: [c_long; 20],
}


#[repr(C)]
#[cfg(not(any(target_arch = "x86_64")))]
pub struct FpxRegs {
    pub cwd: c_ushort,
    pub swd: c_ushort,
    pub twd: c_ushort,
    pub fop: c_ushort,
    pub fip: c_long,
    pub fcs: c_long,
    pub foo: c_long,
    pub fos: c_long,
    pub mxcsr: c_long,
    pub reserved: c_long,
    pub st_space: [c_long; 32],
    pub xmm_space: [c_long; 32],
    pub padding: [c_long; 56],
}

#[repr(C)]
#[cfg(not(any(target_arch = "x86_64")))]
pub struct Regs {
    pub ebx: c_long,
    pub ecx: c_long,
    pub edx: c_long,
    pub esi: c_long,
    pub edi: c_long,
    pub ebp: c_long,
    pub eax: c_long,
    pub xds: c_long,
    pub xes: c_long,
    pub xfs: c_long,
    pub xgs: c_long,
    pub orig_eax: c_long,
    pub eip: c_long,
    pub xcs: c_long,
    pub eflags: ExtendedFlags,
    pub esp: c_long,
    pub xss: c_long,
}


#[repr(C)]
#[cfg(not(any(target_arch = "x86_64")))]
pub struct User {
    regs: Regs,
    u_fpvalid: c_int,
    i387: FpRegs,
    u_tsize: c_ulong,
    u_dsize: c_ulong,
    u_ssize: c_ulong,
    start_code: c_ulong,
    start_stack: c_ulong,
    signal: c_long,
    reserved: c_int,
    u_ar0: *mut Regs,
    u_fpstate: *mut FpRegs,
    magic: c_ulong,
    u_comm: [c_char; 32],
    u_debugreg: [c_int; 8],
}
