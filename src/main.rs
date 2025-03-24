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

use std::env;
use std::os::unix::io::AsRawFd;
use std::path::Path;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};

use std::time::{Duration, SystemTime};

extern crate nix;
use nix::fcntl::{O_NONBLOCK, O_RDWR};
use nix::poll::*;
use nix::{fcntl, sys};

extern crate alsa_seq;
extern crate midi;
use alsa_seq::*;
use midi::*;

extern crate hsl;
use hsl::HSL;

#[macro_use(osc_args)]
extern crate tinyosc;
use tinyosc as osc;

mod base;
mod devices;

use base::{maschine, Maschine, MaschineButton, MaschineHandler};

fn ev_loop(dev: &mut dyn Maschine, mhandler: &mut MHandler) {
    let mut fds = [
        PollFd::new(dev.get_fd(), POLLIN, EventFlags::empty()),
        PollFd::new(mhandler.osc_socket.as_raw_fd(), POLLIN, EventFlags::empty()),
    ];

    let mut now = SystemTime::now();
    let mut now2 = SystemTime::now();
    let timer_interval = Duration::from_millis(16);
    let mut timer_interval2;
    let mut step = 0;
    let mut check = 0;
    let mut active = false;
    loop {
        poll(&mut fds, 16).unwrap();

        if fds[0].revents().unwrap().contains(POLLIN) {
            dev.readable(mhandler);
        }

        if fds[1].revents().unwrap().contains(POLLIN) {
            mhandler.recv_osc_msg(dev);
        }

        if now.elapsed().unwrap() >= timer_interval {
            dev.write_lights();
            now = SystemTime::now();
        }
        if dev.get_playing() == true {
            timer_interval2 = Duration::from_millis(dev.get_seq_speed());
            active = true;
            if dev.note_check(step) == 1 && now2.elapsed().unwrap() >= timer_interval2 && check == 0
            {
                let msg = dev.load_notes(step, 1);
                mhandler.seq_port.send_message(&msg).unwrap();
                mhandler.seq_handle.drain_output();
                check = 1;
            };
            if now2.elapsed().unwrap() >= timer_interval2 * 2 && dev.note_check(step) == 1 {
                let msg = dev.load_notes(step, 0);
                mhandler.seq_port.send_message(&msg).unwrap();
                mhandler.seq_handle.drain_output();
                now2 = SystemTime::now();
                step += 1;
                check = 0;
            } else if now2.elapsed().unwrap() >= timer_interval2 * 2 && dev.note_check(step) == 0 {
                step += 1;
                check = 0;
                now2 = SystemTime::now();
            };
            if step >= 16 {
                step = 0;
            };
        } else if active == true {
            let msg = dev.load_notes(step, 0);
            mhandler.seq_port.send_message(&msg).unwrap();
            mhandler.seq_handle.drain_output();
            active = false;
        }
    }
}

fn usage(prog_name: &String) {
    println!("usage: {} <hidraw device>", prog_name);
}

const PAD_RELEASED_BRIGHTNESS: f32 = 0.015;

#[allow(dead_code)]
enum PressureShape {
    Linear,
    Exponential(f32),
    Constant(f32),
}

struct MHandler<'a> {
    color: HSL,

    seq_handle: &'a SequencerHandle,
    seq_port: &'a SequencerPort<'a>,

    pressure_shape: PressureShape,
    send_aftertouch: bool,

    osc_socket: &'a UdpSocket,
    osc_outgoing_addr: SocketAddr,
}

fn osc_button_to_btn_map(osc_button: &str) -> Option<MaschineButton> {
    match osc_button {
        "restart" => Some(MaschineButton::Restart),
        "step_left" => Some(MaschineButton::Stepleft),
        "step_right" => Some(MaschineButton::Stepright),
        "grid" => Some(MaschineButton::Grid),
        "play" => Some(MaschineButton::Play),
        "rec" => Some(MaschineButton::Rec),
        "stop" => Some(MaschineButton::Erase),
        "shift" => Some(MaschineButton::Shift),

        "browse" => Some(MaschineButton::Browse),
        "sampling" => Some(MaschineButton::Sampling),
        "note_repeat" => Some(MaschineButton::Noterepeat),

        "encoder" => Some(MaschineButton::Encoder),

        "f1" => Some(MaschineButton::F1),
        "f2" => Some(MaschineButton::F2),
        "f3" => Some(MaschineButton::F3),
        "f4" => Some(MaschineButton::F4),
        "f5" => Some(MaschineButton::F5),
        "f6" => Some(MaschineButton::F6),
        "f7" => Some(MaschineButton::F7),
        "f8" => Some(MaschineButton::F8),

        "swing" => Some(MaschineButton::Swing),
        "step" => Some(MaschineButton::Step),
        "volume" => Some(MaschineButton::Volume),

        "enter" => Some(MaschineButton::Enter),
        "auto" => Some(MaschineButton::Auto),
        "all" => Some(MaschineButton::All),
        "navigate" => Some(MaschineButton::Navigate),
        "tempo" => Some(MaschineButton::Tempo),
        //"stop" => Some(MaschineButton::Erase),
        "control" => Some(MaschineButton::Control),
        "nav" => Some(MaschineButton::Nav),
        "nav_left" => Some(MaschineButton::Navleft),
        "nav_right" => Some(MaschineButton::Navright),
        "main" => Some(MaschineButton::Main),

        "scene" => Some(MaschineButton::Scene),
        "pattern" => Some(MaschineButton::Pattern),
        "pad_mode" => Some(MaschineButton::Padmode),
        "view" => Some(MaschineButton::View),
        "duplicate" => Some(MaschineButton::Duplicate),
        "select" => Some(MaschineButton::Select),
        "solo" => Some(MaschineButton::Solo),
        "mute" => Some(MaschineButton::Mute),

        "group_a" => Some(MaschineButton::GroupA),
        "group_b" => Some(MaschineButton::GroupB),
        "group_c" => Some(MaschineButton::GroupC),
        "group_d" => Some(MaschineButton::GroupD),
        "group_e" => Some(MaschineButton::GroupE),
        "group_f" => Some(MaschineButton::GroupF),
        "group_g" => Some(MaschineButton::GroupG),
        "group_h" => Some(MaschineButton::GroupH),

        "page_right" => Some(MaschineButton::Pageright),
        "page_left" => Some(MaschineButton::Pageleft),

        _ => None,
    }
}

fn btn_to_osc_button_map(btn: MaschineButton) -> &'static str {
    match btn {
        MaschineButton::Restart => "restart",
        MaschineButton::Stepleft => "step_left",
        MaschineButton::Stepright => "step_right",
        MaschineButton::Grid => "grid",
        MaschineButton::Play => "play",
        MaschineButton::Rec => "rec",
        MaschineButton::Erase => "stop",
        MaschineButton::Shift => "shift",

        MaschineButton::Browse => "browse",
        MaschineButton::Sampling => "sampling",
        MaschineButton::Noterepeat => "note_repeat",

        MaschineButton::Encoder => "encoder",

        MaschineButton::F1 => "f1",
        MaschineButton::F2 => "f2",
        MaschineButton::F3 => "f3",
        MaschineButton::F4 => "f4",
        MaschineButton::F5 => "f5",
        MaschineButton::F6 => "f6",
        MaschineButton::F7 => "f7",
        MaschineButton::F8 => "f8",

        MaschineButton::Swing => "swing",
        MaschineButton::Step => "step",
        MaschineButton::Volume => "volume",

        MaschineButton::Enter => "enter",
        MaschineButton::Auto => "auto",
        MaschineButton::All => "all",
        MaschineButton::Navigate => "navigate",
        MaschineButton::Tempo => "tempo",

        MaschineButton::Control => "control",
        MaschineButton::Nav => "nav",
        MaschineButton::Navleft => "nav_left",
        MaschineButton::Navright => "nav_right",
        MaschineButton::Main => "main",

        MaschineButton::Scene => "scene",
        MaschineButton::Pattern => "pattern",
        MaschineButton::Padmode => "pad_mode",
        MaschineButton::View => "view",
        MaschineButton::Duplicate => "duplicate",
        MaschineButton::Select => "select",
        MaschineButton::Solo => "solo",
        MaschineButton::Mute => "mute",

        MaschineButton::GroupA => "group_a",
        MaschineButton::GroupB => "group_b",
        MaschineButton::GroupC => "group_c",
        MaschineButton::GroupD => "group_d",
        MaschineButton::GroupE => "group_e",
        MaschineButton::GroupF => "group_f",
        MaschineButton::GroupG => "group_g",
        MaschineButton::GroupH => "group_h",

        MaschineButton::Pageright => "page_right",
        MaschineButton::Pageleft => "page_left",
        MaschineButton::R1 => "R1",
        MaschineButton::R2 => "R2",
        MaschineButton::R3 => "R3",
        MaschineButton::R4 => "R4",
        MaschineButton::R5 => "R5",
        MaschineButton::R6 => "R6",
        MaschineButton::R7 => "R7",
        MaschineButton::R8 => "R8",

        MaschineButton::A1 => "A1",
        MaschineButton::A2 => "A2",
        MaschineButton::A3 => "A3",
        MaschineButton::A4 => "A4",
        MaschineButton::A5 => "A5",
        MaschineButton::A6 => "A6",
        MaschineButton::A7 => "A7",
        MaschineButton::A8 => "A8",

        MaschineButton::B1 => "B1",
        MaschineButton::B2 => "B2",
        MaschineButton::B3 => "B3",
        MaschineButton::B4 => "B4",
        MaschineButton::B5 => "B5",
        MaschineButton::B6 => "B6",
        MaschineButton::B7 => "B7",
        MaschineButton::B8 => "B8",

        MaschineButton::C1 => "C1",
        MaschineButton::C2 => "C2",
        MaschineButton::C3 => "C3",
        MaschineButton::C4 => "C4",
        MaschineButton::C5 => "C5",
        MaschineButton::C6 => "C6",
        MaschineButton::C7 => "C7",
        MaschineButton::C8 => "C8",

        MaschineButton::D1 => "D1",
        MaschineButton::D2 => "D2",
        MaschineButton::D3 => "D3",
        MaschineButton::D4 => "D4",
        MaschineButton::D5 => "D5",
        MaschineButton::D6 => "D6",
        MaschineButton::D7 => "D7",
        MaschineButton::D8 => "D8",

        MaschineButton::E1 => "E1",
        MaschineButton::E2 => "E2",
        MaschineButton::E3 => "E3",
        MaschineButton::E4 => "E4",
        MaschineButton::E5 => "E5",
        MaschineButton::E6 => "E6",
        MaschineButton::E7 => "E7",
        MaschineButton::E8 => "E8",

        MaschineButton::FF1 => "FF1",
        MaschineButton::FF2 => "FF2",
        MaschineButton::FF3 => "FF3",
        MaschineButton::FF4 => "FF4",
        MaschineButton::FF5 => "FF5",
        MaschineButton::FF6 => "FF6",
        MaschineButton::FF7 => "FF8",
        MaschineButton::FF8 => "FF8",

        MaschineButton::G1 => "G1",
        MaschineButton::G2 => "G2",
        MaschineButton::G3 => "G3",
        MaschineButton::G4 => "G4",
        MaschineButton::G5 => "G5",
        MaschineButton::G6 => "G6",
        MaschineButton::G7 => "G7",
        MaschineButton::G8 => "G8",

        MaschineButton::H1 => "H1",
        MaschineButton::H2 => "H2",
        MaschineButton::H3 => "H3",
        MaschineButton::H4 => "H4",
        MaschineButton::H5 => "H5",
        MaschineButton::H6 => "H6",
        MaschineButton::H7 => "H7",
        MaschineButton::H8 => "H8",

        MaschineButton::I1 => "I1",
        MaschineButton::I2 => "I2",
        MaschineButton::I3 => "I3",
        MaschineButton::I4 => "I4",
        MaschineButton::I5 => "I5",
        MaschineButton::I6 => "I6",
        MaschineButton::I7 => "I7",
        MaschineButton::I8 => "I8",

        MaschineButton::J1 => "J1",
        MaschineButton::J2 => "J2",
        MaschineButton::J3 => "J3",
        MaschineButton::J4 => "J4",
        MaschineButton::J5 => "J5",
        MaschineButton::J6 => "J6",
        MaschineButton::J7 => "J7",
        MaschineButton::J8 => "J8",

        MaschineButton::K1 => "K1",
        MaschineButton::K2 => "K2",
        MaschineButton::K3 => "K3",
        MaschineButton::K4 => "K4",
        MaschineButton::K5 => "K5",
        MaschineButton::K6 => "K6",
        MaschineButton::K7 => "K7",
        MaschineButton::K8 => "K8",

        MaschineButton::L1 => "L1",
        MaschineButton::L2 => "L2",
        MaschineButton::L3 => "L3",
        MaschineButton::L4 => "L4",
        MaschineButton::L5 => "L5",
        MaschineButton::L6 => "L6",
        MaschineButton::L7 => "L7",
        MaschineButton::L8 => "L8",

        MaschineButton::M1 => "M1",
        MaschineButton::M2 => "M2",
        MaschineButton::M3 => "M3",
        MaschineButton::M4 => "M4",
        MaschineButton::M5 => "M5",
        MaschineButton::M6 => "M6",
        MaschineButton::M7 => "M7",
        MaschineButton::M8 => "M8",

        MaschineButton::N1 => "N1",
        MaschineButton::N2 => "N2",
        MaschineButton::N3 => "N3",
        MaschineButton::N4 => "N4",
        MaschineButton::N5 => "N5",
        MaschineButton::N6 => "N6",
        MaschineButton::N7 => "N7",
        MaschineButton::N8 => "N8",

        MaschineButton::O1 => "O1",
        MaschineButton::O2 => "O2",
        MaschineButton::O3 => "O3",
        MaschineButton::O4 => "O4",
        MaschineButton::O5 => "O5",
        MaschineButton::O6 => "O6",
        MaschineButton::O7 => "O7",
        MaschineButton::O8 => "O8",

        MaschineButton::P1 => "P1",
        MaschineButton::P2 => "P2",
        MaschineButton::P3 => "P3",
        MaschineButton::P4 => "P4",
        MaschineButton::P5 => "P5",
        MaschineButton::P6 => "P6",
        MaschineButton::P7 => "P7",
        MaschineButton::P8 => "P8",

        _=> "NO",
    }
}

impl<'a> MHandler<'a> {
    fn pad_color(&self) -> u32 {
        let (r, g, b) = self.color.to_rgb();

        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    fn pressure_to_vel(&self, pressure: f32) -> U7 {
        (match self.pressure_shape {
            PressureShape::Linear => pressure,
            PressureShape::Exponential(power) => pressure.powf(power),
            PressureShape::Constant(c_pressure) => c_pressure,
        } * 127.0) as U7
    }

    #[allow(dead_code)]
    fn update_pad_colors(&self, maschine: &mut dyn Maschine) {
        for i in 0..16 {
            let brightness = match maschine.get_pad_pressure(i).unwrap() {
                b if b == 0.0 => PAD_RELEASED_BRIGHTNESS,
                pressure @ _ => pressure.sqrt(),
            };

            maschine.set_pad_light(i, self.pad_color(), brightness);
        }
    }

    fn recv_osc_msg(&self, maschine: &mut dyn Maschine) {
        let mut buf = [0u8; 128];

        let nbytes = match self.osc_socket.recv_from(&mut buf) {
            Ok((nbytes, _)) => nbytes,
            Err(e) => {
                println!(" :: error in recv_from(): {}", e);
                return;
            }
        };

        let msg = match osc::Message::deserialize(&buf[..nbytes]) {
            Ok(msg) => msg,
            Err(_) => {
                println!(" :: couldn't decode OSC message :c");
                return;
            }
        };

        self.handle_osc_messge(maschine, &msg);
    }

    fn handle_osc_messge(&self, maschine: &mut dyn Maschine, msg: &osc::Message) {
        if msg.path.starts_with("/maschine/button") {
            let btn = match osc_button_to_btn_map(&msg.path[17..]) {
                Some(btn) => btn,
                None => return,
            };

            match msg.arguments.len() {
                1 => maschine.set_button_light(
                    btn,
                    0xFFFFFF,
                    match msg.arguments[0] {
                        osc::Argument::i(val) => val as f32,
                        osc::Argument::f(val) => val,
                        _ => return,
                    },
                ),

                2 => {
                    if let (&osc::Argument::i(color), &osc::Argument::f(brightness)) =
                        (&msg.arguments[0], &msg.arguments[1])
                    {
                        maschine.set_button_light(btn, (color as u32) & 0xFFFFFF, brightness);
                    }
                }

                _ => return,
            };
        } else if msg.path.starts_with("/maschine/pad") {
            match msg.arguments.len() {
                3 => {
                    if let (
                        &osc::Argument::i(pad),
                        &osc::Argument::i(color),
                        &osc::Argument::f(brightness),
                    ) = (&msg.arguments[0], &msg.arguments[1], &msg.arguments[2])
                    {
                        maschine.set_pad_light(
                            pad as usize,
                            (color as u32) & 0xFFFFFF,
                            brightness as f32,
                        );
                    }
                }

                _ => return,
            }
        } else if msg.path.starts_with("/maschine/midi_note_base") {
            match msg.arguments.len() {
                1 => {
                    if let osc::Argument::i(base) = msg.arguments[0] {
                        maschine.set_midi_note_base(base as u8);
                    }
                }
                _ => return,
            }
        }
    }

    fn send_osc_msg(&self, path: &str, arguments: Vec<osc::Argument>) {
        let msg = osc::Message {
            path: path,
            arguments: arguments,
        };

        match self
            .osc_socket
            .send_to(&*msg.serialize().unwrap(), &self.osc_outgoing_addr)
        {
            Ok(_) => {}
            Err(e) => println!(" :: error in send_to: {}", e),
        }
    }

//Status is Byte from previous stupid naming!
    fn send_osc_button_msg(
        &mut self,
        maschine: &mut dyn Maschine,
        btn: MaschineButton,
        status: usize,
        is_down: bool,
    ) {
        let button = btn_to_osc_button_map(btn);
        let controlbase = 15;
        let modpress = maschine.get_mod();
        //println!("{} is:  {}", button, status);
        if button.contains("shift") {
            if status > 0 {
                maschine.set_mod(1);
            } else {
                maschine.set_mod(0);
            }
        }
        if button.contains("C") {
            let idx = 1;
            //println!("C: {}", status);
            if button == "C8" {
                maschine.set_roller_state(status, idx);
                //println!("3={}", status);
            };
            if button == "C7" {
                maschine.set_roller_state(status, idx);
                //println!("2={}", status);
            };
        };
        if button.contains("E") {
            let idx = 2;
            if button == "E8" {
                maschine.set_roller_state(status, idx);
                //println!("3={}", status);
            };
        };
        if button.contains("G") {
            let idx = 3;
            if button == "G8" {
                maschine.set_roller_state(status, idx);
                //println!("3={}", status);
            };
        };
        if button.contains("I") {
            let idx = 4;
            if button == "I8" {
                maschine.set_roller_state(status, idx);
                //println!("3={}", status);
            };
        };
        if button.contains("K") {
            let idx = 5;
            if button == "K8" {
                maschine.set_roller_state(status, idx);
                //println!("3={}", status);
            };
        };
        if button.contains("M") {
            let idx = 6;
            if button == "M8" {
                maschine.set_roller_state(status, idx);
                //println!("3={}", status);
            };
        };
        if button.contains("O") {
            let idx = 7;
            if button == "O8" {
                maschine.set_roller_state(status, idx);
                //println!("3={}", status);
            };
        };
        if button.contains("A8") {
            let msg = Message::RPN7(Ch1, controlbase, status as u8 * 8);
            self.seq_port.send_message(&msg).unwrap();
            self.seq_handle.drain_output();
        }

        if is_down == true && status <= 250 {
            match button {
                "play" => {
                    if status > 0 && maschine.get_padmode() != 2 {
                        let msg = Message::RPN7(Ch1, 1, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    } else if maschine.get_padmode() == 2 {
                        maschine.set_playing(1);
                        println!("playing notes");
                    };
                }

                "stop" => {
                    if status > 0 && maschine.get_padmode() != 2 {
                        let msg = Message::RPN7(Ch1, 2, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    } else {
                        maschine.set_playing(0);
                        println!("stop");
                        //let msg2 = Message::AllNotesOff(Ch2);
                        //self.seq_port.send_message(&msg2).unwrap();
                        //self.seq_handle.drain_output();
                    }
                }
                "rec" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 3, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "grid" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 4, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "step_left" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 5, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "step_right" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 6, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "restart" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 7, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "browse" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 8, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "sampling" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 9, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "note_repeat" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 10, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "control" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 11, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "nav" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 12, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "nav_left" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 13, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "nav_right" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 14, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "main" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 24, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "scene" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 25, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "pattern" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 26, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "pad_mode" => {
                    if modpress == 1 {
                        maschine.set_padmode(1);
                    } else {
                        if status > 0 {
                            let msg = Message::RPN7(Ch1, 27, status as u8);
                            self.seq_port.send_message(&msg).unwrap();
                            self.seq_handle.drain_output();
                        }
                    }
                }
                "view" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 28, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "duplicate" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 29, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "select" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 30, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "solo" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 31, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "step" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 32, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "mute" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 33, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "navigate" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 34, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "tempo" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 35, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "enter" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 36, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "auto" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 37, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "all" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 38, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "f1" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 39, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "f2" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 40, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "f3" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 41, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "f4" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 42, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "f5" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 43, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "f6" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 44, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "f7" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 45, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "f8" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 46, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "page_right" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 47, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }
                "page_left" => {
                    if status > 0 {
                        let msg = Message::RPN7(Ch1, 48, status as u8);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    }
                }

                "B6" => {
                    let idx = 1;
                    let state = maschine.get_roller_state(idx);
                    let status = status / 4 + state * 64;
                    if modpress != 1 {
                        let msg = Message::RPN14(Ch1, controlbase + 1, status as u16 / 2);
                        self.seq_port.send_message(&msg).unwrap();
                        self.seq_handle.drain_output();
                    } else {
                        maschine.set_seq_speed(status);
                    }
                }
                "D6" => {
                    let idx = 2;
                    let state = maschine.get_roller_state(idx);
                    let status = status / 4 + state * 64;
                    let msg = Message::RPN14(Ch1, controlbase + 2, status as u16 / 2);
                    self.seq_port.send_message(&msg).unwrap();
                    self.seq_handle.drain_output();
                }
                "FF6" => {
                    let idx = 3;
                    let state = maschine.get_roller_state(idx);
                    let status = status / 4 + state * 64;
                    let msg = Message::RPN14(Ch1, controlbase + 3, status as u16 / 2);
                    self.seq_port.send_message(&msg).unwrap();
                    self.seq_handle.drain_output();
                }
                "H6" => {
                    let idx = 4;
                    let state = maschine.get_roller_state(idx);
                    let status = status / 4 + state * 64;
                    let msg = Message::RPN14(Ch1, controlbase + 4, status as u16 / 2);
                    self.seq_port.send_message(&msg).unwrap();
                    self.seq_handle.drain_output();
                }
                "J6" => {
                    let idx = 5;
                    let state = maschine.get_roller_state(idx);
                    let status = status / 4 + state * 64;
                    let msg = Message::RPN14(Ch1, controlbase + 5, status as u16 / 2);
                    self.seq_port.send_message(&msg).unwrap();
                    self.seq_handle.drain_output();
                }
                "L6" => {
                    let idx = 6;
                    let state = maschine.get_roller_state(idx);
                    let status = status / 4 + state * 64;
                    let msg = Message::RPN14(Ch1, controlbase + 6, status as u16 / 2);
                    self.seq_port.send_message(&msg).unwrap();
                    self.seq_handle.drain_output();
                }
                "N6" => {
                    let idx = 7;
                    let state = maschine.get_roller_state(idx);
                    let status = status / 4 + state * 64;
                    let msg = Message::RPN14(Ch1, controlbase + 7, status as u16 / 2);
                    self.seq_port.send_message(&msg).unwrap();
                    self.seq_handle.drain_output();
                }
                "P6" => {
                    let msg = Message::RPN14(Ch1, controlbase + 8, status as u16 / 2);
                    self.seq_port.send_message(&msg).unwrap();
                    self.seq_handle.drain_output();
                }

                "group_a" => {
                    maschine.set_midi_note_base(24);
                }
                "group_b" => {
                    maschine.set_midi_note_base(36);
                }
                "group_c" => {
                    maschine.set_midi_note_base(48);
                }
                "group_d" => {
                    maschine.set_midi_note_base(60);
                }
                "group_e" => {
                    maschine.set_midi_note_base(72);
                }
                "group_f" => {
                    maschine.set_midi_note_base(84);
                }
                "group_g" => {
                    maschine.set_midi_note_base(96);
                }
                "group_h" => {
                    maschine.set_midi_note_base(108);
                }
                _ => {}
            }
        }
        self.send_osc_msg(&*format!("/{}", button), osc_args![status as f32]);
    }

    fn send_osc_encoder_msg(&self, maschine: &mut dyn Maschine, idx: usize,  status: i32) {
        let state = maschine.get_roller_state(idx);
        let status = status / 4 + state as i32 * 64 ;
        if status - maschine.get_roller_status(idx) < 40 && maschine.get_roller_status(idx) - status < 40{
            let msg = Message::RPN14(Ch1, idx as u16 + 16, status as u16);
            maschine.set_roller_status(status, idx);
            self.seq_port.send_message(&msg).unwrap();
            self.seq_handle.drain_output();
        }
    }
}

const PAD_NOTE_MAP: [U7; 16] = [12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3];

impl<'a> MaschineHandler for MHandler<'a> {
    fn pad_pressed(&mut self, maschine: &mut dyn Maschine, pad_idx: usize, pressure: f32) {
        let midi_note = maschine.get_midi_note_base() + PAD_NOTE_MAP[pad_idx];
        let msg = Message::NoteOn(Ch1, midi_note, self.pressure_to_vel(pressure));
        if maschine.get_padmode() == 2 {
            if maschine.get_mod() != 1 {
                if maschine.note_check(pad_idx) == 0 {
                    maschine.note_state(pad_idx, 1);
                    maschine.set_pad_light(pad_idx, self.pad_color(), pressure.sqrt());
                } else {
                    maschine.note_state(pad_idx, 0);
                    maschine.set_pad_light(pad_idx, self.pad_color(), PAD_RELEASED_BRIGHTNESS);
                };
            } else {
                maschine.note_save(pad_idx, midi_note, self.pressure_to_vel(pressure));
            };
        } else {
            self.seq_port.send_message(&msg).unwrap();
            self.seq_handle.drain_output();
            maschine.set_pad_light(pad_idx, self.pad_color(), pressure.sqrt());
        };
    }

    fn pad_aftertouch(&mut self, maschine: &mut dyn Maschine, pad_idx: usize, pressure: f32) {
        match self.pressure_shape {
            PressureShape::Constant(_) => return,
            _ => {}
        }

        if !self.send_aftertouch {
            return;
        }

        let midi_note = maschine.get_midi_note_base() + PAD_NOTE_MAP[pad_idx];
        let msg = Message::PolyphonicPressure(Ch1, midi_note, self.pressure_to_vel(pressure));

        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();

        maschine.set_pad_light(pad_idx, self.pad_color(), pressure.sqrt());
    }

    fn pad_released(&mut self, maschine: &mut dyn Maschine, pad_idx: usize) {
        if maschine.get_padmode() != 2 {
            let midi_note = maschine.get_midi_note_base() + PAD_NOTE_MAP[pad_idx];
            let msg = Message::NoteOff(Ch1, midi_note, 0);
            self.seq_port.send_message(&msg).unwrap();
            self.seq_handle.drain_output();
            maschine.set_pad_light(pad_idx, self.pad_color(), PAD_RELEASED_BRIGHTNESS);
        };
    }

    fn encoder_step(&mut self, maschine: &mut dyn Maschine, idx: usize, state: i32) {
        self.send_osc_encoder_msg(maschine, idx, state);
    }

    fn button_down(
        &mut self,
        maschine: &mut dyn Maschine,
        btn: MaschineButton,
        byte: u8,
        is_down: bool,
    ) {
        //println!("{}", byte);
        self.send_osc_button_msg(maschine, btn, byte as usize, is_down);
    }

    fn button_up(
        &mut self,
        maschine: &mut dyn Maschine,
        btn: MaschineButton,
        byte: u8,
        is_down: bool,
    ) {
        self.send_osc_button_msg(maschine, btn, byte as usize, is_down);
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        usage(&args[0]);
        panic!("missing hidraw device path");
    }

    let dev_fd = match fcntl::open(
        Path::new(&args[1]),
        O_RDWR | O_NONBLOCK,
        sys::stat::Mode::empty(),
    ) {
        Err(err) => panic!("couldn't open {}: {}", args[1], err.errno().desc()),
        Ok(file) => file,
    };

    let osc_socket = UdpSocket::bind("127.0.0.1:42434").unwrap();

    let seq_handle = SequencerHandle::open("maschine.rs", HandleOpenStreams::Output).unwrap();
    let seq_port = seq_handle
        .create_port(
            "Pads MIDI",
            PortCapabilities::PORT_CAPABILITY_READ | PortCapabilities::PORT_CAPABILITY_SUBS_READ,
            PortType::MidiGeneric,
        )
        .unwrap();

    let mut dev = devices::mk2::Mikro::new(dev_fd);

    let mut handler = MHandler {
        color: HSL {
            h: 0.0,
            s: 1.0,
            l: 0.3,
        },

        seq_port: &seq_port,
        seq_handle: &seq_handle,

        pressure_shape: PressureShape::Exponential(0.4),
        send_aftertouch: false,

        osc_socket: &osc_socket,
        osc_outgoing_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 42435)),
    };

    dev.clear_screen();

    //Trying to draw stuff here
    if args.len() < 3 {
        dev.write_screen();
    } else {
        println!("RUNNING!")
    }
    //println!("{}", std::env::current_dir().unwrap().display());
    for i in 0..16 {
        dev.set_pad_light(i, handler.pad_color(), PAD_RELEASED_BRIGHTNESS);
    }

    ev_loop(&mut dev, &mut handler);
}
