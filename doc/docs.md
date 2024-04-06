# Documentation for the code, release notes and ramblings


# Contents

- How to do things and notes on the project
- Release notes
- Midi and OSC
- The brain damage that is Image writing (please fix)
- History


# How to do things and notes on the project

How to do things:
- Read at the code.
- Read the original maschine.rs code.
- Read the documentation for the crates used.
- Read the documentation for thae crates used in the crates.
- (Optional) Read the rust book.
- Write code and see what happens.

Notes:
- This project uses some kind of selfmade Alsa-seq crate that is not documented, so look at the alsa-sys and the midi crate documentation.
- **Figure it out**
- There's a text file with all the HID tables listed and some notes on how those may (*or may not*) work.


# Release notes

Nothing new, see maschinemk2.rs commits for more info.


# Midi and OSC

The Midi system is literally just a match statement taped onto the OSC-match-buttons thing.
The encoder step function for OSC does not work anymore and is there only for implementing this for similar gear with even more buttons.
I don't know any usecases for OSC, but it is still there.


# The brain damage that is image writing

So the image writing works by taking the png as you would with the PNG crate.
Then dividing it by 4 (removing colors).
Then dividing that into a string of 8bit binary.
Then turning that into a number and putting it on screen.

A person who has a frontal lobe would have probably made it faster by using the write heads properly.


# History

- I found maschine.rs and decided to try if it worked on my MK2. **It Did Not**
- I then decided to learn rust by making it work.
- I mapped out the buttons that were working, and put them in the OSC maps and such in the right order.
- I spent a few days trying to do lights. I eventually figured out that the MK2 has 3 different light modules and got them working*.
- I spent a few days trying to clear the screen. I eventually found out that the MK2 has different parameters for the writing heads.
- I spent a few days trying to draw on one screen, turns out I am not good at programming and made a really unscalable way to parse raw png data.
- One morning I decided to make every button midi CC because I have no idea how to take midi input with these barely documented crates.

