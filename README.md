Download model and put into directory ./model from [VOSK Model](https://alphacephei.com/vosk/models)

# Text-to-speech package
```shell
sudo apt-get install libspeechd-dev
```

# Sound package
```shell
sudo apt-get install libasound2-dev

sudo apt-get install libsdl2-dev
```

# Adapt VOSK Model
Open root location of kaldi
```shell
export KALDI_ROOT=`pwd`/kaldi
git clone https://github.com/kaldi-asr/kaldi
cd kaldi/tools
make
# install all required dependencies and repeat `make` if needed
extras/install_opengrm.sh
```

```shell
export PATH=$KALDI_ROOT/tools/openfst/bin:$PATH
export LD_LIBRARY_PATH=$KALDI_ROOT/tools/openfst/lib/fst
```

Use model directory
```shell
cd model/graph
fstsymbols --save_osymbols=words.txt Gr.fst > /dev/null
farcompilestrings --fst_type=compact --symbols=words.txt --keep_symbols text.txt | \
 ngramcount | ngrammake | \
 fstconvert --fst_type=ngram > Gr.new.fst
mv Gr.new.fst Gr.fst
```