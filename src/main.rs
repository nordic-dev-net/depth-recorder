use ads1x1x::{channel, Ads1x1x, SlaveAddr};
use chrono::prelude::*;
use clap::Parser;
use embedded_hal::adc::OneShot;
use linux_embedded_hal::I2cdev;
use log::info;
use nb::block;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::thread::sleep;

#[derive(Parser)]
#[clap(version = "1.0", author = "Satu Koskinen")]
struct Cli {
    /// Output path for the depth data
    #[clap(required = true)]
    output_path: PathBuf,

    /// Recording interval in seconds
    #[clap()]
    interval: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct DepthData {
    timestamp: DateTime<Utc>,
    adc_value: i16,
    voltage: f32,
    pressure_psi: f32,
    depth_meters: f32,
}

fn calculate_depth(adc_value: i16) -> (f32, f32, f32) {
    // ADS1015 has 12-bit resolution, giving value range of [-2048,2047]
    // with ads1x1x crate in singleshot mode

    // Map ADC value to voltage in range 0-3.3V
    let adc_range_start: i16 = -2048;
    let adc_range_end: i16 = 2047;
    let voltage_range_start: f32 = 0.0;
    let voltage_range_end: f32 = 3.3;
    let mapped_voltage = (adc_value - adc_range_start) as f32
        * (voltage_range_end - voltage_range_start)
        / (adc_range_end - adc_range_start) as f32;

    // Map voltage in range 0-3.3V to pressure in range 0-100psi
    let pressure_range_start: f32 = 0.0;
    let pressure_range_end: f32 = 100.0;
    let mapped_pressure_psi = (mapped_voltage - voltage_range_start)
        * (pressure_range_end - pressure_range_start)
        / (voltage_range_end - voltage_range_start);

    // Convert pressure from psi to Pa, then calculate depth
    let pressure_pascal = mapped_pressure_psi * 689.47573;
    let saltwater_density = 1023.6;
    let gravity = 9.80665;
    let depth = pressure_pascal / (saltwater_density * gravity);
    return (mapped_voltage, mapped_pressure_psi, depth);
}

fn init_csv_writer(output_path: PathBuf) -> csv::Writer<File> {
    let filename = format!("{}_depth_data.csv", Utc::now().format("%Y-%m-%dT%H-%M-%S"));
    let file_path = output_path.join(filename);
    info!("Writing depth data to {}", file_path.display());
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)
        .expect("Failed to create and open file");
    csv::Writer::from_writer(file)
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    let interval = args.interval.unwrap_or(10);

    info!(
        "Start recording depth data to {} with {} second interval",
        args.output_path.display(),
        interval
    );

    let mut writer = init_csv_writer(args.output_path);

    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default(); // default address 0x48
    let mut adc = Ads1x1x::new_ads1015(dev, address);
    loop {
        let adc_value = block!(adc.read(&mut channel::SingleA0)).expect("Failed to read ADC");
        let (voltage, pressure_psi, depth_meters) = calculate_depth(adc_value);
        info!(
            "ADC value: {}, converted voltage: {}, calculated pressure: {}, calculated depth: {}",
            adc_value, voltage, pressure_psi, depth_meters
        );
        writer
            .serialize(DepthData {
                timestamp: Utc::now(),
                adc_value,
                voltage,
                pressure_psi,
                depth_meters,
            })
            .expect("Failed to serialize and write data to file");
        writer.flush().expect("Failed to flush writer");
        sleep(std::time::Duration::from_secs(interval));
    }
}
