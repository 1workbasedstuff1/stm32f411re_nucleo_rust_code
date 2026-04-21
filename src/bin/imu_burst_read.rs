#![no_std]
#![no_main]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;
use stm32f411_nucleo::i2c;
use stm32f411_nucleo::mpu6050;
use stm32f411_nucleo::qmc5883l;
use stm32f411_nucleo::tim2;
use stm32f411_nucleo::uart;

#[entry]
fn main() -> ! {
    let periph = stm32f411::Peripherals::take().unwrap();

    uart::uart2_init(&periph, 115200);
    let mut uart = uart::Uart::new(&periph.USART2);
    write!(uart, "uart init ok\n").ok();

    i2c::i2c1_init(&periph);
    write!(uart, "i2c init ok\n").ok();

    mpu6050::mpu6050_init_dbg(&periph.I2C1, &mut uart);
    write!(uart, "mpu init ok\n").ok();

    qmc5883l::qmc5883_init(&periph.I2C1);

    // ACCEL DATA

    let mut accel_x: i16;
    let mut accel_y: i16;
    let mut accel_z: i16;

    let mut gyro_x: i16;
    let mut gyro_y: i16;
    let mut gyro_z: i16;

    let mut raw_imu_data = [0u8; 14];

    let mut mag_x: i16;
    let mut mag_y: i16;
    let mut mag_z: i16;

    let mut raw_mag = [0u8; 6];

    tim2::tim2_us_init(&periph);

    let mut t0 = tim2::micros(&periph.TIM2);
    let mut t1 = tim2::micros(&periph.TIM2);
    let mut loop_timting: u32;

    loop {
        t1 = tim2::micros(&periph.TIM2);
        loop_timting = t1 - t0;
        t0 = tim2::micros(&periph.TIM2);

        i2c::i2c1_burst_read(
            &periph.I2C1,
            mpu6050::MPU6050_ADDR,
            mpu6050::ACCEL_XOUT_H,
            &mut raw_imu_data,
        );

        i2c::i2c1_burst_read(
            &periph.I2C1,
            qmc5883l::QMC5883_ADDR,
            qmc5883l::QMC5883_XOUT_L,
            &mut raw_mag,
        );

        accel_x = i16::from_be_bytes([raw_imu_data[0], raw_imu_data[1]]);
        accel_y = i16::from_be_bytes([raw_imu_data[2], raw_imu_data[3]]);
        accel_z = i16::from_be_bytes([raw_imu_data[4], raw_imu_data[5]]);

        gyro_x = i16::from_be_bytes([raw_imu_data[8], raw_imu_data[9]]);
        gyro_y = i16::from_be_bytes([raw_imu_data[10], raw_imu_data[11]]);
        gyro_z = i16::from_be_bytes([raw_imu_data[12], raw_imu_data[13]]);

        mag_x = i16::from_be_bytes([raw_mag[1], raw_mag[0]]);
        mag_y = i16::from_be_bytes([raw_mag[3], raw_mag[2]]);
        mag_z = i16::from_be_bytes([raw_mag[5], raw_mag[4]]);

        write!(
            uart,
            "MX: {}, MY: {}, MZ: {}, AX: {}, AY: {}, AZ: {}, gX: {}, gY: {}, gZ: {}, t: {}\n",
            mag_x, mag_y, mag_z,
            accel_x, accel_y, accel_z,
            gyro_x, gyro_y, gyro_z, loop_timting
        )
        .ok();

        // write!(uart, "accel x: {}", accel_x).ok();
        // write!(uart, "data low x: {}", data_low_x[0]).ok();
    }
}
