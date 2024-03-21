# RP2040 + XN297L based noname joysticks receiver

## Description
This program was developed for specific purpose of connecting wireless no-name joysticks to computer
via USB. Such joysticks and a 'gaming console' which was together with them are using XN297L transceiver chip for one way 'joysticks -> console' communication.

Specific parameters for XN297L configuration were dumped from the 'gaming console' with help of logic analyzer device. 

Program in this repository targeting RP2040 based dev-board with attached XN297L transceiver. Communication with XN297L happens with SPI protocol.

This software was build with above described purpose, and was not intended to be used as a library of any kind. Interfaces are not 
well-defined and a lot of things were short-cutted there.

## Boards and branches

Main target is the [Waveshare RP2040-Zero](https://www.waveshare.com/rp2040-zero.htm)
and code in the `master` branch is intended to run on this board. However, due to lack of debugging capabilities
big chunk of development was done on [Raspberry Pi Pico](https://www.raspberrypi.com/products/raspberry-pi-pico/) board.
Code which compiles and runs on rp-pico is living in the respective [branch](https://github.com/lobziik/2040UsbReceiver/tree/rp-pico)
There are some minor differences between these boards, mainly in pin layout.

## TODO

- ~~SPI communication with xn297l~~
- ~~Translation of bytes coming from spi to properly formed `JoystickReport` struct~~
- Write up an article describing entire adventure
  - ~~add logic2 plugin there~~
  - ~~add arduino c++ version of xn297l polling firmware~~
  - pictures? 