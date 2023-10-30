use core::arch::asm;

#[derive(Copy,Clone,Default,PartialEq)]
pub struct Vga18 {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

#[inline]
fn mode13h_vga_arr() -> &'static mut [[u8; 320]; 200] {
    unsafe { &mut *(0xa0000 as *mut [[u8; 320]; 200]) }
}

unsafe fn outb(port: u16, data: u8) {
    asm! {
        "out dx, al",
        in("dx") port,
        in("al") data,
    }
}

pub fn set_vga_dac_colors(start_index: u8, colors: &[Vga18]) {
    if colors.is_empty() { return }
    unsafe { outb(0x3c8, start_index); }
    for (i, &Vga18 { red, green, blue }) in colors.iter().enumerate() {
        if i + start_index as usize >= 256 {
            break
        }
        unsafe {
            outb(0x3c9, red);
            outb(0x3c9, green);
            outb(0x3c9, blue);
        }
    }
}

pub struct Mode13hDisplay {
    buffer: [[u8; 320]; 200]
}

impl Default for Mode13hDisplay {
    fn default() -> Self {
        Self { buffer: [[0; 320]; 200] }
    }
}

impl Mode13hDisplay {
    #[allow(unused)]
    pub fn flush(&self) {
        let vga = mode13h_vga_arr();
        *vga = self.buffer;
    }

    pub fn clear(&mut self) {
        self.buffer = [[0; 320]; 200];
    }

    pub fn copy_to_screen(&mut self, screen_col: isize, screen_line: isize, src_width: usize, src_height: usize, bytes: &[u8]) {
        let x = screen_col;
        let y = screen_line;

        for l in 0..src_height {
            let l_y = l as isize + y;
            // past screen
            if l_y >= 200 { break; }
            // before screen
            else if l_y < 0 { continue; }
            // past screen
            if -x >= src_width as isize { break; }

            let line_len = if x >= 0 {
                (320 - x).min(src_width as isize)
            } else {
                320.min(src_width as isize + x)
            };

            let src_off = if x >= 0 {
                l * src_width
            } else { ((l * src_width) as isize - x) as usize };

            let x_adj = x.max(0);

            self.buffer[l_y as usize][x_adj as usize..x_adj as usize+line_len as usize].copy_from_slice(&bytes[src_off..src_off+line_len as usize]);
        }
    }
}
