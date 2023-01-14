use std::{process, sync::{Arc, Mutex}, time::Duration};
use std::borrow::{Borrow, BorrowMut};
use std::io::empty;
use std::mem::take;
use std::ptr::{NonNull, null};

use cpal::{ChannelCount, SampleFormat, Stream, traits::{DeviceTrait, HostTrait, StreamTrait}};
use dasp::{Sample, sample::ToSample};
use tts::{Error, Tts};

use july_lib::{DecodingState, Model, Recognizer, SpeakerModel};

pub struct Assistant {
    tts: Tts,
    in_stream: Option<Stream>,
}

impl Assistant {
    pub fn new() -> Self {
        let mut tts = Tts::default().unwrap();

        Self {
            tts,
            in_stream: None,
        }
    }

    pub fn init_vosk_model(
        model_path: &str,
        speaker_model_path: &str) -> Stream {
        let audio_host = cpal::default_host();
        let audio_input_device = audio_host
            .default_input_device()
            .expect("No input device connected");
        let audio_input_config = audio_input_device
            .default_input_config()
            .expect("Failed to load default input config");
        let channels = audio_input_config.channels();

        let model = Model::new(model_path).expect("Could not create the model");
        let spk_model =
            SpeakerModel::new(speaker_model_path).expect("Could not create the speaker model");

        let recognizer = Recognizer::new_with_speaker(
            &model,
            audio_input_config.sample_rate().0 as f32,
            &spk_model,
        )
            .expect("Could not create the Recognizer");

        let recognizer = Arc::new(Mutex::new(recognizer));

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let recognizer_clone = recognizer.clone();

        let in_stream = match audio_input_config.sample_format() {
            SampleFormat::F32 => audio_input_device.build_input_stream(
                &audio_input_config.into(),
                move |data: &[f32], _| Self::recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
                err_fn,
            ),
            SampleFormat::U16 => audio_input_device.build_input_stream(
                &audio_input_config.into(),
                move |data: &[u16], _| Self::recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
                err_fn,
            ),
            SampleFormat::I16 => audio_input_device.build_input_stream(
                &audio_input_config.into(),
                move |data: &[i16], _| Self::recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
                err_fn,
            ),
        }
            .expect("Could not build input stream");

        return in_stream;
    }

    fn stop(mut self) {
        drop(self.tts);
        drop(self.in_stream);
    }
}

trait Listen {
    fn recognize<T: Sample + ToSample<i16>>(recognizer: &mut Recognizer, data: &[T], channels: ChannelCount);
    fn stereo_to_mono(input_data: &[i16]) -> Vec<i16>;
}

impl Listen for Assistant {
    fn recognize<T: Sample + ToSample<i16>>(
        recognizer: &mut Recognizer,
        data: &[T],
        channels: ChannelCount,
    ) {
        let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
        let data = if channels != 1 {
            Self::stereo_to_mono(&data)
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
                Assistant::process_message(
                    (recognizer.final_result().single().unwrap().text)
                        .parse()
                        .unwrap(),
                );
                return;
            }
            DecodingState::Failed => println!("error"),
        }
    }

    fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
        let mut result = Vec::with_capacity(input_data.len() / 2);
        result.extend(
            input_data
                .chunks_exact(2)
                .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
        );
        result
    }
}

pub trait Response {
    fn process_message(message: String);
    fn write_to_speak(&mut self, text: Box<str>) -> Result<(), Error>;
}

impl Response for Assistant {
    fn process_message(message: String) {
        println!("{}", message);
        if message.contains("take a rest july") {
            process::exit(0x0100);
        }
    }

    fn write_to_speak(&mut self, text: Box<str>) -> Result<(), Error> {
        let _ = self.tts.speak(text, false)?;
        Ok(())
    }
}

