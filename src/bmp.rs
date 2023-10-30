use core::mem::size_of;

use itertools::Itertools;

use alloc::boxed::Box;
use bitvec::{view::BitView, field::BitField};

use crate::println;
use crate::vga::Vga18;

#[repr(packed)]
struct PackedBmpHeader {
    bmp_type: u16,
    size: u32,
    reserved: u32,
    offset: u32,
    header_size: u32,
    width: u32,
    height: u32,
    planes: u16,
    bpp: u16,
    compression: u32,
    size_image: u32,
    xppm: u32,
    yppm: u32,
    colors_used: u32,
    colors_important: u32,
}
pub struct BmpHeader {
    pub bmp_type: u16,
    pub size: u32,
    pub reserved: u32,
    pub offset: u32,
    pub header_size: u32,
    pub width: u32,
    pub height: u32,
    pub planes: u16,
    pub bpp: u16,
    pub compression: u32,
    pub size_image: u32,
    pub xppm: u32,
    pub yppm: u32,
    pub colors_used: u32,
    pub colors_important: u32,
}
impl From::<PackedBmpHeader> for BmpHeader {
    fn from(value: PackedBmpHeader) -> Self {
	Self { bmp_type: value.bmp_type, size: value.size, reserved: value.reserved, offset: value.offset, header_size: value.header_size, width: value.width, height: value.height, planes: value.planes, bpp: value.bpp, compression: value.compression, size_image: value.size_image, xppm: value.xppm, yppm: value.yppm, colors_used: value.colors_used, colors_important: value.colors_important }
    }
}

pub struct Bmp {
    pub header: BmpHeader,
    pub palette_table: alloc::vec::Vec<Vga18>,
    pub data: Box<[u8]>
}

fn load_data(header: &PackedBmpHeader, raw: &[u8]) -> Option<Box<[u8]>> {
    match header.bpp {
        8 => {
	    let row_size = (((8 * header.width + 31) >> 5) << 2) as usize;
	    let chunks = raw[..row_size * header.height as usize]
		.chunks_exact(row_size).rev()
		.flat_map(|f| f[..header.width as usize].iter().cloned());
	    Some(chunks.collect())
	},
	bpp @ (4|2|1) => {
	    let row_size = ((bpp as usize * header.width as usize + 31) >> 5) << 2;
	    let chunks = raw[..row_size * header.height as usize]
		.chunks_exact(row_size).rev()
		.map(|row| row.view_bits::<bitvec::order::Msb0>()
		    .chunks(bpp as usize).take(header.width as usize) 
		    .map(|b| b.load_le::<u8>()));
	    Some(chunks.flatten().collect())
	}
	bpp => {
	    println!("Unsupported BPP {}", bpp);
	    return None
	}
    }
}

pub fn load_bmp(source: &[u8]) -> Option<Bmp> {
    // i'm sorry but i love loading file headers like this
    let header: PackedBmpHeader = unsafe {
	let mut copy: [u8; size_of::<PackedBmpHeader>()] = [0; size_of::<PackedBmpHeader>()];
	copy.copy_from_slice(&source[..size_of::<PackedBmpHeader>()]);
	core::mem::transmute(copy)
    };
    
    // 'BM'
    if header.bmp_type != 0x4D42 {
	println!("Invalid BMP type");
	return None
    }
    if header.size as usize != source.len() {
	println!("BMP size does not match file length!");
	return None
    }
    if header.offset as usize > source.len() {
	println!("BMP bitmap offset greater than file length!");
	return None
    }
    if header.compression != 0 {
	println!("Compressed BMPs are not supported");
	return None
    }
    if {
	let row_size = ((header.bpp as usize * header.width as usize + 31) >> 5) << 2;
	let (calc_size, overflow) = row_size.overflowing_mul(header.height as usize);
	(calc_size > (source.len() - header.offset as usize)) | overflow
    } {
	println!("File cannot possibly contain full BMP data!");
	return None
    }

    let color_table = &source[(header.offset - header.colors_used * 4) as usize..];
    let palette: alloc::vec::Vec<_> = color_table.iter()
	.tuples().take(header.colors_used as usize)
	.map(|(b,g,r,_)| Vga18 { red:r>>2,green:g>>2,blue:b>>2 })
	.collect();

    let raw_data = &source[header.offset as usize..];
    let data = load_data(&header, raw_data);

    Some(Bmp {
        header: BmpHeader::from(header), 
        palette_table: palette,
        data: data?
    })
}
