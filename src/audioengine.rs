extern crate anyhow;
extern crate cpal;
use std::convert::From;
use std::error::Error;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct Host {
    pub host: cpal::Host,
    pub device: cpal::Device,
    pub config: cpal::SupportedStreamConfig,
}

pub fn init_host() -> Result<Host, Box<dyn Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config()?;

    let host_struct: Host = Host {
        host: host,
        device: device,
        config: config,
    };

    Ok(host_struct)
}

pub fn run<T>(host: &Host, freq: f32) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample,
{
    let channels = host.config.channels() as usize;

    let sample_rate: f32 = host.config.sample_rate().0 as f32;

    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;

        println!("Sample clock is {}", freq);
        (sample_clock * freq * 2.0 * 3.141592 / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = host.device.build_output_stream(
        &cpal::StreamConfig::from(host.clone().config.clone()),
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;

    Ok(stream)
}

pub fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        //println!("Playing with value {}",next_sample);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
