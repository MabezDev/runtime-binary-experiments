#![no_std]
#![no_main]

extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
extern crate stm32l432xx_hal as hal;

use cortex_m_rt::entry;

#[link_section = ".app_section.data"]
static mut APPLICATION_RAM: [u8; 24 * 1024] = [0u8; 24 * 1024];

#[entry]
fn main() -> ! {
    let _app_addr = unsafe { &APPLICATION_RAM as *const _ } as usize;
    // setup serial

    // listen for start bytes

    // read in the binary
    // the binary will be sent as two ascii encoded bytes i.e 1 to F, anything outside this range will finish the loading
    // making a range of 0x01 to 0xFF 

    // convert the first 4 bytes into a ffi function pointer
    // let addr = ((APPLICATION_RAM[3] as u32) << 24)
    //         | ((APPLICATION_RAM[2] as u32) << 16)
    //         | ((APPLICATION_RAM[1] as u32) << 8)
    //         | ((APPLICATION_RAM[0] as u32) << 0);
    // let ptr = addr as *const ();
    // let code: extern "C" fn(*const Table) -> u32 = ::core::mem::transmute(ptr);
    // excute the function
    // let result = code(&t);
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
