#![no_std]
#![no_main]
use core::{any::Any, fmt::Write};
use cortex_m::peripheral;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;
use stm32f411_nucleo::button_interrupt;
use stm32f411_nucleo::uart;

#[entry]
fn main() -> ! {
    let periph = stm32f411::Peripherals::take().unwrap();

    button_interrupt::led_init(&periph);

    loop {
        button_interrupt::led_on(&periph.GPIOA);

        for i in 0..100_000 {
            cortex_m::asm::nop();
        }

        button_interrupt::led_off(&periph.GPIOA);

        for i in 0..100_000 {
            cortex_m::asm::nop();
        }
    }
}
