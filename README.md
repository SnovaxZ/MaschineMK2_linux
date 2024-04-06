# maschinemk2.rs

**maschinemk2.rs is an open-source program that allows you to (possibly) use your Maschine MK2 device on linux**
It is absolutely and completely untested and I have never written rust before this project.


# Features
- Functional midi pads
- Buttons as midi cc so you can map them into a DAW or something
- Encoders as midi cc (not quite 100% functional)
- Lights
- Picture on the screen
- Osc from the original maschine.rs
- Mappable 



# Building
to build maschinemk2.rs you will need rustc and cargo.
to install rust, go to: https://www.rust-lang.org/tools/install

build:
``` sh
git clone https://github.com/SnovaxZ/maschinemk2.rs.git
cd maschinemk2.rs
./build.sh
```

the *build.sh* just runs cargo build and moves the test picture into the release directory.

*if you can not run the build program you might need to make sure it is executable*




# Use
First: figure out which hidraw path your maschine uses.

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
 - With the program open in a terminal, open a new terminal `cd maschinemk2.rs` and `./insert_colors.sh`


