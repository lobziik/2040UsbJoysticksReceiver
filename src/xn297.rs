use core::convert::Infallible;

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

pub struct Xn297L<SPI, CSn, CE>
where
    SPI: Transfer<u8>,
    CSn: OutputPin<Error = Infallible>,
    CE: OutputPin<Error = Infallible>,
{
    spi: SPI,
    csn_pin: CSn,
    ce_pin: CE,
}

const REG_WRITE_CMD: u8 = 0x20;

impl<SPI, CSn, CE> Xn297L<SPI, CSn, CE>
where
    SPI: Transfer<u8>,
    CSn: OutputPin<Error = Infallible>,
    CE: OutputPin<Error = Infallible>,
{
    pub fn new(spi: SPI, csn_pin: CSn, ce_pin: CE) -> Xn297L<SPI, CSn, CE> {
        Xn297L {
            spi,
            csn_pin,
            ce_pin,
        }
    }

    pub fn set_ce_high(&mut self) {
        self.ce_pin.set_high().unwrap()
    }

    pub fn set_ce_low(&mut self) {
        self.ce_pin.set_low().unwrap()
    }

    fn spi_transfer<'a>(&'a mut self, words: &'a mut [u8]) -> Result<&[u8], SPI::Error> {
        let _ = self.csn_pin.set_low();
        let result = self.spi.transfer(words)?;
        let _ = self.csn_pin.set_high();
        Ok(result)
    }

    pub fn read_register<const REG_LEN: usize>(
        &mut self,
        register: u8,
    ) -> Result<[u8; REG_LEN], SPI::Error> {
        let mut words: [u8; REG_LEN] = [0; REG_LEN];
        words[0] = register;
        match self.spi_transfer(words.as_mut()) {
            Ok(_) => Ok(words),
            Err(e) => Err(e),
        }
    }

    pub fn write_register<const REG_LEN: usize>(
        &mut self,
        data: [u8; REG_LEN],
    ) -> Result<[u8; REG_LEN], SPI::Error> {
        let mut to_send = data;
        to_send[0] |= REG_WRITE_CMD;
        match self.spi_transfer(to_send.as_mut()) {
            Ok(_) => Ok(to_send),
            Err(e) => Err(e),
        }
    }

    // TODO PAYLOAD_SIZE lies now, should be +1 because register number is a part of spi transmission
    pub fn read_rx_payload<const PAYLOAD_SIZE: usize>(
        &mut self,
    ) -> Result<Option<[u8; PAYLOAD_SIZE]>, SPI::Error> {
        let [_, reg_value] = self.read_register::<2>(0x07)?;

        if (reg_value & (1 << 6)) > 0 {
            match self.read_register::<PAYLOAD_SIZE>(0x61) {
                Ok(pl) => {
                    // clean 6th bit in 0x07 register, we've got transmission
                    self.write_register::<2>([0x07, reg_value | (1 << 6)])?;
                    Ok(Some(pl))
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(None)
        }
    }

    pub fn init(&mut self) -> Result<(), SPI::Error> {
        self.set_ce_low();

        // Power on
        self.write_register::<2>([0x01, 0x8E])?;

        // Clear
        self.write_register::<2>([0x07, 0x70])?;

        // Set ce controlled by pin
        self.write_register::<2>([0x1D, 0x0])?;

        // Set BB_CAL
        self.write_register::<6>([0x1F, 0x0A, 0x6D, 0x67, 0x9C, 0x46])?;

        // Set RF_CAL
        self.write_register::<4>([0x1E, 0xF6, 0x37, 0x5D])?;

        // Set DEMOD_CAL
        self.write_register::<2>([0x19, 0x1])?;

        // Set RF_CAL2
        self.write_register::<7>([0x1A, 0x45, 0x21, 0xEF, 0x2C, 0x5A, 0x40])?;

        // Set DEMOD_CAL2
        self.write_register::<4>([0x1B, 0x0B, 0xDF, 0x02])?;

        self.write_register::<2>([0x01, 0x03])?; // auto ack
        self.write_register::<2>([0x02, 0x03])?; // data pipe 0 and 1 enable
        self.write_register::<2>([0x03, 0x03])?; // addr width
        self.write_register::<2>([0x04, 0x02])?; // auto retransmit
        self.write_register::<2>([0x06, 0x3F])?; // data rate 1mbps

        self.write_register::<2>([0x11, 0x02])?; // payload length pipe 0
        self.write_register::<2>([0x12, 0x02])?; // payload length pipe 1
        self.write_register::<2>([0x1C, 0x00])?; // dynamic payload length disabled

        self.write_register::<6>([0x0A, 0xA7, 0x93, 0xB4, 0x55, 0xAA])?; // set addr pipe 0
        self.write_register::<6>([0x0B, 0x81, 0xC6, 0xB2, 0xAA, 0x55])?; // set addr pipe 1

        self.write_register::<2>([0x5, 0x31])?; // set channel 49 (dec)

        self.write_register::<2>([0x00, 0x8F])?; // RX on

        self.set_ce_high();

        Ok(())
    }
}
