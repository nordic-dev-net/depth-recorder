use ads1x1x::{channel, Ads1x1x, SlaveAddr};
use chrono::prelude::*;
use clap::Parser;
use embedded_hal::adc::OneShot;
use linux_embedded_hal::I2cdev;
use log::{info};
use nb::block;
use serde::{Deserialize, Serialize};
use std::fs::{OpenOptions, File};
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
    voltage: f32,
    pressure: f32,
    depth: f32,
}

fn calculate_depth(voltage: i16) -> (f32, f32, f32) {
    // ADS1015 has 12-bit resolution, giving value range of [-2048,2047]
    // with ads1x1x crate in singleshot mode
    let input_range_start: i16 = -2048;
    let input_range_end: i16 = 2047;
    // Map ADC value to voltage in range 0-3.3V
    let output_range_start: f32 = 0.0;
    let output_range_end: f32 = 3.3;
    let mapped_voltage = (voltage - input_range_start) as f32
        * (output_range_end - output_range_start)
        / (input_range_end - input_range_start) as f32;
    // TODO: implement mapping from voltage to pressure
    let mapped_pressure = mapped_voltage;
    // TODO: implement mapping from pressure to depth
    let saltwater_density = 1023.6;
    let gravity = 9.80665;
    let depth = mapped_pressure / (saltwater_density * gravity);
    return (mapped_voltage, mapped_pressure, depth);
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
        let value = block!(adc.read(&mut channel::SingleA0)).expect("Failed to read ADC");
        let (voltage, pressure, depth) = calculate_depth(value);
        info!(
            "ADC value: {}, converted voltage: {}, calculated pressure: {}, calculated depth: {}",
            value, voltage, pressure, depth
        );
        let _ = writer.serialize(DepthData {
            timestamp: Utc::now(),
            voltage: voltage,
            pressure: pressure,
            depth: depth,
        }).expect("Failed to serialize and write data to file");
        writer.flush().expect("Failed to flush writer");
        sleep(std::time::Duration::from_secs(interval));
    }
    // get I2C device back
    let _dev = adc.destroy_ads1015();
}
