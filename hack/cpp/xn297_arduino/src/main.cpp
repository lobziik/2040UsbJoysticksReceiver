#include <Arduino.h>

#include "xn297.h"


// Xn297      Arduino  Wire
//  pin        pin     color

//  IRQ         9      gray
//  MISO       12      green
//  MOSI       11      yellow
//  SCLK       13      violet
//  CSN         8      blue
//  CE          7      brown
//  VCC      3.3V      black
//  GND       GND      red

Xn297 radio;

static const uint64_t pipe0ls = 0xA793B455AA;
static const uint64_t pipe1ls = 0x81C6B2AA55;

uint8_t channels[4] = {0x4, 0x1D, 0x31, 0x4F};

void debug(const char* s) {
  Serial.println(s);
}

void onReceive() {

  uint8_t data[2];

  radio.readRxPayload(data, 2);
  Serial.println("DATA!!!!");

  Serial.print(data[0]);
  Serial.print(' ');
  Serial.print(data[1]);
  Serial.println();
  Serial.println("-------");
}

void setup() {
  Serial.begin(9600);
  Serial.println("hello");
  radio.setOnDebug(debug);
  radio.setOnReceive(onReceive);

  // irq, miso, mosi, sck, csn, ce
  radio.begin(9, 12, 11, 13, 8, 7);

  // radio.setRxAddressP0(pipe0ls);
  // radio.setRxAddressP1(pipe1ls);
  //radio.setCh(channels[curr_channel_i]);
}


void loop() {  
  radio.tick();
  delay(20);
}