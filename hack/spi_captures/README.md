# SPI captures

SPI communication captures made with cheap logic analyzer compatible with [Saleae Logic2](https://www.saleae.com/pages/downloads).

Explored chip is [Panchip XN297LBW](https://www.panchip.com/static/upload/file/20190916/1568621331607821.pdf) which uses only 3 pins for SPI communication. 

Captures made from the moment of device boot and lasts approx 30s.

# Wiring

For detailed chip pinout see datasheets in archive. 

| Analyzer channel | chip pin | datasheet description   | Spi analyzer |
|------------------|----------|-------------------------|--------------|
| Ch0              | 1        | CSN / SPI Chip Select   | Enable       |
| Ch1              | 2        | SCK / SPI Clock         | Enable       |
| Ch2              | 3        | DATA / SPI slave in-out | Enable       |


# Decoding SPI

Logic2 plugin is placed in '[analyzer](analyzer)' folder next to captures.