#![no_std]

device_driver::create_device!(
    device_name: Rda5807m,
    manifest: "manifests/rda5807m.yaml"
);

impl<I, E> Rda5807m<I>
where
    I: device_driver::RegisterInterface<AddressType = u8, Error = E>,
{
    /// Power up the chip with audio outputs enabled and mute disabled.
    pub fn power_on(&mut self) -> Result<(), E> {
        self.general_control().modify(|w| {
            w.set_enable(true);
            w.set_dhiz(true);
            w.set_dmute(true);
            w.set_new_method(true);
        })
    }

    /// Power down the chip (low power mode).
    pub fn power_off(&mut self) -> Result<(), E> {
        self.general_control().modify(|w| {
            w.set_enable(false);
        })
    }

    /// Trigger a soft reset of the device.
    pub fn soft_reset(&mut self) -> Result<(), E> {
        self.general_control().modify(|w| {
            w.set_soft_reset(true);
        })
    }

    /// Set volume (0 to 15). Values above 15 will be clamped to 15.
    pub fn set_volume(&mut self, volume: u8) -> Result<(), E> {
        let vol = volume.min(15);
        self.volume_and_control().modify(|w| {
            w.set_volume(vol);
        })
    }

    /// Set mute state (true = muted, false = unmuted).
    pub fn set_mute(&mut self, mute: bool) -> Result<(), E> {
        self.general_control().modify(|w| {
            w.set_dmute(!mute);
        })
    }

    /// Enable or disable bass boost.
    pub fn set_bass_boost(&mut self, enable: bool) -> Result<(), E> {
        self.general_control().modify(|w| {
            w.set_bass(enable);
        })
    }

    /// Force mono reception (true = force mono, false = stereo enabled).
    pub fn set_mono(&mut self, force_mono: bool) -> Result<(), E> {
        self.general_control().modify(|w| {
            w.set_mono(force_mono);
        })
    }

    /// Set de-emphasis filter time constant (false = 75 µs for US/Japan, true = 50 µs for EU/AU).
    pub fn set_deemphasis(&mut self, deemphasis_50us: bool) -> Result<(), E> {
        self.audio_control().modify(|w| {
            w.set_deemphasis(deemphasis_50us);
        })
    }

    /// Enable or disable soft mute.
    pub fn set_softmute(&mut self, enable: bool) -> Result<(), E> {
        self.audio_control().modify(|w| {
            w.set_softmute_en(enable);
        })
    }

    /// Enable or disable RDS/RBDS decoding.
    pub fn set_rds_enable(&mut self, enable: bool) -> Result<(), E> {
        self.general_control().modify(|w| {
            w.set_rds_en(enable);
        })
    }

    /// Configure reference clock mode.
    pub fn set_clock_mode(&mut self, mode: ClkMode) -> Result<(), E> {
        self.general_control().modify(|w| {
            w.set_clk_mode(mode);
        })
    }

    /// Configure tuning band and channel spacing.
    pub fn set_band_and_spacing(&mut self, band: Band, space: Space) -> Result<(), E> {
        self.channel_tuning().modify(|w| {
            w.set_band(band);
            w.set_space(space);
        })
    }

    /// Tune to a specific frequency in kHz (e.g. 101100 for 101.1 MHz).
    pub fn set_frequency_khz(&mut self, freq_khz: u32) -> Result<(), E> {
        let tuning = self.channel_tuning().read()?;
        let band = tuning.band().unwrap_or(Band::UsaEurope);
        let space = tuning.space().unwrap_or(Space::KHz100);

        let (base_freq_khz, spacing_khz) = match (band, space) {
            (Band::UsaEurope, Space::KHz100) => (87000, 100),
            (Band::UsaEurope, Space::KHz200) => (87000, 200),
            (Band::UsaEurope, Space::KHz50) => (87000, 50),
            (Band::UsaEurope, Space::KHz25) => (87000, 25),

            (Band::Japan, Space::KHz100) => (76000, 100),
            (Band::Japan, Space::KHz200) => (76000, 200),
            (Band::Japan, Space::KHz50) => (76000, 50),
            (Band::Japan, Space::KHz25) => (76000, 25),

            (Band::WorldWide, Space::KHz100) => (76000, 100),
            (Band::WorldWide, Space::KHz200) => (76000, 200),
            (Band::WorldWide, Space::KHz50) => (76000, 50),
            (Band::WorldWide, Space::KHz25) => (76000, 25),

            (Band::EastEuropeOrExtended, Space::KHz100) => (65000, 100),
            (Band::EastEuropeOrExtended, Space::KHz200) => (65000, 200),
            (Band::EastEuropeOrExtended, Space::KHz50) => (65000, 50),
            (Band::EastEuropeOrExtended, Space::KHz25) => (65000, 25),
        };

        let chan = if freq_khz >= base_freq_khz {
            ((freq_khz - base_freq_khz) / spacing_khz)
                .try_into()
                .unwrap()
        } else {
            0.try_into().unwrap()
        };

        self.channel_tuning().modify(|w| {
            w.set_chan(chan);
            w.set_tune(true);
        })
    }

    /// Read current tuned frequency in kHz.
    pub fn get_frequency_khz(&mut self) -> Result<u32, E> {
        let tuning = self.channel_tuning().read()?;
        let status = self.status_1().read()?;

        let band = tuning.band().unwrap_or(Band::UsaEurope);
        let space = tuning.space().unwrap_or(Space::KHz100);

        let (base_freq_khz, spacing_khz) = match (band, space) {
            (Band::UsaEurope, Space::KHz100) => (87000, 100),
            (Band::UsaEurope, Space::KHz200) => (87000, 200),
            (Band::UsaEurope, Space::KHz50) => (87000, 50),
            (Band::UsaEurope, Space::KHz25) => (87000, 25),

            (Band::Japan, Space::KHz100) => (76000, 100),
            (Band::Japan, Space::KHz200) => (76000, 200),
            (Band::Japan, Space::KHz50) => (76000, 50),
            (Band::Japan, Space::KHz25) => (76000, 25),

            (Band::WorldWide, Space::KHz100) => (76000, 100),
            (Band::WorldWide, Space::KHz200) => (76000, 200),
            (Band::WorldWide, Space::KHz50) => (76000, 50),
            (Band::WorldWide, Space::KHz25) => (76000, 25),

            (Band::EastEuropeOrExtended, Space::KHz100) => (65000, 100),
            (Band::EastEuropeOrExtended, Space::KHz200) => (65000, 200),
            (Band::EastEuropeOrExtended, Space::KHz50) => (65000, 50),
            (Band::EastEuropeOrExtended, Space::KHz25) => (65000, 25),
        };

        let readchan: u32 = status.readchan().into();
        Ok(base_freq_khz + readchan * spacing_khz)
    }

    /// Trigger autonomous channel seeking.
    /// `seek_up`: true to seek upwards in frequency, false to seek downwards.
    /// `stop_at_limit`: true to stop seeking at band boundary, false to wrap around.
    pub fn seek(&mut self, seek_up: bool, stop_at_limit: bool) -> Result<(), E> {
        self.general_control().modify(|w| {
            w.set_seek(true);
            w.set_seekup(seek_up);
            w.set_sk_mode(stop_at_limit);
        })
    }

    /// Check if tune or seek operation is complete.
    pub fn is_seek_tune_complete(&mut self) -> Result<bool, E> {
        let status = self.status_1().read()?;
        Ok(status.stc())
    }

    /// Check if seek operation failed.
    pub fn is_seek_failed(&mut self) -> Result<bool, E> {
        let status = self.status_1().read()?;
        Ok(status.sf())
    }

    /// Read Received Signal Strength Indicator (RSSI) value (0..127).
    pub fn get_rssi(&mut self) -> Result<u8, E> {
        let status2 = self.status_2().read()?;
        Ok(status2.rssi())
    }

    /// Check if the currently tuned channel is a valid station.
    pub fn is_station(&mut self) -> Result<bool, E> {
        let status2 = self.status_2().read()?;
        Ok(status2.fm_true())
    }

    /// Check if current reception is stereo.
    pub fn is_stereo(&mut self) -> Result<bool, E> {
        let status1 = self.status_1().read()?;
        Ok(status1.st())
    }

    /// Read Chip ID (expected 0x58).
    pub fn read_chip_id(&mut self) -> Result<u8, E> {
        let chip_id_reg = self.chip_id().read()?;
        Ok(chip_id_reg.chip_id())
    }

    /// Read RDS blocks (Block A, Block B, Block C, Block D).
    pub fn read_rds_blocks(&mut self) -> Result<(u16, u16, u16, u16), E> {
        let block_a = self.rds_data_0().read()?.block_a();
        let block_b = self.rds_data_1().read()?.block_b();
        let block_c = self.rds_data_2().read()?.block_c();
        let block_d = self.rds_data_3().read()?.block_d();
        Ok((block_a, block_b, block_c, block_d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyInterface {
        regs: [u16; 16],
    }

    impl DummyInterface {
        fn new() -> Self {
            let mut regs = [0u16; 16];
            regs[0x00] = 0x5800; // ChipId
            Self { regs }
        }

        fn set_reg(&mut self, addr: usize, val: u16) {
            if addr < self.regs.len() {
                self.regs[addr] = val;
            }
        }
    }

    impl device_driver::RegisterInterface for DummyInterface {
        type AddressType = u8;
        type Error = ();

        fn write_register(
            &mut self,
            address: u8,
            _size_bits: u32,
            data: &[u8],
        ) -> Result<(), Self::Error> {
            if (address as usize) < self.regs.len() {
                self.regs[address as usize] = u16::from_be_bytes([data[0], data[1]]);
            }
            Ok(())
        }

        fn read_register(
            &mut self,
            address: u8,
            _size_bits: u32,
            data: &mut [u8],
        ) -> Result<(), Self::Error> {
            if (address as usize) < self.regs.len() {
                let bytes = self.regs[address as usize].to_be_bytes();
                data[0] = bytes[0];
                data[1] = bytes[1];
            } else {
                data[0] = 0;
                data[1] = 0;
            }
            Ok(())
        }
    }

    struct FailingInterface;

    impl device_driver::RegisterInterface for FailingInterface {
        type AddressType = u8;
        type Error = &'static str;

        fn write_register(
            &mut self,
            _address: u8,
            _size_bits: u32,
            _data: &[u8],
        ) -> Result<(), Self::Error> {
            Err("I2C bus error")
        }

        fn read_register(
            &mut self,
            _address: u8,
            _size_bits: u32,
            _data: &mut [u8],
        ) -> Result<(), Self::Error> {
            Err("I2C bus error")
        }
    }

    #[test]
    fn test_power_on_and_off() {
        let mut driver = Rda5807m::new(DummyInterface::new());

        driver.power_on().unwrap();
        let gc = driver.general_control().read().unwrap();
        assert!(gc.enable());
        assert!(gc.dhiz());
        assert!(gc.dmute());
        assert!(gc.new_method());

        driver.power_off().unwrap();
        let gc = driver.general_control().read().unwrap();
        assert!(!gc.enable());
    }

    #[test]
    fn test_soft_reset() {
        let mut driver = Rda5807m::new(DummyInterface::new());
        driver.soft_reset().unwrap();
        let gc = driver.general_control().read().unwrap();
        assert!(gc.soft_reset());
    }

    #[test]
    fn test_volume_control_and_clamping() {
        let mut driver = Rda5807m::new(DummyInterface::new());

        driver.set_volume(0).unwrap();
        assert_eq!(driver.volume_and_control().read().unwrap().volume(), 0);

        driver.set_volume(15).unwrap();
        assert_eq!(driver.volume_and_control().read().unwrap().volume(), 15);

        // Clamping check for values > 15
        driver.set_volume(25).unwrap();
        assert_eq!(driver.volume_and_control().read().unwrap().volume(), 15);
    }

    #[test]
    fn test_audio_features() {
        let mut driver = Rda5807m::new(DummyInterface::new());

        // Mute
        driver.set_mute(true).unwrap();
        assert!(!driver.general_control().read().unwrap().dmute());
        driver.set_mute(false).unwrap();
        assert!(driver.general_control().read().unwrap().dmute());

        // Bass boost
        driver.set_bass_boost(true).unwrap();
        assert!(driver.general_control().read().unwrap().bass());

        // Force mono
        driver.set_mono(true).unwrap();
        assert!(driver.general_control().read().unwrap().mono());

        // De-emphasis
        driver.set_deemphasis(true).unwrap();
        assert!(driver.audio_control().read().unwrap().deemphasis());

        // Softmute
        driver.set_softmute(true).unwrap();
        assert!(driver.audio_control().read().unwrap().softmute_en());
    }

    #[test]
    fn test_clock_mode_setting() {
        let mut driver = Rda5807m::new(DummyInterface::new());
        driver.set_clock_mode(ClkMode::MHz12).unwrap();
        assert_eq!(
            driver.general_control().read().unwrap().clk_mode().unwrap(),
            ClkMode::MHz12
        );
    }

    #[test]
    fn test_frequency_tuning_usa_europe() {
        let mock = DummyInterface::new();
        let mut driver = Rda5807m::new(mock);

        driver
            .set_band_and_spacing(Band::UsaEurope, Space::KHz100)
            .unwrap();

        // 101.1 MHz (101100 kHz) -> CHAN = (101100 - 87000) / 100 = 141
        driver.set_frequency_khz(101100).unwrap();
        let ct = driver.channel_tuning().read().unwrap();
        assert_eq!(ct.chan(), 141);
        assert!(ct.tune());

        // Mock status1 readchan return value = 141
        driver.interface().set_reg(0x0A, 141);
        assert_eq!(driver.get_frequency_khz().unwrap(), 101100);
    }

    #[test]
    fn test_frequency_tuning_japan() {
        let mut driver = Rda5807m::new(DummyInterface::new());

        driver
            .set_band_and_spacing(Band::Japan, Space::KHz100)
            .unwrap();

        // 80.0 MHz (80000 kHz) on Japan band (76.0 MHz base) -> CHAN = (80000 - 76000) / 100 = 40
        driver.set_frequency_khz(80000).unwrap();
        let ct = driver.channel_tuning().read().unwrap();
        assert_eq!(ct.chan(), 40);

        driver.interface().set_reg(0x0A, 40);
        assert_eq!(driver.get_frequency_khz().unwrap(), 80000);
    }

    #[test]
    fn test_frequency_tuning_worldwide() {
        let mut driver = Rda5807m::new(DummyInterface::new());

        driver
            .set_band_and_spacing(Band::WorldWide, Space::KHz200)
            .unwrap();

        // 96.0 MHz (96000 kHz) with 200 kHz spacing (76.0 MHz base) -> CHAN = (96000 - 76000) / 200 = 100
        driver.set_frequency_khz(96000).unwrap();
        let ct = driver.channel_tuning().read().unwrap();
        assert_eq!(ct.chan(), 100);

        driver.interface().set_reg(0x0A, 100);
        assert_eq!(driver.get_frequency_khz().unwrap(), 96000);
    }

    #[test]
    fn test_frequency_tuning_east_europe() {
        let mut driver = Rda5807m::new(DummyInterface::new());

        driver
            .set_band_and_spacing(Band::EastEuropeOrExtended, Space::KHz50)
            .unwrap();

        // 67.0 MHz (67000 kHz) with 50 kHz spacing (65.0 MHz base) -> CHAN = (67000 - 65000) / 50 = 40
        driver.set_frequency_khz(67000).unwrap();
        let ct = driver.channel_tuning().read().unwrap();
        assert_eq!(ct.chan(), 40);

        driver.interface().set_reg(0x0A, 40);
        assert_eq!(driver.get_frequency_khz().unwrap(), 67000);
    }

    #[test]
    fn test_seek_operation() {
        let mut driver = Rda5807m::new(DummyInterface::new());

        driver.seek(true, true).unwrap();
        let gc = driver.general_control().read().unwrap();
        assert!(gc.seek());
        assert!(gc.seekup());
        assert!(gc.sk_mode());

        driver.seek(false, false).unwrap();
        let gc = driver.general_control().read().unwrap();
        assert!(gc.seek());
        assert!(!gc.seekup());
        assert!(!gc.sk_mode());
    }

    #[test]
    fn test_status_queries() {
        let mut driver = Rda5807m::new(DummyInterface::new());

        // Mock Status1 (0x0A): STC=bit 14, SF=bit 13, ST=bit 10
        // 0b0110_0100_0000_0000 = 0x6400 -> STC=1, SF=1, ST=1
        driver.interface().set_reg(0x0A, 0x6400);

        assert!(driver.is_seek_tune_complete().unwrap());
        assert!(driver.is_seek_failed().unwrap());
        assert!(driver.is_stereo().unwrap());

        // Mock Status2 (0x0B): RSSI=bits 15:9 (e.g. 85), FM_TRUE=bit 8
        // (85 << 9) | (1 << 8) = 0xAABC
        driver.interface().set_reg(0x0B, (85 << 9) | (1 << 8));

        assert_eq!(driver.get_rssi().unwrap(), 85);
        assert!(driver.is_station().unwrap());
    }

    #[test]
    fn test_rds_operations() {
        let mut driver = Rda5807m::new(DummyInterface::new());

        driver.set_rds_enable(true).unwrap();
        assert!(driver.general_control().read().unwrap().rds_en());

        // Mock RDS data registers 0x0C, 0x0D, 0x0E, 0x0F
        driver.interface().set_reg(0x0C, 0x1234);
        driver.interface().set_reg(0x0D, 0x5678);
        driver.interface().set_reg(0x0E, 0x9ABC);
        driver.interface().set_reg(0x0F, 0xDEF0);

        let (a, b, c, d) = driver.read_rds_blocks().unwrap();
        assert_eq!(a, 0x1234);
        assert_eq!(b, 0x5678);
        assert_eq!(c, 0x9ABC);
        assert_eq!(d, 0xDEF0);
    }

    #[test]
    fn test_chip_id() {
        let mut driver = Rda5807m::new(DummyInterface::new());
        assert_eq!(driver.read_chip_id().unwrap(), 0x58);
    }

    #[test]
    fn test_error_propagation() {
        let mut driver = Rda5807m::new(FailingInterface);

        assert!(driver.power_on().is_err());
        assert!(driver.get_rssi().is_err());
        assert!(driver.read_chip_id().is_err());
    }

    #[test]
    fn test_extended_registers_rw() {
        let mut driver = Rda5807m::new(DummyInterface::new());

        // Test Register 0x06 (OpenReservedMode)
        driver
            .open_reserved_mode()
            .modify(|w| {
                w.set_open_mode(0b11);
            })
            .unwrap();
        assert_eq!(
            driver.open_reserved_mode().read().unwrap().open_mode(),
            0b11
        );

        // Test Register 0x07 (SystemControl2)
        driver
            .system_control_2()
            .modify(|w| {
                w.set_softblend_en(true);
                w.set_mode_65_m_50_m(true);
            })
            .unwrap();
        let sc2 = driver.system_control_2().read().unwrap();
        assert!(sc2.softblend_en());
        assert!(sc2.mode_65_m_50_m());
    }
}
