use std::{
    env,
    sync::{Arc, Mutex},
    time::Duration,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    ChannelCount, SampleFormat,
};
use dasp::{sample::ToSample, Sample};
use serde_json::to_string;

use july::{DecodingState, Model, Recognizer};

static  mut IS_EXIT: bool = false;

fn main() {
    let model_path = "./model";

    let record_duration = Duration::from_secs(20);
    let rest_duration = Duration::from_secs(10);

    let audio_input_device = cpal::default_host()
        .default_input_device()
        .expect("No input device connected");

    let config = audio_input_device
        .default_input_config()
        .expect("Failed to load default input config");
    let channels = config.channels();

    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer = Recognizer::new(&model, config.sample_rate().0 as f32)
        .expect("Could not create the Recognizer");

    // recognizer.set_words(true);
    // recognizer.set_partial_words(true);

    let recognizer = Arc::new(Mutex::new(recognizer));

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let recognizer_clone = recognizer.clone();
    let stream = match config.sample_format() {
        SampleFormat::F32 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
            err_fn,
        ),
        SampleFormat::U16 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[u16], _| recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
            err_fn,
        ),
        SampleFormat::I16 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
            err_fn,
        ),
    }
        .expect("Could not build stream");

    stream.play().expect("Could not play stream");
    println!("July is on...");

    while true {    // use while instead of loop to work with this situation, don't know why

        unsafe {
            if IS_EXIT {
                break;
            }
        }
        std::thread::sleep(record_duration);
        stream.pause().unwrap();
        println!("July is pause");
        std::thread::sleep(rest_duration);
        stream.play().expect("Could not play stream");
        println!("July is on again...");
    }

    drop(stream);
}

fn recognize<T: Sample + ToSample<i16>>(
    recognizer: &mut Recognizer,
    data: &[T],
    channels: ChannelCount
) {
    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
    let data = if channels != 1 {
        stereo_to_mono(&data)
    } else {
        data
    };

    let state = recognizer.accept_waveform(&data);
    match state {
        DecodingState::Running => {
            // println!("partial: {:#?}", recognizer.partial_result());
        }
        DecodingState::Finalized => {
            // println!("result: {:#?}", recognizer.final_result().single().unwrap().text);
            processing((recognizer.final_result().single().unwrap().text).parse().unwrap());

        }
        DecodingState::Failed => println!("error"),
    }
}

pub fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
    let mut result = Vec::with_capacity(input_data.len() / 2);
    result.extend(
        input_data
            .chunks_exact(2)
            .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
    );
    result
}

fn processing(message: String) {
    println!("{}", message);
    if message.contains("take a rest july") {
        unsafe { IS_EXIT = true; }
    }
}