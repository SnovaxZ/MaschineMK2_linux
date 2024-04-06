#!/usr/bin/env sh
echo Specify brightness 1 - 127
read b
echo Writing lights!
oscsend localhost 42434 /maschine/button/f1 i $b
oscsend localhost 42434 /maschine/button/f2 i $b
oscsend localhost 42434 /maschine/button/f3 i $b
oscsend localhost 42434 /maschine/button/f4 i $b
oscsend localhost 42434 /maschine/button/f5 i $b
oscsend localhost 42434 /maschine/button/f6 i $b
oscsend localhost 42434 /maschine/button/f7 i $b
oscsend localhost 42434 /maschine/button/f8 i $b
oscsend localhost 42434 /maschine/button/auto i $b
oscsend localhost 42434 /maschine/button/all i $b
oscsend localhost 42434 /maschine/button/page_left i $b
oscsend localhost 42434 /maschine/button/page_right i $b
oscsend localhost 42434 /maschine/button/swing i $b
oscsend localhost 42434 /maschine/button/volume i $b
oscsend localhost 42434 /maschine/button/tempo i $b
oscsend localhost 42434 /maschine/button/all i $b
oscsend localhost 42434 /maschine/button/navigate i $b
oscsend localhost 42434 /maschine/button/auto i $b
oscsend localhost 42434 /maschine/button/browse i $b
oscsend localhost 42434 /maschine/button/sampling i $b
oscsend localhost 42434 /maschine/button/note_repeat i $b
oscsend localhost 42434 /maschine/button/step i $b
oscsend localhost 42434 /maschine/button/restart i $b
oscsend localhost 42434 /maschine/button/rec i $b
oscsend localhost 42434 /maschine/button/step_left i $b
oscsend localhost 42434 /maschine/button/step_right i $b
oscsend localhost 42434 /maschine/button/grid i $b
oscsend localhost 42434 /maschine/button/play i $b
oscsend localhost 42434 /maschine/button/stop i $b
oscsend localhost 42434 /maschine/button/shift i $b
oscsend localhost 42434 /maschine/button/control i $b
oscsend localhost 42434 /maschine/button/mute i $b
oscsend localhost 42434 /maschine/button/solo i $b
oscsend localhost 42434 /maschine/button/select i $b
oscsend localhost 42434 /maschine/button/duplicate i $b
oscsend localhost 42434 /maschine/button/pad_mode i $b
oscsend localhost 42434 /maschine/button/pattern i $b
oscsend localhost 42434 /maschine/button/scene i $b
oscsend localhost 42434 /maschine/button/group_a i $b
oscsend localhost 42434 /maschine/button/group_b i $b
oscsend localhost 42434 /maschine/button/group_c i $b
oscsend localhost 42434 /maschine/button/group_d i $b
oscsend localhost 42434 /maschine/button/group_e i $b
oscsend localhost 42434 /maschine/button/group_f i $b
oscsend localhost 42434 /maschine/button/group_g i $b
oscsend localhost 42434 /maschine/button/group_h i $b
oscsend localhost 42434 /maschine/button/enter i $b
oscsend localhost 42434 /maschine/button/erase i $b
oscsend localhost 42434 /maschine/button/nav_left i $b
oscsend localhost 42434 /maschine/button/nav_right i $b
oscsend localhost 42434 /maschine/button/enter i $b

echo Done!
