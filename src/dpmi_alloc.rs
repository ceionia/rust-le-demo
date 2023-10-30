use core::{ptr::null_mut, arch::asm, alloc::Layout};
use core::fmt::Write;
use heapless::String;

pub struct DpmiAlloc { }
impl DpmiAlloc {
    const fn new() -> Self { DpmiAlloc {} }
}

unsafe impl core::alloc::GlobalAlloc for DpmiAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        //  3.53 - DOS function 0FF91h - DOS/32 Advanced Allocate High Memory Block
        //  In: 	AX = 0FF91h
        //  EBX = size of memory block in bytes
        //  Out: 	
        //  if successful:
        //  CF clear
        //  EBX = linear address of allocated memory block
        //  ESI = handle of allocated memory block
        //  if failed:
        //  CF set
        let ptr: u32;
        let _handle: u32;
        asm!(
            "mov edx, esi",
            "int 0x21",
            "mov eax, esi",
            "mov esi, edx",
            // We can calculate the handle ourself
            inout("eax") 0x0FF91 => _handle,
            inout("ebx") layout.size() as u32 => ptr,
            out("edx") _
        );
        ptr as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr as u32 == 0 { return; }
        //  3.54 - DOS function 0FF92h - DOS/32 Advanced Free High Memory Block
        //  In: 	AX = 0FF92h
        //  ESI = handle of previously allocated memory block
        //  Out: 	
        //  if successful:
        //  CF clear
        //  if failed:
        //  CF set
        asm!(
            "mov edx, esi",
            "mov esi, ebx",
            "int 0x21",
            "mov esi, edx",
            in("eax") 0x0FF92,
            in("ebx") ptr as u32 - 0x10, // in dos32a the handle should be just 0x10 less than the pointer
            out("edx") _
        );
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        if ptr.is_null() { return null_mut(); }
        //  3.55 - DOS function 0FF93h - DOS/32 Advanced Resize High Memory Block
        //  In: 	AX = 0FF93h
        //  EBX = new size of memory block in bytes
        //  ESI = handle of previously allocated memory block
        //  Out: 	

        //  if successful:
        //  CF clear
        //  EBX = new linear address of allocated memory block
        //  ESI = new handle of allocated memory block

        //  if failed:
        //  CF set
        let new_ptr: u32;
        let _handle: u32;
        asm!(
            "xchg edx, esi",
            "int 0x21",
            "xchg esi, edx",
            inout("eax") 0x0FF93 => _,
            inout("ebx") new_size as u32 => new_ptr,
            inout("edx") ptr as u32 - 0x10 => _handle
        );
        new_ptr as *mut u8
    }
}

#[global_allocator]
static ALLOCATOR: DpmiAlloc = DpmiAlloc::new();

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    let mut s: String<256> = String::new();
    s.write_fmt(args).unwrap();
    crate::dpmi::dpmi_print(s.as_str());
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    _print(format_args!("allocation error: {:?}$", layout));
    crate::dpmi::dpmi_exit();
}

