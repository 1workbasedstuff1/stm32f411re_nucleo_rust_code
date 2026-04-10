#![no_std]
#![no_main]
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;

#[entry]
fn main() -> ! {
    let dp = stm32f411::Peripherals::take().unwrap();

    // enable GPIOA
    let rcc = dp.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());

    // set PA5 as output
    let gpioa = dp.GPIOA;
    gpioa.moder.modify(|_, w| w.moder5().output());

    loop {
        // set PA5 high
        gpioa.odr.modify(|r, w| w.odr5().bit(!r.odr5().bit()));

        for _ in 0..800000 {
            cortex_m::asm::nop();
        }
    }
}
