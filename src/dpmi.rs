#![allow(dead_code)]
use core::{arch::asm, fmt::{Arguments, Write}, ffi::CStr, str::FromStr};

extern crate alloc;

#[allow(dead_code)]
#[repr(packed)]
#[derive(Default)]
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

// BIOS function INT 10,0 Set video mode
pub fn set_video_mode(mode: u8) {
    let mut regs = DpmiRegs::zero();
    regs.eax = mode as u32;
    real_int(0x10, &mut regs);
}

// BIOS function INT 16,0 Wait for keystroke and read
pub fn getchar() -> u16 {
    let mut regs = DpmiRegs::zero();
    real_int(0x16, &mut regs);
    regs.eax as u16
}

// BIOS function INT 16,1 Get keyboard status
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

pub fn get_psp(buff: &mut [u8; 256]) {
    // DOS DPMI function 21h,AH 62h - Get PSP Selector
    // Out: EBX = PSP selector
    // this really was hurting me so i just did it in assembly
    unsafe { asm!(
        "int 0x21",
        "push esi",
        "push edi",
        "mov ds, bx",
        "xor esi, esi",
        "mov ecx, 256",
        "rep movsb",
        "pop edi",
        "pop esi",
        "push es",
        "pop ds",
        in("ax") 0x6200_u16,
        out("ebx") _,
        in("edi") buff.as_mut_ptr(),
    ); }
}

pub fn get_args() -> alloc::vec::Vec<alloc::string::String> {
    let mut buff = [0_u8; 256];
    get_psp(&mut buff);
    let cmd_byte_cnt = buff[0x80];
    let cmd_line = &buff[0x81..0x100.min(0x81+cmd_byte_cnt as usize)];
    // this is a horrible way of doing this, but i'm lazy
    core::str::from_utf8(cmd_line).unwrap()
        .split_whitespace()
        .map(|s| alloc::string::String::from_str(s).unwrap())
        .collect()
}

pub struct File {
    handle: u32,
    size: u32
}

impl File {
    pub fn open(string: &CStr) -> Option<Self> {
        // DOS DPMI function 21h, AH 3Dh - Open File
        // In:
        //     AL = access mode
        //     DS:EDX = pointer to ASCIIZ file name
        // Out:
        //     if successful:
        //     CF clear
        //     EAX = file handle
        //     
        //     if failed:
        //     CF set
        //     EAX = DOS error code
        let err: u8;
        let eax: u32;
        unsafe { asm!(
            "int 0x21",
            "setc bl",
            inout("eax") 0x00003D00_u32 => eax,
            in("edx") string.as_ptr(),
            inout("bl") 0_u8 => err
        );}

        // Error TODO return error code instead of printing
        if err == 1 {
            crate::println!("Could not open file \"{}\" ({}).", string.to_str().unwrap(), eax);
            return None;
        }

        let mut file = Self {
            handle: eax,
            size: 0
        };
        let size = file.find_size()?;
        file.size = size;

        return Some(file);
    }

    pub fn get_size(&self) -> u32 { self.size }

    fn seek(&mut self, origin: u8) -> Option<u32> {
        // DOS DPMI function 21h, AH 42h - Set Current File Position
        // In:
        //     AH = 42h
        //     AL = origin of move:
        //     
        //         00h = start
        //         01h = current
        //         02h = end
        //     
        //     ECX:EDX = file position
        // Out:
        //     if successful:
        //     CF clear
        //     EDX:EAX = current file position
        //     
        //     if failed:
        //     CF set
        //     EAX = DOS error code
        let err: u32;
        let eax: u32;
        let edx: u32;
        unsafe { asm!(
            "int 0x21",
            "setc bl",
            inout("eax") 0x00004200_u32 | origin as u32 => eax,
            in("ecx") 0,
            inout("edx") 0 => edx,
            inout("ebx") self.handle => err
        );}

        // Error TODO return error code
        if err == 1 { return None }

        return Some(eax | (edx << 16));
    }

    fn find_size(&mut self) -> Option<u32> {
        let size = self.seek(2);
        self.seek(0)?;
        return size;
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Option<u32> {
        // DOS DPMI function 21h, AH 3Fh - Read File
        // In:
        //      AH = 3Fh
        //      EBX = file handle
        //      ECX = number of bytes to read (size)
        //      DS:EDX = pointer to buffer to read to (addr)
        // Out:
        //      if successful:
        //      CF clear
        //      EAX = number of bytes read
        //      
        //      if failed:
        //      CF set
        //      EAX = DOS error code
        let err: u32;
        let eax: u32;
        unsafe { asm!(
            "int 0x21",
            "mov ebx, 0",
            "setc bl",
            inout("eax") 0x00003F00_u32 => eax,
            inout("ebx") self.handle => err,
            in("ecx") buffer.len(),
            in("edx") buffer.as_mut_ptr(),
        );}

        // Error TODO return error code
        if err == 1 { return None }

        return Some(eax);
    }
}

pub fn real_int(int: u8, regs: &mut DpmiRegs) {
    // DPMI function 0300h - Simulate Real Mode Interrupt
    // TODO get error codes from AX/CF
    unsafe { asm!(
        "int 0x31",
        in("bx") 0x0000_u16 | int as u16,
        in("cx") 0x0000_u16,
        in("edi") regs,
        inout("ax") 0x0300_u16 => _
    );}
}

pub struct IntHandler {
    interrupt: u8,
    selector: u16,
    offset: u32
}
impl IntHandler {
    pub fn new(interrupt: u8) -> Self {
        // get old handler
        // DPMI function 0204h - Get Protected Mode Interrupt Vector
        let selector: u16;
        let offset: u32;
        unsafe { asm!(
            "int 0x31",
            in("ax") 0x0204,
            in("bl") interrupt,
            out("edx") offset,
            out("cx") selector
        ); }
        Self { interrupt, offset, selector }
    }
    pub fn set_handler(&mut self, f: extern "x86-interrupt" fn()) {
        // install new handler
        // DPMI function 0205h - Set Protected Mode Interrupt Vector
        let func_ptr = f as *const () as u32;
        unsafe { asm!(
            "mov cx, cs",
            "int 0x31",
            inout("ax") 0x0205 => _,
            in("bl") self.interrupt,
            in("edx") func_ptr,
            out("cx") _
        ); }
    }
    pub fn restore_handler(&mut self) {
        // restore original handler
        // DPMI function 0205h - Set Protected Mode Interrupt Vector
        unsafe { asm!(
            "int 0x31",
            inout("ax") 0x0205 => _,
            in("bl") self.interrupt,
            in("cx") self.selector,
            in("edx") self.offset,
        ); }
    }
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
    () => ($crate::_raw_print!("\r\n$"));
    ($($arg:tt)*) => ($crate::_raw_print!("{}\r\n$", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    let mut s: alloc::string::String = alloc::string::String::new();
    s.write_fmt(args).unwrap();
    dpmi_print(s.as_str());
}

