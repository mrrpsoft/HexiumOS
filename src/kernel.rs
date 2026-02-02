#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_colors;
mod writer;

use vga_colors::{Color, color_code};
use writer::Writer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let mut writer = Writer::new(color_code(Color::LightGreen, Color::Black));
    writer.clear();

    writer.write_str("Welcome to Hexium OS!\n");

    writer.set_color(Color::Yellow, Color::Black);
    writer.write_str("Sigma list:\n");
    writer.set_color(Color::LightCyan, Color::Black);
    writer.write_str("  - 67\n");
    writer.write_str("  - So Sigma\n");
    writer.write_str("  - Wowie\n");
    writer.write_str("  - Bleh :P\n");

    loop {}
}


