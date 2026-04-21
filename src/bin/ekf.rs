#![no_std]
#![no_main]
use core::fmt::Write;
use cortex_m_rt::entry;
use ekf_filter::quaternion_EKF::EkfBuilder::{
    self, EkfKalmanQuaternionBuilder,
};
use ekf_filter::quaternion_EKF::Measurement::{
    ImuBuildError, ImuBuilder, Measurement,
};
use nalgebra::SMatrix;
use panic_halt as _;
use stm32f4::stm32f411;
use stm32f411_nucleo::fpu;
use stm32f411_nucleo::i2c;
use stm32f411_nucleo::mpu6050;
use stm32f411_nucleo::qmc5883l;
use stm32f411_nucleo::quaternion_EKF::{read_data_into, unwrap_EKF_result};
use stm32f411_nucleo::uart::{self};

#[entry]
fn main() -> ! {
    let periph = stm32f411::Peripherals::take().unwrap();

    uart::uart2_init(&periph, 115200);
    fpu::fpu_init(&periph);

    let mut uart = uart::Uart::new(&periph.USART2);

    let measure_builder = ImuBuilder::new()
        .soft_iron_matrix(SMatrix::<f32, 3, 3>::new(
            1.91715093e-03,
            2.28003441e-05,
            -7.03602287e-05,
            2.28003441e-05,
            1.95164683e-03,
            -1.41819606e-06,
            -7.03602287e-05,
            -1.41819606e-06,
            1.95113017e-03,
        ))
        .hard_iron([34.82969897, -11.89332996, -43.86232264].into())
        .accel_scale_xyz(
            1.007776304854359,
            1.0066299908735399,
            1.0097825455175826,
        )
        .accel_offset_xyz(0.6405205, -0.097682, 0.634414)
        .gyro_offset_xyz(-86.83265, -28.607669, 61.9872929)
        .build();

    let mut measure = match measure_builder {
        Ok(model) => model,
        Err(error_enum) => match error_enum {
            ImuBuildError::MissingSoftIron => {
                write!(uart, "missing soft iron!\n").ok();
                loop {}
            }

            ImuBuildError::MissingHardIron => {
                write!(uart, "missing hard_iron!\n").ok();
                loop {}
            }

            ImuBuildError::MissingAccelScale => {
                write!(uart, "missing accel scale!\n").ok();
                loop {}
            }

            ImuBuildError::MissingAccelOffset => {
                write!(uart, "missing accel offset!\n").ok();
                loop {}
            }

            ImuBuildError::MissingGyroOffset => {
                write!(uart, "missing gyro offset!\n").ok();
                loop {}
            }
        },
    };

    write!(uart, "success\n").ok();

    i2c::i2c1_init(&periph);
    mpu6050::mpu6050_init(&periph.I2C1);
    qmc5883l::qmc5883_init(&periph.I2C1);

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

    loop {
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

        // put data into
        read_data_into(
            &mut measure,
            accel_x,
            accel_y,
            accel_z,
            gyro_x,
            gyro_y,
            gyro_z,
            mag_x,
            mag_y,
            mag_z,
        );

        let kalman_builder = EkfKalmanQuaternionBuilder::new()
            .x_state([1., 0., 0., 0.].into())
            .u_control([0., 0., 0.].into())
            .measurement_noise(0.5 * 0.5, 0.8 * 0.8)
            .gyro_noise(0.3 * 0.3)
            // to be adjusted
            .set_timing(0.01)
            .build();

        let kalman = unwrap_EKF_result(&mut uart, kalman_builder);

        measure.convert_and_callibrate();
    }
}
