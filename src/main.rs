use std::{sync::{Arc, Mutex}, time::Duration};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    ChannelCount, SampleFormat,
};
use dasp::{sample::ToSample, Sample};
use tts::{Error, Tts};


use july::{DecodingState, Model, Recognizer, SpeakerModel};

static mut IS_EXIT: bool = false;

fn main() -> Result<(), Error> {
    let mut tts = Tts::default()?;

    write_to_speak(&mut tts, Box::from("hello sir, july is on")).unwrap();

    let model_path = "./model";
    let speaker_model_path = "./speaker-model";

    let record_duration = Duration::from_secs(50);
    let idle_duration = Duration::from_secs(10);

    let audio_host = cpal::default_host();
    let audio_input_device = audio_host
        .default_input_device()
        .expect("No input device connected");
    let audio_input_config = audio_input_device
        .default_input_config()
        .expect("Failed to load default input config");
    let channels = audio_input_config.channels();

    let model = Model::new(model_path).expect("Could not create the model");
    let spk_model = SpeakerModel::new(speaker_model_path).expect("Could not create the speaker model");

    let mut recognizer =
        Recognizer::new_with_speaker(&model, audio_input_config.sample_rate().0 as f32, &spk_model)
        .expect("Could not create the Recognizer");

    let recognizer = Arc::new(Mutex::new(recognizer));

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let recognizer_clone = recognizer.clone();
    let in_stream = match audio_input_config.sample_format() {
        SampleFormat::F32 => audio_input_device.build_input_stream(
            &audio_input_config.into(),
            move |data: &[f32], _| recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
            err_fn,
        ),
        SampleFormat::U16 => audio_input_device.build_input_stream(
            &audio_input_config.into(),
            move |data: &[u16], _| recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
            err_fn,
        ),
        SampleFormat::I16 => audio_input_device.build_input_stream(
            &audio_input_config.into(),
            move |data: &[i16], _| recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
            err_fn,
        ),
    }
        .expect("Could not build input stream");

    in_stream.play().expect("Could not play input stream");
    println!("July is on...");

    loop {
        unsafe {
            if IS_EXIT {
                write_to_speak(&mut tts, Box::from("got it, goodbye sir")).unwrap();
                println!("July is turn off");
                drop(tts);
                drop(in_stream);
                break;
            } else {
                std::thread::sleep(record_duration);
                in_stream.pause().unwrap();
                println!("July is pause");
                std::thread::sleep(idle_duration);
                in_stream.play().expect("Could not play stream");
                println!("July is on again...");
            }
        }
    }
    Ok(())
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
            process_message((recognizer.final_result().single().unwrap().text).parse().unwrap());
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

fn process_message(message: String) {
    println!("{}", message);
    if message.contains("take a rest july") {
        unsafe { IS_EXIT = true; }
    }
}

fn write_to_speak(tts: &mut Tts, text: Box<str>) -> Result<(), Error> {
    let _ = &tts.speak(text, false)?;
    Ok(())
}