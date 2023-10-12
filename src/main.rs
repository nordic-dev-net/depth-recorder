use ads1x1x::{channel, Ads1x1x, SlaveAddr};
use chrono::prelude::*;
use clap::Parser;
use embedded_hal::adc::OneShot;
use linux_embedded_hal::I2cdev;
use log::{debug, error, info, warn};
use nb::block;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
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
    voltage: i16,
    depth: f32,
}

fn calculate_depth(voltage: i16) -> f32 {
    let voltage = voltage as f32;
    let depth = 0.0;
    return depth;
}

fn main() {
    env_logger::init();
    let args = Cli::parse();

    let interval = args.interval.unwrap_or(10);
    info!("Recording interval: {} seconds", interval);

    let filename = format!("{}_depth_data.csv", Utc::now().format("%Y-%m-%dT%H-%M-%S"));
    let file_path = args.output_path.join(filename);
    info!("Writing depth data to {}", file_path.display());
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)
        .expect("Failed to create and open file");
    let mut writer = csv::Writer::from_writer(file);

    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let mut adc = Ads1x1x::new_ads1015(dev, address);
    loop {
        let voltage = block!(adc.read(&mut channel::SingleA0)).expect("Failed to read ADC");
        let depth = calculate_depth(voltage);
        info!("Voltage: {}, calculated depth: {}", voltage, depth);
        writer.serialize(DepthData {
            timestamp: Utc::now(),
            voltage: voltage,
            depth: 0.0,
        });
        writer.flush().expect("Failed to flush writer");
        sleep(std::time::Duration::from_secs(interval));
    }
    // get I2C device back
    let _dev = adc.destroy_ads1015();
}
