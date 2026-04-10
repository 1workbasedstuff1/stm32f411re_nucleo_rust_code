#![no_std]
use crate::i2c;
use crate::mpu6050;
use crate::uart;
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;

// MPU6050 I2C address when AD0 = GND
pub const MPU6050_ADDR: u8 = 0x68;

// Accelerometer output registers
pub const ACCEL_XOUT_H: u8 = 0x3B;
pub const ACCEL_XOUT_L: u8 = 0x3C;
pub const ACCEL_YOUT_H: u8 = 0x3D;
pub const ACCEL_YOUT_L: u8 = 0x3E;
pub const ACCEL_ZOUT_H: u8 = 0x3F;
pub const ACCEL_ZOUT_L: u8 = 0x40;

// Gyroscope output registers
pub const GYRO_XOUT_H: u8 = 0x43;
pub const GYRO_XOUT_L: u8 = 0x44;
pub const GYRO_YOUT_H: u8 = 0x45;
pub const GYRO_YOUT_L: u8 = 0x46;
pub const GYRO_ZOUT_H: u8 = 0x47;
pub const GYRO_ZOUT_L: u8 = 0x48;

// Configuration registers
pub const GYRO_CONFIG: u8 = 0x1B;
pub const ACCEL_CONFIG: u8 = 0x1C;
pub const PWR_MGMT_1: u8 = 0x6B;

// MPU6050 uses 100khz standard by default, can use 400khz
pub fn mpu6050_init(i2c1: &stm32f411::I2C1) {
    // Sorts issues upon reset
    let wake_data: u8 = 0x00; // PWR_MGMT_1 = 0 (wake)

    i2c::i2c1_burst_write(i2c1, MPU6050_ADDR, 0x6B, 1, &[wake_data]);

    let cfg: u8 = (1 << 3) | (0 << 4); // AFS_SEL = 01
                                       // Setting +- 4g resolution
    i2c::i2c1_burst_write(i2c1, MPU6050_ADDR, ACCEL_CONFIG, 1, &[cfg]);
    // Setting +- 500 degrees per second
    i2c::i2c1_burst_write(i2c1, MPU6050_ADDR, GYRO_CONFIG, 1, &[cfg]);
}

pub fn mpu6050_init_dbg(i2c1: &stm32f411::I2C1, uart: &mut uart::Uart) {
    write!(uart, "mpu: starting init\n").ok();

    let wake_data: u8 = 0x00;
    write!(
        uart,
        "mpu: sending wake command to 0x{:02X}\n",
        MPU6050_ADDR
    )
    .ok();
    write!(uart, "Sending address byte: {:x}\n", MPU6050_ADDR << 1);

    i2c::i2c1_burst_write_dbg(i2c1, MPU6050_ADDR, 0x6B, 1, &[wake_data], uart);
    write!(uart, "mpu: wake ok\n").ok(); // if this doesn't print, wake is the crash

    let cfg: u8 = (1 << 3) | (0 << 4);
    write!(uart, "mpu: sending accel config 0x{:02X}\n", cfg).ok();
    i2c::i2c1_burst_write_dbg(
        i2c1,
        MPU6050_ADDR,
        ACCEL_CONFIG,
        1,
        &[cfg],
        uart,
    );
    write!(uart, "mpu: accel config ok\n").ok(); // if this doesn't print, accel cfg is the crash

    write!(uart, "mpu: sending gyro config\n").ok();
    i2c::i2c1_burst_write_dbg(i2c1, MPU6050_ADDR, GYRO_CONFIG, 1, &[cfg], uart);
    write!(uart, "mpu: gyro config ok\n").ok();

    write!(uart, "mpu: init complete\n").ok();
}
