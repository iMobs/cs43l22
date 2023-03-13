#![no_std]

use embedded_hal::blocking::i2c;

#[derive(Clone, Copy, PartialEq)]
pub enum OutputDevice {
    Speaker,
    Headphone,
    Both,
    Auto,
}

impl OutputDevice {
    fn value(self) -> u8 {
        match self {
            Self::Speaker => 0xFA,
            Self::Headphone => 0xAF,
            Self::Both => 0xAA,
            Self::Auto => 0x05,
        }
    }
}

enum Audio {
    Pause = 0,
    Resume = 1,
}

enum PowerDownMode {
    Hardware = 1,
    Software = 2,
}

enum Mute {
    On = 1,
    Off = 0,
}

pub struct CS43L22<Bus> {
    address: u8,
    bus: Bus,
}

impl<Bus, I2CError> CS43L22<Bus>
where
    Bus: i2c::WriteRead<u8, Error = I2CError>,
{
    pub fn new(
        bus: Bus,
        address: u8,
        output_device: OutputDevice,
        volume: u8,
    ) -> Result<Self, I2CError> {
        // let mut counter = 0;

        let mut cs43l22 = Self { address, bus };

        // POWER_CTL1
        cs43l22.write(&[0x02, 0x01])?;
        // POWER_CTL2
        cs43l22.write(&[0x04, output_device.value()])?;

        // clock config auto detect
        cs43l22.write(&[0x05, 0x81])?;

        // slave mode and audio standard
        cs43l22.write(&[0x06, 0x04])?;

        // set master volume
        cs43l22.set_volume(volume)?;

        // if speaker is enabled, set mono mode and volume attenuation level
        if output_device != OutputDevice::Headphone {
            // set speaker mono mode
            cs43l22.write(&[0x0F, 0x06])?;

            // set speaker attenuation
            cs43l22.write(&[0x24, 0x00])?;
            cs43l22.write(&[0x25, 0x00])?;
        }

        /* Additional configuration for the CODEC. These configurations are done to reduce
        the time needed for the Codec to power off. If these configurations are removed,
        then a long delay should be added between powering off the Codec and switching
        off the I2S peripheral MCLK clock (which is the operating clock for Codec).
        If this delay is not inserted, then the codec will not shut down properly and
        it results in high noise after shut down. */

        /* Disable the analog soft ramp */
        cs43l22.write(&[0x0A, 0x00])?;
        /* Disable the digital soft ramp */
        cs43l22.write(&[0x0E, 0x04])?;
        /* Disable the limiter attack level */
        //counter += CODEC_IO_Write(DeviceAddr, CS43L22_REG_LIMIT_CTL1, 0x00);
        /* Adjust Bass and Treble levels */
        cs43l22.write(&[0x1F, 0x0F])?;
        /* Adjust PCM volume level */
        cs43l22.write(&[0x1A, 0x0A])?;
        cs43l22.write(&[0x1B, 0x0A])?;

        Ok(cs43l22)
    }

    pub fn release(self) -> Bus {
        let Self { bus, .. } = self;
        bus
    }

    pub fn set_volume(&mut self, volume: u8) -> Result<(), I2CError> {
        let volume = scale_volume(volume);

        if volume > 0xE6 {
            self.write(&[0x20, volume - 0xE7])?;
            self.write(&[0x21, volume - 0xE7])?;
        } else {
            self.write(&[0x20, volume + 0x19])?;
            self.write(&[0x21, volume + 0x19])?;
        }

        Ok(())
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), I2CError> {
        let mut result = [0];

        self.bus.write_read(self.address, bytes, &mut result)?;

        if result[0] == 0 {
            Ok(())
        } else {
            todo!()
        }
    }
}

fn scale_volume(volume: u8) -> u8 {
    ((volume as usize) * 255 / 100).min(0xFF) as u8
}
