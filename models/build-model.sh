#!/bin/sh

echo "Downloading speech model"
wget -O vosk-vosk-model-self-en-us.zip https://alphacephei.com/vosk/models/vosk-model-en-us-0.42-gigaspeech.zip
unzip vosk-vosk-model-self-en-us.zip

echo "Downloading speaker model"
wget -O vosk-model-spk https://alphacephei.com/vosk/models/vosk-model-spk-0.4.zip
unzip vosk-model-spk.zip