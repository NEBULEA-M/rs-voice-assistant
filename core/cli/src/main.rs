extern crate cpal;
extern crate july_common;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use std::string::ToString;

use cpal::Stream;
use cpal::traits::StreamTrait;

use july_common::assistant::{Assistant, Response};

fn main() {
    let model_path = "../models/vosk-model-en-us";
    let speaker_model_path = "../models/vosk-speaker-model";

    let mut assistant = Assistant::new();

    assistant.write_to_speak(Box::from("hello sir, july is on, initializing environment...")).unwrap();

    let record_duration = Duration::from_secs(50);
    let idle_duration = Duration::from_secs(10);

    let mut in_stream = Assistant::init_vosk_model(model_path, speaker_model_path);
    in_stream.play().expect("Could not play input stream");

    println!("July is on...");

    loop {
        std::thread::sleep(record_duration);
        in_stream.pause().unwrap();
        println!("July is pause");
        std::thread::sleep(idle_duration);
        in_stream.play().expect("Could not play stream");
        println!("July is on again...");
    }
}
