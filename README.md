# MaschineMK2_linux
**MaschineMK2_linux is an open-source program that allows you to (possibly) use your Maschine MK2 device on linux**

Works on atleast ubuntu and fedora, but most distros that ship alsa, jack and QjackCtl or similar should work.

Forked from https://github.com/wrl/maschine.rs
Without the maschine.rs project I would not have a working MIDI controller right now!


# Features
- Functional midi pads
- Buttons as midi cc so you can map them into a DAW or something
- Encoders as midi cc (not quite 100% functional)
- Lights
- Picture on the screen
- The same OSC idea from the original maschine.rs

**ABSOLUTELY TESTING** sequencer mode:
- press Shift+Padmode twice to activate.
- Press pads to activate them for the sequence, they light up when active.
- ~~- Shift+Small roller 1 controls the speed of the sequencer~~ **Under construction**
- Holding shift and tapping a pad, then tapping another pad changes the note of the first pad.
- While in sequencer mode, the play button starts the sequencer.

# Building
to build MaschineMK2_linux you will need rustc and cargo.
to install rust, go to: https://www.rust-lang.org/tools/install

build:
``` sh
git clone https://github.com/SnovaxZ/MaschineMK2_linux.git
cd MaschineMK2_linux
./build.sh
```

the *build.sh* just runs cargo build and moves the test picture into the release directory.

*if you can not run the build program you might need to make sure it is executable*

**Other stuff You probably need:**
Pipewire also works
 - Alsa   *On some distros you may need the alsa-lib-devel package or similar*
 - Jack
 - Patchance, Qjackctl or any similar program.
 

# Use
First: figure out which hidraw path your maschine uses. (*The run.sh does this for you now!*)

Second (optional): change the udev rules so you can run without sudo.

Third: You can just run the *run.sh* in your terminal (sudo if needed).
`sudo ./run.sh`

*if you can not run the build program you might need to make sure it is executable*

You can also run it 
- with cargo `cargo run --release`
- directly from the release directory `./maschine /dev/hidrawX` X is the number for your hidraw location.
- Without the picture on screen (in release directory) `./maschine /dev/hidrawX no`



Fourth (optional): I have included a shellscript to turn on all the lights at once.
It's a bit inconvenient but:
 - With the program open in a terminal, open a new terminal `cd MaschineMK2_linux` and `./insert_colors.sh`

*Info*

- Group buttons change the midi note base.
- Most other buttons can be mapped in Reaper (I don't know about other DAW's).
- Encoders are currently absolute 360 degrees, but they stop at 98% (-ish).

# future todos:

- Remove OSC (I still depend on it to turn the lights on)
- Make the midi statement for buttons better
- add padmodes for different CC configurations
- maybe a personal config file?
- fix encoder jankyness
- screens?
