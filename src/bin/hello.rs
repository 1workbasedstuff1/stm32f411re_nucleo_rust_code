#![no_std]
#![no_main]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;
use stm32f411_nucleo::uart;

#[entry]
fn main() -> ! {
    let dp = stm32f411::Peripherals::take().unwrap();

    uart::uart2_init(&dp);
    let mut uart = uart::Uart::new(&dp.USART2);

    let mut count: u32 = 1;

    loop {
        // uart::uart2_print(&dp.USART2, "hello\n");
        write!(uart, "count: {}\n", count).ok();
        count += 1;
        for _ in 0..1 {
            cortex_m::asm::nop();
        }
    }
}
