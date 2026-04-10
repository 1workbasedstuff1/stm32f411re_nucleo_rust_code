#![no_std]
use crate::i2c;
use crate::mpu6050;
use crate::uart;
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;

//Registers
pub const QMC5883_ADDR: u8 = 0x2C;

//Registers
pub const QMC5883_CHIP_ID: u8 = 0x00;
pub const QMC5883_XOUT_L: u8 = 0x01;
pub const QMC5883_XOUT_H: u8 = 0x02;
pub const QMC5883_YOUT_L: u8 = 0x03;
pub const QMC5883_YOUT_H: u8 = 0x04;
pub const QMC5883_ZOUT_L: u8 = 0x05;
pub const QMC5883_ZOUT_H: u8 = 0x06;
pub const QMC5883_STATUS: u8 = 0x09;
pub const QMC5883_CTRL1: u8 = 0x0A;
pub const QMC5883_CTRL2: u8 = 0x0B;

// Status Bits
pub const QMC5883_DRDY: u8 = 1 << 0;
pub const QMC5883_OVFL: u8 = 1 << 1;

// Ctrl Bits
pub const QMC5883_MODE_SUSPEND: u8 = 0x00 << 0;
pub const QMC5883_MODE_NORMAL: u8 = 0x01 << 0;
pub const QMC5883_MODE_SINGLE: u8 = 0x02 << 0;
pub const QMC5883_MODE_CONTINUOUS: u8 = 0x03 << 0;

pub const QMC5883_ODR_10HZ: u8 = 0x00 << 2;
pub const QMC5883_ODR_50HZ: u8 = 0x01 << 2;
pub const QMC5883_ODR_100HZ: u8 = 0x02 << 2;
pub const QMC5883_ODR_200HZ: u8 = 0x03 << 2;

pub const QMC5883_OSR1_8: u8 = 0x00 << 4;
pub const QMC5883_OSR1_4: u8 = 0x01 << 4;
pub const QMC5883_OSR1_2: u8 = 0x02 << 4;
pub const QMC5883_OSR1_1: u8 = 0x03 << 4;

pub const QMC5883_OSR2_1: u8 = 0x00 << 6;
pub const QMC5883_OSR2_2: u8 = 0x01 << 6;
pub const QMC5883_OSR2_4: u8 = 0x02 << 6;
pub const QMC5883_OSR2_8: u8 = 0x03 << 6;

pub const QMC5883_SOFT_RST: u8 = 1 << 7;
pub const QMC5883_SELF_TEST: u8 = 1 << 6;

pub const QMC5883_RNG_30G: u8 = 0x00 << 2;
pub const QMC5883_RNG_12G: u8 = 0x01 << 2;
pub const QMC5883_RNG_8G: u8 = 0x02 << 2;
pub const QMC5883_RNG_2G: u8 = 0x03 << 2;

pub const QMC5883_SETRESET_ON: u8 = 0x00 << 0;
pub const QMC5883_SET_ONLY_ON: u8 = 0x01 << 0;
pub const QMC5883_SETRESET_OFF: u8 = 0x02 << 0;

pub fn qmc5883_init(i2c: &stm32f411::I2C1) {
    // set bit on CTRL 2
    i2c::i2c1_burst_write(
        &i2c,
        QMC5883_ADDR,
        QMC5883_CTRL2,
        1,
        &[QMC5883_SOFT_RST],
    );

    /* --- CTRL2 --- */
    // RNG = 8 Gauss (10)
    // Set/Reset = ON (00)
    let mut cfg: u8;
    cfg = (2 << 2) | (0 << 0);
    i2c::i2c1_burst_write(i2c, QMC5883_ADDR, QMC5883_CTRL2, 1, &[cfg]);

    /* ---------- CTRL1 ---------- */
    // OSR2 = 11 (8)
    // OSR1 = 11 (1)
    // ODR  = 10 (100Hz)
    // MODE = 11 (Continuous)
    cfg = (3 << 6) |   // OSR2
          (3 << 4) |   // OSR1
          (2 << 2) |   // ODR
          (3 << 0); // MODE (continuous)

    i2c::i2c1_burst_write(i2c, QMC5883_ADDR, QMC5883_CTRL1, 1, &[cfg]);
}

// #endif
