#!/usr/bin/env sh

FILES=/dev/hidraw*
for f in $FILES
do
  FILE=${f##*/}
  DEVICE="$(cat /sys/class/hidraw/${FILE}/device/uevent | grep HID_NAME | cut -d '=' -f2)"
  printf "%s \t %s\n" $FILE "$DEVICE"
done


echo Which hidraw NUMBER is your Maschine MK2?
read hidraw
echo starting MK2

./target/release/maschine /dev/hidraw$hidraw
