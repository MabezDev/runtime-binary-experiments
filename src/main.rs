#![no_std]
#![no_main]

extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
#[macro_use(block)]
extern crate nb;
extern crate stm32l432xx_hal as hal;
extern crate cortex_m_semihosting as sh;
extern crate embedded_graphics;

use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::stm32l4::stm32l4x2;

use ssd1351::builder::Builder;
use ssd1351::mode::{GraphicsMode};
use ssd1351::prelude::*;

use crate::hal::delay::Delay;
use crate::hal::spi::Spi;


use cortex_m_rt::{entry, exception, ExceptionFrame};
use core::fmt::Write;

#[link_section = ".app_section.data"]
static mut APPLICATION_RAM: [u8; 32 * 1024] = [0u8; 32 * 1024];

use mabez_watch_sdk_core::{Table, Context};

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[entry]
fn main() -> ! {
    let mut hstdout = sh::hio::hstdout().unwrap();
    let app_addr = unsafe { &APPLICATION_RAM as *const _ } as usize;
    
    let p = stm32l4x2::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);

    let clocks = rcc.cfgr.sysclk(80.mhz()).pclk1(80.mhz()).pclk2(80.mhz()).freeze(&mut flash.acr);

    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);

    let mut delay = Delay::new(cp.SYST, clocks);
    let mut rst = gpiob
        .pb0
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let dc = gpiob
        .pb1
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let spi = Spi::spi1(
        p.SPI1,
        (sck, miso, mosi),
        SSD1351_SPI_MODE,
        2.mhz(), // TODO increase this when off the breadboard!
        clocks,
        &mut rcc.apb2,
    );

    let mut display: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();
    display.reset(&mut rst, &mut delay);
    display.init().unwrap();

    let serial = Serial::usart1(p.USART1, (tx, rx), 11520.bps(), clocks, &mut rcc.apb2);
    let (mut tx, mut rx) = serial.split();

    let mut context = Context {
        display: display
    };
    let context = &mut context;

    loop {

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
            let ptr = addr as *const ();

            // struct callbacks_t {
            //     void* p_context;
            //     int32_t(*puts)(void* p_context, const char*);
            // };
            writeln!(hstdout, "Loaded {} bytes into buffer at {:08X}. Will execute at {:08X}.", i, app_addr, addr).unwrap();
            context.display.clear();
            let t = Table {
                context: context as *mut Context,
                draw_pixel: draw_pixel,
            };
            let code: extern "C" fn(*const Table) -> u32 = ::core::mem::transmute(ptr);
            // excute the function
            let result = code(&t);

            writeln!(hstdout, "Result of execution {}", result);
        }
    }
}

pub(crate) extern "C" fn draw_pixel(context: *mut Context, x: u8, y: u8, colour: u16) -> i32 {
    let ctx = unsafe {
        &mut *context
    };
    ctx.display.set_pixel(x as u32, y as u32, colour);
    0
}
