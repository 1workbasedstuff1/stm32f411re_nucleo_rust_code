#![no_std]
#![no_main]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;
use stm32f411_nucleo::gy37y3530;
use stm32f411_nucleo::uart;

#[entry]
fn main() -> ! {
    let dp = stm32f411::Peripherals::take().unwrap();

    gy37y3530::motor_setup(&dp);
    uart::uart2_init(&dp);
    let mut uart = uart::Uart::new(&dp.USART2);

    // Read initial state of PA2/PA3 for encoder
    let initial_state = gy37y3530::motor_read(&dp.GPIOA);

    // Create encoder with raw ticks
    let mut encoder = gy37y3530::Encoder::new(initial_state, 2096);

    write!(uart, "loop begin\n").ok();
    write!(uart, "setup finished\n").ok();

    loop {
        // Read current pins
        let current_state = gy37y3530::motor_read(&dp.GPIOA);

        // Update encoder with new state
        encoder.update(current_state);

        // Print raw tick count
        write!(
            uart,
            "PA2, PA3 = ({}, {}), ticks = {}\n",
            current_state.0,
            current_state.1,
            encoder.ticks()
        )
        .ok();

        // Small delay to avoid spamming UART too fast
        for _ in 0..1 {
            cortex_m::asm::nop();
        }
    }
}
