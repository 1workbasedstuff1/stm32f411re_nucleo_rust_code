#![no_std]
#![no_main]
use core::fmt::Write;
use core::hint::black_box;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;
use stm32f411_nucleo::fpu;
use stm32f411_nucleo::tim2;
use stm32f411_nucleo::uart;

#[entry]
fn main() -> ! {
    let periph = stm32f411::Peripherals::take().unwrap();

    uart::uart2_init(&periph);
    fpu::fpu_init(&periph);
    tim2::tim2_us_init(&periph);
    // fpu::fpu_disable(&periph);

    let mut uart = uart::Uart::new(&periph.USART2);

    let mut begin: u32 = 0;
    let mut end: u32 = 0;
    let mut time: u32 = 0;
    let mut output1 = 0.0f32;

    loop {
        let test1: f32 = 0.33333;
        let test2: f32 = 9.000;

        let mut output2 = 0.0f32;

        begin = tim2::micros(&periph.TIM2);
        for i in 0..10_000 {
            // output1 += test1 * test2;
            // output2 += black_box(test1) * black_box(test2);
            // output2 = black_box(output2 + test1 * test2)
            fpu::fpu_multiply_accumulate(output2, test1, test2);
        }

        end = tim2::micros(&periph.TIM2);
        time = end - begin;

        write!(uart, "time {}, result {}\n", time, output2).ok();
        output2 += 1.0;

        fpu::fpu_multiply_accumulate(output2, test1, output2);
    }
}
