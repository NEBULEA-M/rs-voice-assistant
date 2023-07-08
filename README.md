Download model and put into directory ./model from [VOSK Model](https://alphacephei.com/vosk/models)

# Text-to-speech package
```shell
sudo apt-get install libspeechd-dev

sudo apt-get install libclang-dev
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

## For execution
The libraries also have to be discoverable by the executable at runtime. You will have to follow one of the approaches in the same section you chose in compilation.

### Windows and Linux (Recommended)

For both approaches, you will need to copy the libraries to the root of the executable (target/<cargo profile name> by default). It is recommended that you use a tool such as cargo-make to automate moving the libraries from another, more practical, directory to the destination during build.

### Windows-only
If you added your libraries to a directory in your PATH, no extra steps are needed as long as that is also the case for the target machine.

### Linux-only
If you followed option 1 in the compilation section: No extra steps are needed as long as the target machine also has the libraries in one of the mentioned directories.
If you followed option 2: You will need to add the directory containing the libraries to the LD_LIBRARY_PATH environment variable. Note that this directory does not have to be the same added to LIBRARY_PATH in the compilation step.