#![no_std]
#![no_main]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;
use stm32f411_nucleo::uart::{self, uart2_print};

// NOTE: Read me:
// Development journey:
// first I tried using PA3 but then realised that it is connect
// to the virtual com port so cant be used for rx if the usb
// is plugged in. So this is why I had to change it to PA10
// Next: I had to stop using a blocking read because this would cause
// the GPS messages to stop being sent
// Next
//
//
//
// connect gps tx pin to pa10 stm32 rx pin

#[entry]
fn main() -> ! {
    let dp = stm32f411::Peripherals::take().unwrap();

    // gps neo6m baud rate
    uart::uart2_rx_tx_init(&dp, 9600);
    uart::uart1_rx_tx_init(&dp, 9600);
    let mut uart = uart::Uart::new(&dp.USART2);
    let mut buf = [0u8; 1];

    loop {
        // uart2_print(&dp.USART2, "working\n");
        // let b =
        // if let Some(b) = uart::uart1_try_read(&dp.USART1) {
        //     buf[0] = b;
        //     uart2_print(
        //         &dp.USART2,
        //         core::str::from_utf8(&buf).unwrap_or("\r\n?\r\n"),
        //     );
        // }

        match uart::uart1_try_read(&dp.USART1) {
            None => {} //uart2_print(&dp.USART2, "\r\nno input\r\n"),
            Some(b) => {
                buf[0] = b;
                uart2_print(
                    &dp.USART2,
                    core::str::from_utf8(&buf).unwrap_or("\r\n?\r\n"),
                );
            }
        }

        for _ in 0..1 {
            cortex_m::asm::nop();
        }
    }
}
