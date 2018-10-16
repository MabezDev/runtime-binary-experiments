#![no_std]
#![no_main]

extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
extern crate stm32l432xx_hal as hal;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // setup serial

    // listen for start bytes

    // read in the binary
    // the binary will be sent as two ascii encoded bytes i.e 1 to F, anything outside this range will finish the loading
    // making a range of 0x01 to 0xFF 

    loop {
        // your code goes here
    }
}
