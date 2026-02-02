use core::arch::asm;
use core::arch::naked_asm;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    offset_high: u16,
}

impl IdtEntry {
    pub const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            zero: 0,
            type_attr: 0,
            offset_high: 0,
        }
    }

    pub fn set_handler(&mut self, handler: u32) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.offset_high = ((handler >> 16) & 0xFFFF) as u16;
        self.selector = 0x08;
        self.zero = 0;
        self.type_attr = 0x8E;
    }
}

#[repr(C, packed)]
pub struct IdtPointer {
    limit: u16,
    base: u32,
}

const IDT_SIZE: usize = 256;
static mut IDT: [IdtEntry; IDT_SIZE] = [IdtEntry::empty(); IDT_SIZE];
static mut IDT_PTR: IdtPointer = IdtPointer { limit: 0, base: 0 };

pub fn init() {
    unsafe {
        IDT[0x21].set_handler(keyboard_interrupt_handler as u32);

        IDT_PTR.limit = (core::mem::size_of::<[IdtEntry; IDT_SIZE]>() - 1) as u16;
        IDT_PTR.base = IDT.as_ptr() as u32;

        asm!("lidt [{}]", in(reg) &IDT_PTR, options(nostack));
    }

    init_pics();

    unsafe {
        asm!("sti", options(nostack));
    }
}

fn init_pics() {
    unsafe {
        outb(0x20, 0x11);
        outb(0xA0, 0x11);

        outb(0x21, 0x20);
        outb(0xA1, 0x28);
       
        outb(0x21, 0x04);
        outb(0xA1, 0x02);

        outb(0x21, 0x01);
        outb(0xA1, 0x01);

        outb(0x21, 0xFD);
        outb(0xA1, 0xFF);
    }
}

unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nostack)
    );
}

pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nostack)
    );
    value
}

const BUFFER_SIZE: usize = 32;
static mut KEY_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
static mut BUFFER_HEAD: usize = 0;
static mut BUFFER_TAIL: usize = 0;

pub fn get_scancode() -> Option<u8> {
    unsafe {
        if BUFFER_HEAD == BUFFER_TAIL {
            None
        } else {
            let scancode = KEY_BUFFER[BUFFER_TAIL];
            BUFFER_TAIL = (BUFFER_TAIL + 1) % BUFFER_SIZE;
            Some(scancode)
        }
    }
}

fn buffer_push(scancode: u8) {
    unsafe {
        let next_head = (BUFFER_HEAD + 1) % BUFFER_SIZE;
        if next_head != BUFFER_TAIL {
            KEY_BUFFER[BUFFER_HEAD] = scancode;
            BUFFER_HEAD = next_head;
        }
    }
}

#[no_mangle]
pub extern "C" fn keyboard_handler_inner() {
    unsafe {
        let scancode = inb(0x60);
        
        buffer_push(scancode);

        outb(0x20, 0x20);
    }
}

#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn keyboard_interrupt_handler() {
    naked_asm!(
        "pusha",
        "call keyboard_handler_inner",
        "popa",
        "iretd",
    );
}
