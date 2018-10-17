#![no_std]
#![no_main]

extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
#[macro_use(block)]
extern crate nb;
extern crate stm32l432xx_hal as hal;
extern crate cortex_m_semihosting as sh;

use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::stm32l4::stm32l4x2;

use cortex_m_rt::{entry, exception, ExceptionFrame};
use core::fmt::Write;

#[link_section = ".app_section.data"]
static mut APPLICATION_RAM: [u8; 32 * 1024] = [0u8; 32 * 1024];

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[entry]
fn main() -> ! {
    let mut hstdout = sh::hio::hstdout().unwrap();
    let app_addr = unsafe { &APPLICATION_RAM as *const _ } as usize;
    
    let p = stm32l4x2::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);

    let clocks = rcc.cfgr.sysclk(80.mhz()).pclk1(80.mhz()).pclk2(80.mhz()).freeze(&mut flash.acr);

    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);

    let serial = Serial::usart1(p.USART1, (tx, rx), 9_600.bps(), clocks, &mut rcc.apb2);
    let (mut tx, mut rx) = serial.split();

    for _b in b"\rload\r" {
        block!(rx.read()).unwrap();
    }

    for b in b"READY" {
        block!(tx.write(*b)).ok();
    }

    let mut i = 0;
    let max_bytes = unsafe { APPLICATION_RAM.len() };
    const ACK_EVERY: usize = 4;
    let mut ack_count = 0;
    while i < max_bytes {
        let ch = loop {
            match block!(rx.read()) {
                Ok(x) => break x,
                _ => {}
            }
        };
        let mut byte = match ch {
            b'0' => 0x00,
            b'1' => 0x10,
            b'2' => 0x20,
            b'3' => 0x30,
            b'4' => 0x40,
            b'5' => 0x50,
            b'6' => 0x60,
            b'7' => 0x70,
            b'8' => 0x80,
            b'9' => 0x90,
            b'A' => 0xA0,
            b'B' => 0xB0,
            b'C' => 0xC0,
            b'D' => 0xD0,
            b'E' => 0xE0,
            b'F' => 0xF0,
            b'a' => 0xA0,
            b'b' => 0xB0,
            b'c' => 0xC0,
            b'd' => 0xD0,
            b'e' => 0xE0,
            b'f' => 0xF0,
            _ => break,
        };
        let ch = loop {
            match block!(rx.read()) {
                Ok(x) => break x,
                _ => {}
            }
        };
        byte |= match ch {
            b'0' => 0x00,
            b'1' => 0x01,
            b'2' => 0x02,
            b'3' => 0x03,
            b'4' => 0x04,
            b'5' => 0x05,
            b'6' => 0x06,
            b'7' => 0x07,
            b'8' => 0x08,
            b'9' => 0x09,
            b'A' => 0x0A,
            b'B' => 0x0B,
            b'C' => 0x0C,
            b'D' => 0x0D,
            b'E' => 0x0E,
            b'F' => 0x0F,
            b'a' => 0x0A,
            b'b' => 0x0B,
            b'c' => 0x0C,
            b'd' => 0x0D,
            b'e' => 0x0E,
            b'f' => 0x0F,
            _ => break,
        };
        unsafe {
            APPLICATION_RAM[i] = byte;
            ack_count += 1;
            if ack_count >= ACK_EVERY {
                let _ = block!(tx.write(b'X')).ok();
                ack_count = 0;
            }
        }
        i = i + 1;
    }

    // read in the binary
    // the binary will be sent as two ascii encoded bytes i.e 1 to F, anything outside this range will finish the loading
    // making a range of 0x01 to 0xFF 

    // convert the first 4 bytes into a ffi function pointer
    unsafe {
        let addr = ((APPLICATION_RAM[3] as u32) << 24)
                | ((APPLICATION_RAM[2] as u32) << 16)
                | ((APPLICATION_RAM[1] as u32) << 8)
                | ((APPLICATION_RAM[0] as u32) << 0);
        let _ptr = addr as *const ();
        // let code: extern "C" fn(*const Table) -> u32 = ::core::mem::transmute(ptr);
        // excute the function
        // let result = code(&t);
        writeln!(hstdout, "Loaded {} bytes into buffer at {:08X}. Will execute at {:08X}.", i, app_addr, addr).unwrap();
    }
    loop {
        // your code goes here
    }
}

//TODO: think about ABI required, example below
// #[repr(C)]
// struct Table {
//     context: *mut Context,
//     putchar: extern "C" fn(*mut Context, u8) -> i32,
//     puts: extern "C" fn(*mut Context, *const u8) -> i32,
//     readc: extern "C" fn(*mut Context) -> i32,
//     wfvbi: extern "C" fn(*mut Context),
//     kbhit: extern "C" fn(*mut Context) -> i32,
//     move_cursor: extern "C" fn(*mut Context, u8, u8),
//     play: extern "C" fn (*mut Context, u32, u8, u8, u8) -> i32,
// }
// let t = Table {
//     context: context as *mut Context,
//     putchar: api::putchar,
//     puts: api::puts,
//     readc: api::readc,
//     wfvbi: api::wfvbi,
//     kbhit: api::kbhit,
//     move_cursor: api::move_cursor,
//     play: api::play,
// };
