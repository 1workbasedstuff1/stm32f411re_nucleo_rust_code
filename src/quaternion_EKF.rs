use crate::{
    i2c,
    uart::{Uart, uart2_print},
};
use core::fmt::Write;
use ekf_filter::quaternion_EKF::{
    EkfBuilder::EkfBuilderError,
    EkfKalman::EkfKalmanQuaternion,
    Measurement::{self, Imu9DofMeasure},
};
use nalgebra::{self, RealField, SMatrix, SVector};
use stm32f4::stm32f411;

pub fn read_data_into(
    measure: &mut Imu9DofMeasure,
    raw_accel_x: i16,
    raw_accel_y: i16,
    raw_accel_z: i16,
    raw_gyro_x: i16,
    raw_gyro_y: i16,
    raw_gyro_z: i16,
    raw_mag_x: i16,
    raw_mag_y: i16,
    raw_mag_z: i16,
) {
    measure.raw_accel_x = raw_accel_x;
    measure.raw_accel_y = raw_accel_y;
    measure.raw_accel_z = raw_accel_z;

    measure.raw_gyro_x = raw_gyro_x;
    measure.raw_gyro_y = raw_gyro_y;
    measure.raw_gyro_z = raw_gyro_z;

    measure.raw_mag_x = raw_mag_x;
    measure.raw_mag_y = raw_mag_y;
    measure.raw_mag_z = raw_mag_z;
}
pub fn unwrap_EKF_result(
    uart: &mut Uart<'_>,
    ekf_build: Result<EkfKalmanQuaternion, EkfBuilderError>,
) -> EkfKalmanQuaternion {
    match ekf_build {
        Ok(ekf_filter) => ekf_filter,
        Err(build_error) => match build_error {
            EkfBuilderError::MissingStateX => {
                write!(uart, "missing state x\n").ok();
                loop {}
            }
            EkfBuilderError::MissingControlU => {
                write!(uart, "missing control u\n").ok();
                loop {}
            }
            EkfBuilderError::MissingGyroNoise => {
                write!(uart, "missing gyro noise\n").ok();
                loop {}
            }
            EkfBuilderError::MissingMeasurementNoise => {
                write!(uart, "missing measurement noise\n").ok();
                loop {}
            }
            EkfBuilderError::MissingTiming => {
                write!(uart, "missing timing\n").ok();
                loop {}
            }
        },
    }
}
