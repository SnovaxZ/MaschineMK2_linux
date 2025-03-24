//  maschine.rs: user-space drivers for native instruments USB HIDs
//  Copyright (C) 2015 William Light <wrl@illest.net>
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Lesser General Public License as
//  published by the Free Software Foundation, either version 3 of the
//  License, or (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Lesser General Public License for more details.
//
//  You should have received a copy of the GNU Lesser General Public
//  License along with this program.  If not, see
//  <http://www.gnu.org/licenses/>.

use std::fs::File;
use std::mem::transmute;
use std::os::unix::io;

extern crate nix;
use midi::{Channel::Ch2, Message, U7};
use nix::unistd;

extern crate hex;
extern crate png;

use base::{Maschine, MaschineButton, MaschineHandler, MaschinePad, MaschinePadStateTransition};

use crate::base::maschine;

const BUTTON_REPORT_TO_MIKROBUTTONS_MAP: [[Option<MaschineButton>; 8]; 24] = [
    [
        Some(MaschineButton::F8),
        Some(MaschineButton::F7),
        Some(MaschineButton::F6),
        Some(MaschineButton::F5),
        Some(MaschineButton::F4),
        Some(MaschineButton::F3),
        Some(MaschineButton::F2),
        Some(MaschineButton::F1),
    ],
    [
        Some(MaschineButton::Auto),
        Some(MaschineButton::All),
        Some(MaschineButton::Pageleft),
        Some(MaschineButton::Pageright),
        Some(MaschineButton::Sampling),
        Some(MaschineButton::Browse),
        Some(MaschineButton::Step),
        Some(MaschineButton::Control),
    ],
    [
        Some(MaschineButton::Nav),
        Some(MaschineButton::Noterepeat),
        Some(MaschineButton::Enter),
        Some(MaschineButton::Navright),
        Some(MaschineButton::Navleft),
        Some(MaschineButton::Tempo),
        Some(MaschineButton::Swing),
        Some(MaschineButton::Volume),
    ],
    [
        Some(MaschineButton::GroupH),
        Some(MaschineButton::GroupG),
        Some(MaschineButton::GroupF),
        Some(MaschineButton::GroupE),
        Some(MaschineButton::GroupD),
        Some(MaschineButton::GroupC),
        Some(MaschineButton::GroupB),
        Some(MaschineButton::GroupA),
    ],
    [
        Some(MaschineButton::Shift),
        Some(MaschineButton::Erase),
        Some(MaschineButton::Rec),
        Some(MaschineButton::Play),
        Some(MaschineButton::Grid),
        Some(MaschineButton::Stepright),
        Some(MaschineButton::Stepleft),
        Some(MaschineButton::Restart),
    ],
    [
        Some(MaschineButton::Mute),
        Some(MaschineButton::Solo),
        Some(MaschineButton::Select),
        Some(MaschineButton::Duplicate),
        Some(MaschineButton::Navigate),
        Some(MaschineButton::Padmode),
        Some(MaschineButton::Pattern),
        Some(MaschineButton::Scene),
    ],
    [
        Some(MaschineButton::R1),
        Some(MaschineButton::R2),
        Some(MaschineButton::R3),
        Some(MaschineButton::R4),
        Some(MaschineButton::R5),
        Some(MaschineButton::R6),
        Some(MaschineButton::R7),
        Some(MaschineButton::R8),
    ],
    [
        Some(MaschineButton::A1),
        Some(MaschineButton::A2),
        Some(MaschineButton::A3),
        Some(MaschineButton::A4),
        Some(MaschineButton::A5),
        Some(MaschineButton::A6),
        Some(MaschineButton::A7),
        Some(MaschineButton::A8),
    ],
    [
        Some(MaschineButton::B1),
        Some(MaschineButton::B2),
        Some(MaschineButton::B3),
        Some(MaschineButton::B4),
        Some(MaschineButton::B5),
        Some(MaschineButton::B6),
        Some(MaschineButton::B7),
        Some(MaschineButton::B8),
    ],
    [
        Some(MaschineButton::C1),
        Some(MaschineButton::C2),
        Some(MaschineButton::C3),
        Some(MaschineButton::C4),
        Some(MaschineButton::C5),
        Some(MaschineButton::C6),
        Some(MaschineButton::C7),
        Some(MaschineButton::C8),
    ],
    [
        Some(MaschineButton::D1),
        Some(MaschineButton::D2),
        Some(MaschineButton::D3),
        Some(MaschineButton::D4),
        Some(MaschineButton::D5),
        Some(MaschineButton::D6),
        Some(MaschineButton::D7),
        Some(MaschineButton::D8),
    ],
    [
        Some(MaschineButton::E1),
        Some(MaschineButton::E2),
        Some(MaschineButton::E3),
        Some(MaschineButton::E4),
        Some(MaschineButton::E5),
        Some(MaschineButton::E6),
        Some(MaschineButton::E7),
        Some(MaschineButton::E8),
    ],
    [
        Some(MaschineButton::FF1),
        Some(MaschineButton::FF2),
        Some(MaschineButton::FF3),
        Some(MaschineButton::FF4),
        Some(MaschineButton::FF5),
        Some(MaschineButton::FF6),
        Some(MaschineButton::FF7),
        Some(MaschineButton::FF8),
    ],
    [
        Some(MaschineButton::G1),
        Some(MaschineButton::G2),
        Some(MaschineButton::G3),
        Some(MaschineButton::G4),
        Some(MaschineButton::G5),
        Some(MaschineButton::G6),
        Some(MaschineButton::G7),
        Some(MaschineButton::G8),
    ],
    [
        Some(MaschineButton::H1),
        Some(MaschineButton::H2),
        Some(MaschineButton::H3),
        Some(MaschineButton::H4),
        Some(MaschineButton::H5),
        Some(MaschineButton::H6),
        Some(MaschineButton::H7),
        Some(MaschineButton::H8),
    ],
    [
        Some(MaschineButton::I1),
        Some(MaschineButton::I2),
        Some(MaschineButton::I3),
        Some(MaschineButton::I4),
        Some(MaschineButton::I5),
        Some(MaschineButton::I6),
        Some(MaschineButton::I7),
        Some(MaschineButton::I8),
    ],
    [
        Some(MaschineButton::J1),
        Some(MaschineButton::J2),
        Some(MaschineButton::J3),
        Some(MaschineButton::J4),
        Some(MaschineButton::J5),
        Some(MaschineButton::J6),
        Some(MaschineButton::J7),
        Some(MaschineButton::J8),
    ],
    [
        Some(MaschineButton::K1),
        Some(MaschineButton::K2),
        Some(MaschineButton::K3),
        Some(MaschineButton::K4),
        Some(MaschineButton::K5),
        Some(MaschineButton::K6),
        Some(MaschineButton::K7),
        Some(MaschineButton::K8),
    ],
    [
        Some(MaschineButton::L1),
        Some(MaschineButton::L2),
        Some(MaschineButton::L3),
        Some(MaschineButton::L4),
        Some(MaschineButton::L5),
        Some(MaschineButton::L6),
        Some(MaschineButton::L7),
        Some(MaschineButton::L8),
    ],
    [
        Some(MaschineButton::M1),
        Some(MaschineButton::M2),
        Some(MaschineButton::M3),
        Some(MaschineButton::M4),
        Some(MaschineButton::M5),
        Some(MaschineButton::M6),
        Some(MaschineButton::M7),
        Some(MaschineButton::M8),
    ],
    [
        Some(MaschineButton::N1),
        Some(MaschineButton::N2),
        Some(MaschineButton::N3),
        Some(MaschineButton::N4),
        Some(MaschineButton::N5),
        Some(MaschineButton::N6),
        Some(MaschineButton::N7),
        Some(MaschineButton::N8),
    ],
    [
        Some(MaschineButton::O1),
        Some(MaschineButton::O2),
        Some(MaschineButton::O3),
        Some(MaschineButton::O4),
        Some(MaschineButton::O5),
        Some(MaschineButton::O6),
        Some(MaschineButton::O7),
        Some(MaschineButton::O8),
    ],
    [
        Some(MaschineButton::P1),
        Some(MaschineButton::P2),
        Some(MaschineButton::P3),
        Some(MaschineButton::P4),
        Some(MaschineButton::P5),
        Some(MaschineButton::P6),
        Some(MaschineButton::P7),
        Some(MaschineButton::P8),
    ],
    [
        Some(MaschineButton::Q1),
        Some(MaschineButton::Q2),
        Some(MaschineButton::Q3),
        Some(MaschineButton::Q4),
        Some(MaschineButton::Q5),
        Some(MaschineButton::Q6),
        Some(MaschineButton::Q7),
        Some(MaschineButton::Q8),
    ],
];

#[allow(dead_code)]
struct ButtonReport {
    pub buttons: u32,
    pub encoder: u8,
}

pub struct Mikro {
    dev: io::RawFd,
    light_buf: [u8; 49],
    light_buf2: [u8; 32],
    light_buf3: [u8; 57],

    pads: [MaschinePad; 16],
    buttons: [u8; 27],

    midi_note_base: u8,
    roller_state: [usize; 9],
    roller_status: [i32; 9],
    mod_state: usize,
    padmode: usize,

    note: [u8; 16],
    note_state: [usize; 16],
    noteset: bool,
    noteidx: usize,

    vel: [U7; 16],
    speed: u64,
    playing: bool,
}

impl Mikro {
    fn sixteen_maschine_pads() -> [MaschinePad; 16] {
        [
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
        ]
    }

    pub fn new(dev: io::RawFd) -> Self {
        let mut _self = Mikro {
            dev: dev,
            light_buf: [0u8; 49],
            light_buf2: [0u8; 32],
            light_buf3: [0u8; 57],

            pads: Mikro::sixteen_maschine_pads(),
            buttons: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
                0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
            ],

            midi_note_base: 48,
            roller_state: [0usize; 9],
            roller_status: [0i32; 9],
            mod_state: 0,
            padmode: 0,

            note: [48u8; 16],
            note_state: [0usize; 16],
            noteset: false,
            noteidx: 0,

            vel: [80u8; 16],
            speed: 100,

            playing: false,
        };

        _self.light_buf[0] = 0x80;
        _self.light_buf2[0] = 0x82;
        _self.light_buf3[0] = 0x81;
        return _self;
    }

    fn read_buttons(&mut self, handler: &mut dyn MaschineHandler, buf: &[u8]) {
        for (idx, &byte) in buf[0..24].iter().enumerate() {
            let mut diff = (byte ^ self.buttons[idx]) as u32;
            //println!("IDX: {}, Value{}", idx, byte);
            let mut off = 0usize;
            while diff != 0 {
                off += (diff.trailing_zeros() + 1) as usize;
                let btn = BUTTON_REPORT_TO_MIKROBUTTONS_MAP[idx][8 - off]
                    .expect("unknown button received from device");
                if idx <= 7 {
                    if (byte & (1 << (off - 1))) != 0 {
                        //println!(" {} ", byte);
                        let is_down = true;
                        handler.button_down(self, btn, byte, is_down);
                    } else {
                        let is_down = false;
                        //print!(" {} ", byte);
                        handler.button_up(self, btn, byte, is_down);
                    };
                } else {
                        if idx % 2 == 0  {
                            handler.encoder_step(self, (idx - 7) / 2 ,byte as i32 );
                        } else {
                            self.set_roller_state(byte as usize, (idx - 8) / 2 as usize);
                        };
                };
                                diff >>= off;
            }

            self.buttons[idx] = byte;
        }

        if self.buttons[23] > 0xF {
            self.buttons[23] = buf[23];
            return;
        } else if self.buttons[23] == buf[23] {
            return;
        }
        self.buttons[23] = buf[23];
    }

    fn read_pads(&mut self, handler: &mut dyn MaschineHandler, buf: &[u8]) {
        let pads: &[u16] = unsafe { transmute(buf) };

        for i in 0..16 {
            let pressure = ((pads[i] & 0xFFF) as f32) / 4095.0;

            match self.pads[i].pressure_val(pressure) {
                MaschinePadStateTransition::Pressed => handler.pad_pressed(self, i, pressure),

                MaschinePadStateTransition::Aftertouch => handler.pad_aftertouch(self, i, pressure),

                MaschinePadStateTransition::Released => handler.pad_released(self, i),

                _ => {}
            }
        }
    }
}

fn set_rgb_light(rgb: &mut [u8], color: u32, brightness: f32) {
    let brightness = brightness * 0.5;

    rgb[0] = (brightness * (((color >> 16) & 0xFF) as f32)) as u8;
    rgb[1] = (brightness * (((color >> 8) & 0xFF) as f32)) as u8;
    rgb[2] = (brightness * (((color) & 0xFF) as f32)) as u8;
}

impl Maschine for Mikro {
    fn get_fd(&self) -> io::RawFd {
        return self.dev;
    }

    fn write_lights(&mut self) {
        unistd::write(self.dev, &self.light_buf).unwrap();
        unistd::write(self.dev, &self.light_buf2).unwrap();
        unistd::write(self.dev, &self.light_buf3).unwrap();
    }

    fn set_pad_light(&mut self, pad: usize, color: u32, brightness: f32) {
        let offset = 1 + (pad * 3);
        let rgb = &mut self.light_buf[offset..(offset + 3)];

        set_rgb_light(rgb, color, brightness);
    }

    fn set_midi_note_base(&mut self, base: u8) {
        self.midi_note_base = base;
    }

    fn get_midi_note_base(&self) -> u8 {
        return self.midi_note_base;
    }

    fn set_roller_state(&mut self, state: usize, idx: usize) {
        self.roller_state[idx] = state;
    }

    fn get_roller_state(&self, idx: usize) -> usize {
        return self.roller_state[idx];
    }

    fn set_roller_status(&mut self, status: i32, idx: usize) {
        self.roller_status[idx] = status;
    }

    fn get_roller_status(&self, idx:usize) -> i32 {
        return self.roller_status[idx]
    }
    fn set_mod(&mut self, state: usize) {
        self.mod_state = state;
    }

    fn get_mod(&self) -> usize {
        return self.mod_state;
    }

    fn set_padmode(&mut self, state: usize) {
        if self.padmode < 3 && state == 1 {
            self.padmode += 1
        } else {
            self.padmode = 0;
        };
        println!("Padmode {}", self.padmode);
        if self.padmode == 2 {
            println!("This is Sequencer mode");
            println!("");
            println!("Tapping on pads activates them for the sequence.");
            println!("Tapping on a pad while holding shift, then pressing another pad");
            println!("will change the note of the pad you pressed first");
        }
    }

    fn get_padmode(&self) -> usize {
        return self.padmode;
    }

    fn set_playing(&mut self, state: usize) {
        if state == 1 {
            self.playing = true;
        } else {
            self.playing = false;
        }
    }

    fn get_playing(&self) -> bool {
        return self.playing;
    }

    fn note_save(&mut self, pad_idx: usize, note: u8, vel: u8) {
        if self.noteset == true {
            self.vel[self.noteidx] = vel;
            self.note[self.noteidx] = note;
            println!(
                "step: {}, note:{}, velocity{}",
                self.noteidx, self.note[self.noteidx], self.vel[self.noteidx]
            );
            self.noteset = false;
        } else {
            self.noteidx = pad_idx;
            self.noteset = true;
        };
    }

    fn note_state(&mut self, pad_idx: usize, msg: usize) {
        self.note_state[pad_idx] = msg;
    }

    fn note_check(&self, pad_idx: usize) -> usize {
        return self.note_state[pad_idx];
    }

    fn load_notes(&self, pad_idx: usize, context: usize) -> midi::Message {
        if context == 1 {
            let msg = Message::NoteOn(Ch2, self.note[pad_idx], self.vel[pad_idx]);
            return msg;
        } else {
            let msg = Message::NoteOff(Ch2, self.note[pad_idx], self.vel[pad_idx]);
            return msg;
        }
    }

    fn set_seq_speed(&mut self, status: usize) {
        self.speed = status as u64;
        println!("sequencer rate: {}", self.speed);
    }

    fn get_seq_speed(&self) -> u64 {
        return self.speed
    }

    fn set_button_light(&mut self, btn: MaschineButton, _color: u32, brightness: f32) {
        let mut idx = 0;
        let mut idx2 = 0;
        match btn {
            MaschineButton::F8 => idx = 1,
            MaschineButton::F7 => idx = 2,
            MaschineButton::F6 => idx = 3,
            MaschineButton::F5 => idx = 4,
            MaschineButton::F4 => idx = 5,
            MaschineButton::F3 => idx = 6,
            MaschineButton::F2 => idx = 7,
            MaschineButton::F1 => idx = 8,

            MaschineButton::Auto => idx = 9,
            MaschineButton::All => idx = 10,
            MaschineButton::Pageleft => idx = 11,
            MaschineButton::Pageright => idx = 12,

            MaschineButton::Sampling => idx = 13,

            MaschineButton::Noterepeat => idx = 14,
            MaschineButton::Enter => idx = 15,
            MaschineButton::Navright => idx = 16,
            MaschineButton::Navleft => idx = 17,
            MaschineButton::Tempo => idx = 18,
            MaschineButton::Swing => idx = 19,
            MaschineButton::Volume => idx = 20,

            MaschineButton::Mute => idx = 21,
            MaschineButton::Solo => idx = 22,
            MaschineButton::Select => idx = 23,
            MaschineButton::Duplicate => idx = 24,
            MaschineButton::Navigate => idx = 25,
            MaschineButton::Padmode => idx = 26,
            MaschineButton::Pattern => idx = 27,
            MaschineButton::Scene => idx = 28,
            MaschineButton::Control => idx = 29,
            MaschineButton::Step => idx = 30,
            MaschineButton::Browse => idx = 31,

            MaschineButton::GroupH => idx2 = 2,
            MaschineButton::GroupG => idx2 = 9,
            MaschineButton::GroupF => idx2 = 14,
            MaschineButton::GroupE => idx2 = 22,
            MaschineButton::GroupD => idx2 = 26,
            MaschineButton::GroupC => idx2 = 34,
            MaschineButton::GroupB => idx2 = 39,
            MaschineButton::GroupA => idx2 = 48,
            MaschineButton::Shift => idx2 = 47,
            MaschineButton::Erase => idx2 = 56,
            MaschineButton::Rec => idx2 = 54,
            MaschineButton::Play => idx2 = 53,
            MaschineButton::Grid => idx2 = 52,
            MaschineButton::Stepright => idx2 = 51,
            MaschineButton::Stepleft => idx2 = 50,
            MaschineButton::Restart => idx2 = 49,

            _ => return,
        };
        if idx != 0 {
            //println!("light this {}, brightness {}", idx, brightness);
            self.light_buf2[idx] = brightness as u8;
        } else {
            self.light_buf3[idx2] = brightness as u8;
        }
    }

    fn readable(&mut self, handler: &mut dyn MaschineHandler) {
        let mut buf = [0u8; 512];

        let nbytes = match unistd::read(self.dev, &mut buf) {
            Err(err) => panic!("read failed: {}", err.to_string()),
            Ok(nbytes) => nbytes,
        };

        let report_nr = buf[0];
        let buf = &buf[1..nbytes];

        match report_nr {
            0x01 => self.read_buttons(handler, &buf),
            0x20 => self.read_pads(handler, &buf),
            _ => println!(" :: {:2X}: got {} bytes", report_nr, nbytes),
        }
    }

    fn get_pad_pressure(&self, pad_idx: usize) -> Result<f32, ()> {
        match pad_idx {
            0..=15 => Ok(self.pads[pad_idx].get_pressure()),
            _ => Err(()),
        }
    }

    fn clear_screen(&mut self) {
        let mut screen_buf = [0u8; 1 + 8 + 512];
        let mut screen_buf2 = [0u8; 1 + 8 + 512];

        screen_buf[0] = 0xE0;
        //screen_buf[3] = 16;
        screen_buf[5] = 0x08;
        screen_buf[7] = 0x20;

        //screen_buf[16] = 0xFF;

        screen_buf2[0] = 0xE1;
        //screen_buf2[3] = 16;
        screen_buf2[5] = 0x08;
        screen_buf2[7] = 0x20;

        let mut k = 0;
        let mut t = 0;
        while k < 9 {
            screen_buf[1] = k * 4;
            screen_buf2[1] = k * 4;
            k += 1;

            if k == 8 {
                screen_buf[3] = t * 4;
                screen_buf2[3] = t * 4;
                if t < 8 {
                    k = 0;
                }
                t += 1;
            }
            unistd::write(self.dev, &screen_buf).unwrap();
            unistd::write(self.dev, &screen_buf2).unwrap();
        }

        println!("Screen clear done?");
    }

    fn write_screen(&mut self) {
        let mut limits = png::Limits::default();
        limits.bytes = 10 * 1024;
        let decoder = png::Decoder::new_with_limits(File::open("picturetest.png").unwrap(), limits);
        let mut reader = decoder.read_info().unwrap();
        let mut picture = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut picture).unwrap();
        let bytes = &picture[..info.buffer_size()];
        let mut screen_buf = [0u8; 1 + 8 + 512];
        //println!("{}", bytes.len());

        //let mut screen_buf2 = [0u8; 1 + 8+ 512];
        screen_buf[0] = 0xE0;
        screen_buf[5] = 0x08;
        screen_buf[7] = 0x20;

        screen_buf[1] = 0;
        screen_buf[3] = 0;

        let mut screen_writer = 9;
        let mut steps = 0;
        let mut bits = [0u8; 4097];
        let mut inc = 0;
        let mut ok = 0;
        let mut count2 = 0;

        let mut a1 = 0;
        let mut a2 = 0;
        let mut a3 = 0;
        let mut a4 = 0;
        let mut a5 = 0;
        let mut a6 = 0;
        let mut a7 = 0;
        let mut a8 = 0;

        for count in 0..bytes.len() {
            let c = 1 + 4 * count;
            let mut swap = 0;
            //if bytes[c] / 8 + bytes[c + 1] / 8  + bytes[c + 2] / 8 + bytes[c + 3] / 8  + bytes[c + 4] / 8  + bytes[c + 5] / 8  + bytes[c + 6] / 8  + bytes[c + 7] / 8 >= 32{
            if c < bytes.len() - 3 {
                if bytes[c] / 2 + bytes[c + 2] / 2 >= 128 {
                    swap = 1;
                } else {
                    swap = 0;
                }
                //println!("{}", swap);
            }
            let mut binary = [0u8; 4097];
            if c < 65534 {
                //print!("{}, ", bytes[count]);
                let intval;
                match inc {
                    0 => a1 = swap,
                    1 => a2 = swap,
                    2 => a3 = swap,
                    3 => a4 = swap,
                    4 => a5 = swap,
                    5 => a6 = swap,
                    6 => a7 = swap,
                    7 => a8 = swap,
                    _ => return,
                }
                inc += 1;
                if inc == 8 {
                    inc = 0;
                }
                ok += 1;
                if ok == 8 {
                    let combination = format!("{}{}{}{}{}{}{}{}", a1, a2, a3, a4, a5, a6, a7, a8);
                    intval = usize::from_str_radix(&combination, 2).unwrap();
                    ok = 0;
                    binary[count2] = intval as u8;
                    bits[count2] = binary[count2];
                    count2 += 1;
                    a1 = 0;
                    a2 = 0;
                    a3 = 0;
                    a4 = 0;
                    a5 = 0;
                    a6 = 0;
                    a7 = 0;
                    a8 = 0;
                }
            }
            //let intval = usize::from_str_radix(&combination, 4).unwrap();
            //println!("{}", combination)
        }

        for a in 0..bits.len() {
            if screen_writer == 10 {
                if steps <= 30 {
                    screen_buf[1] += 1;
                    steps += 1;
                    screen_writer = 9;
                    screen_buf[screen_writer] = bits[a];
                } else {
                    screen_buf[3] += 1;
                    screen_buf[1] = 0;
                    steps = 0;
                    screen_writer = 9;
                    screen_buf[screen_writer] = bits[a];
                }
            }
            //println!("{}", bits[a]);
            unistd::write(self.dev, &screen_buf).unwrap();
            screen_writer += 1;
        }
        println!("RUNNING!");
    }
}
