#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![no_main]
#![no_std]

use core::arch::asm;

extern crate alloc;

mod dpmi;
mod dpmi_alloc;
mod panic;
mod vga;
mod bmp;

use alloc::{vec, ffi::CString};
use bmp::Bmp;
use vga::Mode13hDisplay;

const TEST_BMP: &[u8; 5318] = include_bytes!("chicken.bmp");

#[no_mangle]
pub extern "C" fn start() {
    unsafe { asm!(
        "mov ax, es",
        "push ds",
        "pop es",
    ); }
    main();
    dpmi::dpmi_exit();
}

fn main() {
    let args = dpmi::get_args();
    let filename = {
        if args.is_empty() || args[0].is_empty() {
            println!("Filename required.");
            println!("Press Q to exit, or any key to continue with the default image.");
            match dpmi::getchar() as u8 {
                b'q' | b'Q' => return,
                _ => None
            }
        } else {
            Some(&*args[0])
        }
    };

    // Try to load BMP file from filename, or else use the included test image
    let bmp = {
        let mut bmp_buff;
        let src = if let Some(filename) = filename {
            println!("Loading BMP from {}...", filename);
            let mut file = match dpmi::File::open(&CString::new(filename).unwrap()) {
                Some(f) => f,
                None => {
                    println!("Could not open file.");
                    return;
                }
            };
            println!("File size: {} bytes", file.get_size());

            bmp_buff = vec![0; file.get_size() as usize];
            file.read(&mut bmp_buff);

            &bmp_buff[..]
        } else {
            TEST_BMP
        };
        if let Some(bmp) = bmp::load_bmp(src) {
            bmp
        } else {
            println!("Could not open BMP. Exiting");
            return;
        }
    };

    println!("Width x Height x BPP:   {}x{}x{}", bmp.header.width, bmp.header.height, bmp.header.bpp);
    println!("Colors Used, Important: {},{}", bmp.header.colors_used, bmp.header.colors_important);
    println!("Arrow keys to move, 1-9 to change speed, Q to exit.");
    println!("Press any key to continue.");
    dpmi::getchar();

    // mode 13h, 320x200 256 color graphics
    dpmi::set_video_mode(0x13);
    // Get screen buffer
    let mut vga = Mode13hDisplay::default();

    // sets the VGA screen palette to the BMP color palette
    vga::set_vga_dac_colors(0, &bmp.palette_table);

    // set up new keyboard handler
    // could do getchar, but this is more fun
    let mut kb_handler = dpmi::IntHandler::new(9);
    kb_handler.set_handler(keyboard_int_handler);

    let mut last_scancode = 0xFF;
    let mut delta = 1;
    let mut pos = Position { x: 0, y: 0 };
    loop {
        let scancode = { *SCANCODE.read() };
        if scancode != last_scancode {
            match scancode {
                0x48 => { pos.y += delta; }, // up
                0x4B => { pos.x -= delta; }, // left
                0x4D => { pos.x += delta; }, // right
                0x50 => { pos.y -= delta; }, // down
                s @ 0x02..=0x0A => { delta = s as isize - 1; }, // 1-9
                0x10 => break, // q
                _ => {}
            }
            last_scancode = scancode;
            draw_loop(&mut vga, &bmp, &pos);
        }
        // halt processor so we don't burn the CPU
        unsafe { asm!("hlt"); }
    }

    // restore old keyboard handler
    kb_handler.restore_handler();

    // mode 3h, text mode graphics, DOS default
    dpmi::set_video_mode(0x3);
}

fn draw_loop(vga: &mut Mode13hDisplay, bmp: &Bmp, pos: &Position) {
    vga.clear();
    vga.copy_to_screen(pos.x, pos.y, bmp.header.width as usize, bmp.header.height as usize, &bmp.data);
    // TODO wait for blanking interval
    vga.flush();
}

struct Position {
    x: isize,
    y: isize,
}

static SCANCODE: spin::RwLock<u8> = spin::RwLock::new(0);
pub extern "x86-interrupt" fn keyboard_int_handler() {
    let old_ds: u16;
    unsafe { asm!(
        "mov bx, ds",
        "mov ax, es",
        "mov ds, ax",
        out("bx") old_ds
    ); }

    let code: u8;
    unsafe { asm!(
        "in al, 0x60",
        out("al") code
    ); }

    let mut scan = SCANCODE.write();
    *scan = code;
    drop(scan);

    unsafe { asm!(
        "mov ds, bx",
        "out 0x20, al",
        in("al") 0x20_u8,
        in("bx") old_ds
    ); }
}
