#![no_std]
#![no_main]
use core::fmt::Write;
use cortex_m_rt::entry;
use nalgebra::{SMatrix, SVector};
use panic_halt as _;
use stm32f4::stm32f411;
use stm32f411_nucleo::uart;

type Vec3 = SVector<f32, 3>;
type Mat2x2 = SMatrix<f32, 2, 2>;

#[entry]
fn main() -> ! {
    let dp = stm32f411::Peripherals::take().unwrap();

    uart::uart2_init(&dp, 115200);
    let mut uart = uart::Uart::new(&dp.USART2);

    let mut count: u32 = 1;

    let x: Vec3 = Vec3::new(1.0, 0.0, 1.0);
    let y: Vec3 = Vec3::new(1.0, 0.0, 1.0);

    let id2x2 = Mat2x2::from([[1.0, 0.0], [0.0, 1.0]]);
    let half_2x2 = id2x2 * 0.5;
    // let id2x2 = Mat2x2::new(1.0, 0.0, 0.0, 1.0);

    write!(uart, "\r\ndot {}\r\n", x.dot(&y));
    write!(uart, "\r\ncross {}\r\n", x.cross(&y));
    write!(uart, "\r\nid2x2 {}\r\n", id2x2 * id2x2);
    write!(uart, "\r\nhalf2xd{}\r\n", half_2x2 * half_2x2);
    write!(uart, "\r\nid2x2 {}\r\n", id2x2 * id2x2);
    loop {
        // uart::uart2_print(&dp.USART2, "hello\n");
        // write!(uart, "count: {}\n", count).ok();
        count += 1;
        for _ in 0..1 {
            cortex_m::asm::nop();
        }
    }
}
