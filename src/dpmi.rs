use core::{arch::asm, fmt::{Arguments, Write}};

extern crate alloc;

#[allow(dead_code)]
#[repr(packed)]
pub struct DpmiRegs {
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    reserved_zero: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub status_flags: u16,
    pub es: u16,
    pub ds: u16,
    pub fs: u16,
    pub gs: u16,
    ip_ignored: u16,
    cs_ignored: u16,
    pub sp: u16,
    pub ss: u16
}
impl DpmiRegs {
    pub const fn zero() -> Self {
        DpmiRegs { edi: 0, esi: 0, ebp: 0, reserved_zero: 0, ebx: 0, edx: 0, ecx: 0, eax: 0, status_flags: 0, es: 0, ds: 0, fs: 0, gs: 0, ip_ignored: 0, cs_ignored: 0, sp: 0, ss: 0 }
    }
}

#[inline]
pub fn set_video_mode(mode: u8) {
    let mut regs = DpmiRegs::zero();
    regs.eax = mode as u32;
    real_int(0x10, &mut regs);
}

pub fn getchar() -> u16 {
    let mut regs = DpmiRegs::zero();
    real_int(0x16, &mut regs);
    regs.eax as u16
}

pub fn kb_status() -> Option<u16> {
    let mut regs = DpmiRegs::zero();
    regs.eax = 0x100;
    real_int(0x16, &mut regs);
    match regs.status_flags & 0x40 {
        0 => match regs.eax as u16 {
            0 => None,
            v => Some(v),
        },
        _ => None
    }
}

pub fn get_psp() -> *const u8 {
    let psp: u32;
    unsafe { asm!(
        "int 0x21",
        in("ax") 0x5100_u16,
        out("ebx") psp
    ); }
    (psp << 4) as *const u8
}

pub fn real_int(int: u8, regs: &mut DpmiRegs) {
    unsafe { asm!(
        "int 0x31",
        in("bx") 0x0000_u16 | int as u16,
        in("cx") 0x0000_u16,
        in("edi") regs,
        inout("ax") 0x0300_u16 => _
    );}
}

pub fn dpmi_print(string: &str) {
    // DOS DPMI function 21h, AH 9h
    // Prints string pointed to
    // by EDX, terminated with $
    unsafe { asm!(
        "int 0x21",
        in("ah") 0x9_u8,
        in("edx") string.as_ptr()
    ); }
}

pub fn dpmi_exit() -> ! {
    // DOS DPMI function Exit: 21h, AH 4Ch
    unsafe { asm!(
        "int 0x21",
        in("ah") 0x4C_u8
    ); }
    loop {}
}

#[macro_export]
#[doc(hidden)]
macro_rules! _raw_print {
    ($($arg:tt)*) => ($crate::dpmi::_print(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_raw_print!("{}$", format_args!($($arg)*)));
}
#[macro_export]
macro_rules! println {
    () => (_raw_print!("\r\n$"));
    ($($arg:tt)*) => ($crate::_raw_print!("{}\r\n$", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    let mut s: alloc::string::String = alloc::string::String::new();
    //let mut s: heapless::String<20> = heapless::String::new();
    s.write_fmt(args).unwrap();
    dpmi_print(s.as_str());
}

