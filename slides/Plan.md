Plan

1. Intro
   * what it will be about
     1. Fun hobby project
     2. State overview and personal impressions about Rust for MCUs
   * Console + joysticks overview
   * Set goals
   * Exploration phase, transceiver located, datasheet found
     * NRF24 and its clones
   * What is SPI in two words?
   * Decoding SPI 
   * Validated theory, got transmission
     * xn297, not compatible
2. To the device
   * Show the thing
   * RP2040, rp-pico, debugging, picoprobe
   * Rust
      - What it is in one slide
      - Why is it cool? Of course CARGO!
      - Current progress of rust-embedded community
        - embedded-hal and no-std
        - Traits and BSPs are cool
      - Platforms?
          - Various ARMs (RPi, Stm32), Avr is also there.
      - Production ready? Not yet, but seems soon!
      - materials?
          - https://docs.rust-embedded.org/book/ - book
          - https://docs.rust-embedded.org/embedonomicon/preface.html

Questions for S.

What SPI stand for in context of the talk?:
 - Stateful Packet Inspection
 - Schedule Performance Index
 - Serial Peripheral Interface

What can help with capturing digital communication from the wire?:
- Logic Analyzer
- LCR Meter
- Mielophone

What is 'Cargo'?:
- name of the library for wireless communication
- Dependency and toolchain manager within Rust ecosystem
- Cult

