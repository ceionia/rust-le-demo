use core::{ptr::null_mut, arch::asm, alloc::Layout};
use core::fmt::Write;
use heapless::String;

const PTR_COUNT: usize = 1024;

struct Locked<A> {
    inner: spin::Mutex<A>,
}
impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner)
        }
    }
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub fn init_allocator() { unsafe {
    ALLOCATOR.inner.force_unlock();
    let mut t = ALLOCATOR.lock();
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
    let handle: u32;
    asm!(
        "mov edx, esi",
        "int 0x21",
        "mov eax, esi",
        "mov esi, edx",
        inout("eax") 0x0FF91 => handle,
        inout("ebx") PTR_COUNT * core::mem::size_of::<DpmiPtr>() => ptr,
        out("edx") _
    );
    t.ptrs = ptr;
    t.global_handle = handle;
    let mut ptrs = core::slice::from_raw_parts_mut::<DpmiPtr>(t.ptrs as *mut DpmiPtr, PTR_COUNT);
    for p in ptrs.iter_mut() {
        *p = DpmiPtr { ptr: 0, handle: 0 };
    }
} }

#[derive(Copy, Clone)]
struct DpmiPtr {
    ptr: u32,
    handle: u32
}
pub struct DpmiAlloc {
    ptrs: u32,
    global_handle: u32
}
impl DpmiAlloc {
    const fn new() -> Self { DpmiAlloc { ptrs: 0, global_handle: 0 } }
}

unsafe impl core::alloc::GlobalAlloc for Locked<DpmiAlloc> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut dat = self.lock();
        let mut ptrs = unsafe { core::slice::from_raw_parts_mut::<DpmiPtr>(dat.ptrs as *mut DpmiPtr, PTR_COUNT) };
        let free_idx: usize = {
            let mut tmp = None;
            for (i, p) in ptrs.iter().enumerate() {
                if p.ptr == 0 { tmp = Some(i); break; }
            }
            match tmp {
                None => { asm!("int 0x1", in("eax") ptrs.as_ptr()); return null_mut(); }
                Some(i) => i
            }
        };
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
        let handle: u32;
        asm!(
            "mov edx, esi",
            "int 0x21",
            "mov eax, esi",
            "mov esi, edx",
            inout("eax") 0x0FF91 => handle,
            inout("ebx") layout.size() as u32 => ptr,
            out("edx") _
        );
        ptrs[free_idx] = DpmiPtr { ptr, handle };
        ptr as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr as u32 == 0 { return; }
        let mut dat = self.lock();
        let mut ptrs = unsafe { core::slice::from_raw_parts_mut::<DpmiPtr>(dat.ptrs as *mut DpmiPtr, PTR_COUNT) };
        let mut found_idx: Option<usize> = None;
        for i in 0..ptrs.len() {
            if ptrs[i].ptr == ptr as u32 { found_idx = Some(i); break; }
        }
        let handle = match found_idx {
            None => return,
            Some(i) => ptrs[i].handle
        };
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
            in("ebx") handle,
            out("edx") _
        );
        ptrs[found_idx.unwrap()] = DpmiPtr { ptr: 0, handle: 0 };
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        if ptr.is_null() { return null_mut(); }
        let mut dat = self.lock();
        let mut ptrs = core::slice::from_raw_parts_mut::<DpmiPtr>(dat.ptrs as *mut DpmiPtr, PTR_COUNT);
        let mut found_idx: Option<usize> = None;
        for i in 0..ptrs.len() {
            if ptrs[i].ptr == ptr as u32 { found_idx = Some(i); break; }
        }
        let old_handle = match found_idx {
            None => return null_mut(),
            Some(i) => ptrs[i].handle
        };
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
        let ptr: u32;
        let handle: u32;
        asm!(
            "xchg edx, esi",
            "int 0x21",
            "xchg esi, edx",
            inout("eax") 0x0FF93 => _,
            inout("ebx") new_size as u32 => ptr,
            inout("edx") old_handle => handle
        );
        ptrs[found_idx.unwrap()] = DpmiPtr { ptr, handle };
        ptr as *mut u8
    }
}

#[global_allocator]
static ALLOCATOR: Locked<DpmiAlloc> = Locked::new(DpmiAlloc::new());

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

