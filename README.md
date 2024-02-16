# RP2040 + XN297L based noname joysticks receiver

Currently this repo contains rust code and additional files for build and develop firmware
for RP2040 based boards with attached XN297L transceiver chip.


## TODO

- SPI communication with xn297l
- Translation of bytes coming from spi to properly formed `JoystickReport` struct
- Write up an article describing entire adventure
  - add logic2 plugin there
  - add arduino c++ version of xn297l polling firmware (there are some caveats in configuration)
  - pictures? 