#![no_std]
#![no_main]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;
use stm32f411_nucleo::i2c;
use stm32f411_nucleo::mpu6050;
use stm32f411_nucleo::uart;

#[entry]
fn main() -> ! {
    let periph = stm32f411::Peripherals::take().unwrap();

    uart::uart2_init(&periph);
    let mut uart = uart::Uart::new(&periph.USART2);
    write!(uart, "uart init ok\n").ok();

    i2c::i2c1_init(&periph);
    write!(uart, "i2c init ok\n").ok();

    mpu6050::mpu6050_init_dbg(&periph.I2C1, &mut uart);
    write!(uart, "mpu init ok\n").ok();

    let mut high_accel_x: u8 = 0;
    let low_accel_x: u8 = 0;

    let mut data_high_x = [0];
    let mut data_low_x = [0];

    loop {
        // write!(uart, "working\n").ok();

        // i2c::i2c1_byte_read_dbg(
        //     &periph.I2C1,
        //     mpu6050::MPU6050_ADDR,
        //     mpu6050::ACCEL_XOUT_H,
        //     &mut data_low_x,
        //     &mut uart,
        // );

        // CORRECT - saddr is the MPU6050 I2C address, maddr is the register
        i2c::i2c1_byte_read(
            &periph.I2C1,
            mpu6050::MPU6050_ADDR,
            mpu6050::ACCEL_XOUT_L,
            &mut data_low_x,
        );

        i2c::i2c1_byte_read(
            &periph.I2C1,
            mpu6050::MPU6050_ADDR,
            mpu6050::ACCEL_XOUT_H,
            &mut data_high_x,
        );

        let accel_x: i16 = i16::from_be_bytes([data_high_x[0], data_low_x[0]]);
        write!(uart, "accel x: {}\n", accel_x).ok();

        // write!(uart, "accel x: {}", accel_x).ok();
        // write!(uart, "data low x: {}", data_low_x[0]).ok();
    }
}
