use crate::dpmi::dpmi_exit;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::println!("Panic! {}", info);
    dpmi_exit();
}

