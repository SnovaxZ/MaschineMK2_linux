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

use std::os::unix::io::RawFd;

#[derive(Copy, Clone, Debug)]
pub enum MaschineButton {
    F8,
    F7,
    F6,
    F5,
    F4,
    F3,
    F2,
    F1,

    Auto,
    All,
    Pageleft,
    Pageright,

    Sampling,

    Nav,
    Noterepeat,
    Enter,
    Navright,
    Navleft,
    Tempo,
    Swing,
    Volume,

    GroupH,
    GroupG,
    GroupF,
    GroupE,
    GroupD,
    GroupC,
    GroupB,
    GroupA,

    Shift,
    Erase,
    Rec,
    Play,
    Grid,
    Stepright,
    Stepleft,
    Restart,

    Mute,
    Solo,
    Select,
    Duplicate,
    Navigate,
    Padmode,
    Pattern,
    Scene,
    Browse,
    Step,
    Control,
    Encoder,
    Main,
    View,

    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,

    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    A8,

    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,

    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,

    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,

    E1,
    E2,
    E3,
    E4,
    E5,
    E6,
    E7,
    E8,

    FF1,
    FF2,
    FF3,
    FF4,
    FF5,
    FF6,
    FF7,
    FF8,

    G1,
    G2,
    G3,
    G4,
    G5,
    G6,
    G7,
    G8,

    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    H7,
    H8,

    I1,
    I2,
    I3,
    I4,
    I5,
    I6,
    I7,
    I8,

    J1,
    J2,
    J3,
    J4,
    J5,
    J6,
    J7,
    J8,

    K1,
    K2,
    K3,
    K4,
    K5,
    K6,
    K7,
    K8,

    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
    L8,

    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,

    M1,
    M2,
    M3,
    M4,
    M5,
    M6,
    M7,
    M8,

    O1,
    O2,
    O3,
    O4,
    O5,
    O6,
    O7,
    O8,

    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
    P8,


}
pub trait Maschine {
    fn get_fd(&self) -> RawFd;

    fn get_pad_pressure(&self, pad_idx: usize) -> Result<f32, ()>;

    fn get_midi_note_base(&self) -> u8;
    fn set_midi_note_base(&mut self, base: u8);

    fn set_roller_state(&mut self, state: usize, idx: usize);
    fn get_roller_state(&self, idx: usize) -> usize;

    fn set_pad_light(&mut self, pad_idx: usize, color: u32, brightness: f32);
    fn set_button_light(&mut self, btn: MaschineButton, color: u32, brightness: f32);

    fn set_mod(&mut self, state: usize);
    fn get_mod(&self) -> usize;

    fn set_padmode(&mut self, state: usize);
    fn get_padmode(&self) -> usize;

    fn readable(&mut self, _: &mut dyn MaschineHandler);

    fn clear_screen(&mut self);
    fn write_lights(&mut self);
    fn write_screen(&mut self);
}

#[allow(unused_variables)]
pub trait MaschineHandler {
    fn pad_pressed(&mut self, _: &mut dyn Maschine, pad_idx: usize, pressure: f32) {}
    fn pad_aftertouch(&mut self, _: &mut dyn Maschine, pad_idx: usize, pressure: f32) {}
    fn pad_released(&mut self, _: &mut dyn Maschine, pad_idx: usize) {}

    fn encoder_step(&mut self, _: &mut dyn Maschine, encoder_idx: usize, delta: i32) {}

    fn button_down(&mut self, _: &mut dyn Maschine, button: MaschineButton, byte: u8, is_down: bool) {}
    fn button_up(&mut self, _: &mut dyn Maschine, button: MaschineButton, byte: u8, is_down: bool) {}

    fn read_input(&mut self, _: &mut dyn Maschine) {}
}
