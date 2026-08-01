use rda5807m_rs::{Band, ClkMode, Rda5807m, Space};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
struct SharedMockI2cBus {
    registers: Rc<RefCell<[u16; 16]>>,
}

impl SharedMockI2cBus {
    fn new() -> Self {
        let mut regs = [0u16; 16];
        regs[0x00] = 0x5800; // Chip ID 0x58
        Self {
            registers: Rc::new(RefCell::new(regs)),
        }
    }

    fn set_reg(&self, addr: usize, val: u16) {
        let mut regs = self.registers.borrow_mut();
        if addr < regs.len() {
            regs[addr] = val;
        }
    }
}

impl device_driver::RegisterInterface for SharedMockI2cBus {
    type AddressType = u8;
    type Error = ();

    fn write_register(
        &mut self,
        address: u8,
        _size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let mut regs = self.registers.borrow_mut();
        if (address as usize) < regs.len() {
            regs[address as usize] = u16::from_be_bytes([data[0], data[1]]);
        }
        Ok(())
    }

    fn read_register(
        &mut self,
        address: u8,
        _size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        let regs = self.registers.borrow();
        if (address as usize) < regs.len() {
            let bytes = regs[address as usize].to_be_bytes();
            data[0] = bytes[0];
            data[1] = bytes[1];
        } else {
            data[0] = 0;
            data[1] = 0;
        }
        Ok(())
    }
}

#[test]
fn test_full_fm_tuner_workflow() {
    let mock_bus = SharedMockI2cBus::new();
    let mut driver = Rda5807m::new(mock_bus.clone());

    // 1. Verify Chip ID
    assert_eq!(driver.read_chip_id().unwrap(), 0x58);

    // 2. Power on device
    driver.power_on().unwrap();
    assert!(driver.general_control().read().unwrap().enable());

    // 3. Configure Clock Mode
    driver.set_clock_mode(ClkMode::KHz32768).unwrap();

    // 4. Configure Band and Spacing
    driver
        .set_band_and_spacing(Band::UsaEurope, Space::KHz100)
        .unwrap();

    // 5. Set Volume and Audio features
    driver.set_volume(10).unwrap();
    assert_eq!(driver.volume_and_control().read().unwrap().volume(), 10);

    driver.set_bass_boost(true).unwrap();
    assert!(driver.general_control().read().unwrap().bass());

    driver.set_deemphasis(false).unwrap(); // 75 µs
    assert!(!driver.audio_control().read().unwrap().deemphasis());

    // 6. Tune to 101.1 MHz (101100 kHz)
    driver.set_frequency_khz(101100).unwrap();
    let chan = driver.channel_tuning().read().unwrap().chan();
    assert_eq!(chan, 141);

    // Simulate tuner locked on channel 141 with strong RSSI (90) and Stereo
    mock_bus.set_reg(0x0A, (1 << 14) | (1 << 10) | 141); // STC=1, ST=1, READCHAN=141
    mock_bus.set_reg(0x0B, (90 << 9) | (1 << 8)); // RSSI=90, FM_TRUE=1

    // 7. Verify status and tuned frequency
    assert!(driver.is_seek_tune_complete().unwrap());
    assert!(driver.is_stereo().unwrap());
    assert!(driver.is_station().unwrap());
    assert_eq!(driver.get_rssi().unwrap(), 90);
    assert_eq!(driver.get_frequency_khz().unwrap(), 101100);

    // 8. Enable RDS and read blocks
    driver.set_rds_enable(true).unwrap();
    assert!(driver.general_control().read().unwrap().rds_en());

    mock_bus.set_reg(0x0C, 0xABCD);
    mock_bus.set_reg(0x0D, 0x1234);
    mock_bus.set_reg(0x0E, 0x5678);
    mock_bus.set_reg(0x0F, 0x90EF);

    let (a, b, c, d) = driver.read_rds_blocks().unwrap();
    assert_eq!((a, b, c, d), (0xABCD, 0x1234, 0x5678, 0x90EF));

    // 9. Power off
    driver.power_off().unwrap();
    assert!(!driver.general_control().read().unwrap().enable());
}
