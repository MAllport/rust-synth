mod audioengine;
mod midi;
mod synth;
extern crate cpal;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::f32::consts::PI;
use std::sync::{Arc,Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const AMPLITUDE: f32 = 0.25;

#[derive(Clone, Copy)]
pub enum WaveFunction <'a> {
    Sine(&'a f32),
    Square(&'a f32),
    Triangle(&'a f32),
    Sawtooth(&'a f32),
    SineMag(&'a f32),
}

impl <'a> WaveFunction <'a> {
    pub fn val(&'a self, sample_clock: f32, sample_rate: f32, frequency: f32) -> f32 {
        let f_x = sample_clock as f32 * frequency / sample_rate as f32;
        AMPLITUDE * match self {
            &WaveFunction::Sine(f32) => (2.0 * PI * f_x).sin(),
            &WaveFunction::Square(f32) => (-1.0f32).powf((2.0 * f_x).floor()),
            &WaveFunction::Triangle(f32) => 1.0 - 4.0 * (0.5 - (f_x + 0.25).fract()).abs(),
            &WaveFunction::Sawtooth(f32) => 2.0 * f_x.fract() - 1.0,
            &WaveFunction::SineMag(f32) => 2.0 * (PI * f_x).sin().abs() - 1.0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct WaveGen<'a> {
    function: &'a WaveFunction<'a>,
    sample_rate: f32,
    sample_clock: f32,
    frequency: f32
}

impl <'a> WaveGen<'a> {
    pub fn new(function: &'a WaveFunction<'a>, sample_rate: &'a f32, frequency: &'a f32) -> Self {
        Self {
            function: function,
            sample_rate: *sample_rate,
            sample_clock: 0.0,
            frequency : *frequency
        }
    }

    pub fn get_freq(&'a self) -> f32 {
        self.frequency
    }

    pub fn change_freq(&'a mut self, frequency: f32){
        self.sample_clock = 0.0;
        self.frequency    = frequency;
    }

    pub fn step(&'a mut self) -> f32 {
        let v = self.function.val(self.sample_clock, self.sample_rate,self.frequency);
        self.sample_clock = (self.sample_clock + 1.0) % self.sample_rate;
        v
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_conn = midi::initMidi().unwrap();
    let host = audioengine::init_host().unwrap();

    let midi_input: midir::MidiInput = input_conn.0;
    let midi_input_port: midir::MidiInputPort = input_conn.1;


    let channels = host.config.channels() as usize;

    let sample_rate: f32 = host.config.sample_rate().0 as f32;

    let wave = Arc::new(Mutex::new(WaveGen::new(&WaveFunction::Sine(&400.0), &sample_rate, &400.0)));

    let _conn_in : midir::MidiInputConnection<()> = midi_input.connect(&midi_input_port, "midir-read-input", |stamp,message,_trash| {
        let mut wave2 = wave.clone();
        wave2.lock().unwrap().change_freq(synth::handle_midi_message(stamp,message,_trash).unwrap());
        //println!("Frequency is {}", freq);
    }, ()).unwrap() as midir::MidiInputConnection<()>;
    
    let mut next_value = || {
        wave.lock().unwrap().get_freq()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = host.device.build_output_stream(
        &cpal::StreamConfig::from(host.config),
        |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data(data, &channels, &mut next_value)
        },
        err_fn,
    )?;

    stream.play()?;

    let mut dummy = String::new();
    stdin().read_line(&mut dummy)?; // wait for next enter key press
    Ok(())
}

pub fn create_conn((midi_input, midi_input_port) : (midir::MidiInput,midir::MidiInputPort), wave: std::sync::Arc<std::sync::Mutex<WaveGen>>) -> midir::MidiInputConnection<()>{
    return midi_input.connect(&midi_input_port, "midir-read-input", |stamp,message,_trash| {
        let mut wave2 = wave.clone();
        wave2.lock().unwrap().change_freq(synth::handle_midi_message(stamp,message,_trash).unwrap());
        //println!("Frequency is {}", freq);
    }, ()).unwrap() as midir::MidiInputConnection<()>;
}

pub fn write_data<'a, 'b: 'a, T>(
    output: &mut [T],
    channels: &'b usize,
    next_sample: &'b mut dyn FnMut() -> f32,
) where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(*channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        //println!("Playing with value {}",next_sample);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
