use crate::dpmi::dpmi_exit;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    //crate::println!("Panic! {}", info);
    dpmi_exit();
}

