#![feature(phase)]
#[phase(plugin, link)]
extern crate log;

#[deriving(Copy, Eq, PartialEq, Clone, Show)]
enum State {
    Idle,
    SPIEnable, 
    SPITransfer,
    SPISetClockDivisor,
    SPISetMode,
    SPISetRole,
    SPISetFrame,
    I2CEnable,
    I2CWrite,
    I2CRead,
    I2CSetSlaveAddress,
    I2CSetMode,
    UARTEnable,
    UARTTransfer,
    UARTReceive,
    UARTSetBaudrate,
    UARTSetDataBits,
    UARTSetParity,
    UARTSetStopBits,
    ExpectRepeatCommand,
}

mod commands {
    // Base Addr
    pub const CMD_BASE: u8 =                0x80;

    // General Ops
    pub const CMD_NOP: u8 =                 0x00;
    pub const CMD_SLEEP: u8 =               0x10 | CMD_BASE;

    // SPI
    pub const CMD_SPIENABLE: u8 =           0x20 | CMD_BASE;
    pub const CMD_SPITRANSFER: u8 =         0x21 | CMD_BASE;
    pub const CMD_SPIDISABLE: u8 =          0x22 | CMD_BASE;
    pub const CMD_SPISETCLOCKDIVISOR: u8 =  0x23 | CMD_BASE;
    pub const CMD_SPISETMODE: u8 =          0x24 | CMD_BASE;
    pub const CMD_SPISETROLE: u8 =          0x25 | CMD_BASE;
    pub const CMD_SPISETFRAME: u8 =         0x26 | CMD_BASE;

    // I2C
    pub const CMD_I2CENABLE: u8 =           0x30 | CMD_BASE;
    pub const CMD_I2CWRITE: u8 =            0x31 | CMD_BASE;
    pub const CMD_I2CREAD: u8 =             0x32 | CMD_BASE;
    pub const CMD_I2CDISABLE: u8 =          0x33 | CMD_BASE;
    pub const CMD_I2CSETMODE: u8 =          0x34 | CMD_BASE;
    pub const CMD_I2CSETSLAVEADDRESS: u8 =  0x35 | CMD_BASE;

    // UART
    pub const CMD_UARTENABLE: u8 =          0x40 | CMD_BASE;
    pub const CMD_UARTTRANSFER: u8 =        0x41 | CMD_BASE;
    pub const CMD_UARTRECEIVE: u8 =         0x42 | CMD_BASE;
    pub const CMD_UARTDISABLE: u8 =         0x43 | CMD_BASE;
    pub const CMD_UARTSETBAUDRATE: u8 =     0x44 | CMD_BASE;
    pub const CMD_UARTSETDATABITS: u8 =     0x45 | CMD_BASE;
    pub const CMD_UARTSETPARITY: u8 =       0x46 | CMD_BASE;
    pub const CMD_UARTSETSTOPBITS: u8 =     0x47 | CMD_BASE;

}

trait SPI {
    fn enable(&mut self);
    fn transfer(&mut self, byte: u8) -> u8;
    fn disable(&mut self);
    fn setClockSpeedDivisor(&mut self, divisor: u8);
    fn setMode(&mut self, mode: u8);
    fn setRole(&mut self, role: u8);
    fn setFrame(&mut self, frame: u8);
}

trait I2C {
    fn enable(&mut self);
    fn write(&mut self, byte: u8);
    fn read(&mut self) -> u8;
    fn disable(&mut self);
    fn setSlaveAddress(&mut self, address: u8);
    fn setMode(&mut self, mode: u8);
}

trait UART {
    fn enable(&mut self);
    fn transfer(&mut self, byte: u8);
    fn disable(&mut self);
    fn setBaudrate(&mut self, baudrate: u8);
    fn setDataBits(&mut self, data_bits: u8);
    fn setParity(&mut self, parity: u8);
    fn setStopBits(&mut self, stop_bits: u8);
}

struct IOStateMachine<'a, SPIT: 'a, I2CT: 'a, UARTT: 'a> {
    state: State,
    repeat_remaining: u8,
    spi: &'a mut SPIT,
    i2c: &'a mut I2CT,
    uart: &'a mut UARTT,
}

impl<'a, SPIT, I2CT, UARTT> IOStateMachine<'a, SPIT, I2CT, UARTT> where SPIT: SPI, I2CT: I2C, UARTT: UART {

    fn is_repeat_token(&mut self, byte: u8) -> bool {
        byte < commands::CMD_BASE
    }

    fn is_valid_repeat_state(&mut self) -> bool {
        (self.state == State::SPIEnable 
        || self.state == State::I2CEnable
        || self.state == State::UARTEnable)
    }

    fn handle_byte(&mut self, byte: u8) {
        debug!("Received byte {}", byte);

        // If this is a repeat command
        if byte != 0 && self.is_repeat_token(byte) && self.is_valid_repeat_state() {
            // Set the number of times to repeat
            self.repeat_remaining = byte;
            // Set the state to be expecting the command to repeat
            self.state = State::ExpectRepeatCommand;
            return
        }
        // Repeat number has been set and we need to set the command that we will be repeating
        else if self.repeat_remaining != 0 && self.state == State::ExpectRepeatCommand {
            match byte {
                // If it's a nop, do it now...
                commands::CMD_NOP => { 
                    while self.repeat_remaining > 0 {
                        nop();
                        self.repeat_remaining-=1;
                    };
                    self.state = State::Idle;
                },
                // If it's a sleep command, do it now...
                commands::CMD_SLEEP => { 
                    while self.repeat_remaining > 0 {
                        sleep();
                        self.repeat_remaining-=1;
                    };
                    self.state = State::Idle;
                },
                commands::CMD_SPITRANSFER => {
                    self.state = State::SPITransfer;
                },
                commands::CMD_I2CWRITE => {
                    self.state = State::I2CWrite;
                },
                commands::CMD_I2CREAD => {
                    self.state = State::I2CRead;
                },
                commands::CMD_UARTTRANSFER => {
                    self.state = State::UARTTransfer;
                },
                _ => nop(),
            } 

            return;
        }

        // This is a standard, one-time only command
        else if self.repeat_remaining == 0 {
            self.repeat_remaining = 1;
        }

        // Decrement the count
        self.repeat_remaining-=1;

        // Find the appropriate state to transfer to 
        match (self.state, byte) {
            (State::Idle, commands::CMD_NOP) => nop(),
            (State::Idle, commands::CMD_SLEEP) => sleep(),
            (State::Idle, commands::CMD_SPIENABLE) => { 
                self.spi.enable();
                self.state = State::SPIEnable;
            },
            (State::SPIEnable, commands::CMD_SPITRANSFER) => {
                self.state = State::SPITransfer;
            },
            (State::SPITransfer, _) => {
                self.spi.transfer(byte);

                if self.repeat_remaining == 0 {
                    self.state = State::SPIEnable;
                }
            },
            (State::SPIEnable, commands::CMD_SPIDISABLE) => {
                self.spi.disable();
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_SPISETCLOCKDIVISOR) => {
                self.state = State::SPISetClockDivisor;
            },
            (State::SPISetClockDivisor, _) => {
                self.spi.setClockSpeedDivisor(byte);
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_SPISETMODE) => {
                self.state = State::SPISetMode;
            },
            (State::SPISetMode, _) => {
                self.spi.setMode(byte);
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_SPISETFRAME) => {
                self.state = State::SPISetFrame;
            },
            (State::SPISetFrame, _) => {
                self.spi.setFrame(byte);
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_SPISETROLE) => {
                self.state = State::SPISetRole;
            },
            (State::SPISetRole, _) => {
                self.spi.setRole(byte);
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_I2CENABLE) => {
                self.i2c.enable();
                self.state = State::I2CEnable;
            },
            (State::I2CEnable, commands::CMD_I2CWRITE) => {
                self.state = State::I2CWrite;
            },
            (State::I2CEnable, commands::CMD_I2CREAD) => {
                let ret = self.i2c.read();
                if self.repeat_remaining == 0 {
                    self.state = State::I2CEnable;
                }
            },
            (State::I2CWrite, _) => {
                self.i2c.write(byte);
                if self.repeat_remaining == 0 {
                    self.state = State::I2CEnable;
                }
            },
            (State::I2CRead, _) => {
                let ret = self.i2c.read();
                if self.repeat_remaining == 0 {
                    self.state = State::I2CEnable;
                }
            },
            (State::I2CEnable, commands::CMD_I2CDISABLE) => {
                self.i2c.disable();
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_I2CSETSLAVEADDRESS) => {
                self.state = State::I2CSetSlaveAddress;
            },
            (State::I2CSetSlaveAddress, _) => {
                self.i2c.setSlaveAddress(byte);
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_I2CSETMODE) => {
                self.state = State::I2CSetMode;
            },
            (State::I2CSetMode, _) => {
                self.i2c.setMode(byte);
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_UARTENABLE) => {
                self.uart.enable();
                self.state = State::UARTEnable;
            },
            (State::UARTEnable, commands::CMD_UARTTRANSFER) => {
                self.state = State::UARTTransfer;
            },
            (State::UARTTransfer, _) => {
                self.uart.transfer(byte);
                if self.repeat_remaining == 0 {
                    self.state = State::UARTEnable;
                }
            },
            (State::UARTEnable, commands::CMD_UARTDISABLE) => {
                self.uart.disable();
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_UARTSETBAUDRATE) => {
                self.state = State::UARTSetBaudrate;
            },
            (State::UARTSetBaudrate, _) => {
                self.uart.setBaudrate(byte);
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_UARTSETSTOPBITS) => {
                self.state = State::UARTSetStopBits;
            },
            (State::UARTSetStopBits, _) => {
                self.uart.setStopBits(byte);
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_UARTSETPARITY) => {
                self.state = State::UARTSetParity;
            },
            (State::UARTSetParity, _) => {
                self.uart.setParity(byte);
                self.state = State::Idle;
            },
            (State::Idle, commands::CMD_UARTSETDATABITS) => {
                self.state = State::UARTSetDataBits;
            },
            (State::UARTSetDataBits, _) => {
                self.uart.setDataBits(byte);
                self.state = State::Idle;
            },
            _ => nop(),
        }

    }

    // fn return_byte(&self, ) {

    // }
}

fn nop() {
}

fn sleep() {
    nop();
}


//#[cfg(test)]
mod test {
    use super::State;
    use super::IOStateMachine;
    use super::commands;
    use super::SPI;
    use super::I2C;
    use super::UART;

    #[deriving(Copy, Eq, PartialEq, Clone, Show)]
    struct MockSPI {
        enable: bool,
        clock_speed_divisor: u8,
        out_reg: u8,
        mode: u8,
        frame: u8,
        role: u8,
    }

    impl SPI for MockSPI {
        fn transfer(&mut self, byte: u8) -> u8 {
            if self.enable {
                self.out_reg = byte;
            }
            byte - 1
        }
        fn enable(&mut self) {
            self.enable = true;
        }
        fn disable(&mut self) {
            self.enable = false;
        }
        fn setClockSpeedDivisor(&mut self, divisor: u8) {
            self.clock_speed_divisor = divisor;
        }
        fn setMode(&mut self, mode: u8) {
            self.mode = mode;
        }
        fn setRole(&mut self, role: u8) {
            self.role = role;
        }
        fn setFrame(&mut self, frame: u8) {
            self.frame = frame;
        }
    }

    #[deriving(Copy, Eq, PartialEq, Clone, Show)]
    struct MockI2C {
        enable : bool,
        slave_address: u8,
        mode : u8,
        out_reg : u8,
    }

    impl I2C for MockI2C {
        fn enable(&mut self) {
            self.enable = true;
        }
        fn write(&mut self, byte: u8) {
            self.out_reg = byte;
        }
        fn read(&mut self) -> u8 {
            42
        }
        fn disable(&mut self) {
            self.enable = false;
        }
        fn setSlaveAddress(&mut self, address: u8) {
            self.slave_address = address;
        }
        fn setMode(&mut self, mode: u8) {
            self.mode = mode;
        }
    }

    #[deriving(Copy, Eq, PartialEq, Clone, Show)]
    struct MockUART {
        enable : bool,
        baudrate: u8,
        parity : u8,
        stop_bits: u8,
        data_bits : u8,
        out_reg : u8,
    }

    impl UART for MockUART {
        fn enable(&mut self) {
            self.enable = true;
        }
        fn transfer(&mut self, byte: u8) {
            self.out_reg = byte;
        }
        fn disable(&mut self) {
            self.enable = false;
        }
        fn setBaudrate(&mut self, baudrate: u8) {
            self.baudrate = baudrate;
        }
        fn setDataBits(&mut self, data_bits: u8) {
            self.data_bits = data_bits;
        }
        fn setParity(&mut self, parity: u8) {
            self.parity = parity;
        }
        fn setStopBits(&mut self, stop_bits: u8) {
            self.stop_bits = stop_bits;
        }
    }

    struct MockMCU {
        spi: MockSPI,
        i2c: MockI2C,
        uart: MockUART,
    }


    impl MockMCU{
        fn new() -> MockMCU {
            MockMCU{
                spi : MockSPI{enable:false, clock_speed_divisor: 0, out_reg: 0, mode: 0, frame: 0, role: 0} ,
                i2c : MockI2C{enable: false, out_reg: 0, slave_address: 0, mode: 0}, 
                uart : MockUART{enable: false, out_reg: 0, baudrate: 0, parity: 0, data_bits: 0, stop_bits: 0}
            }
        }
    }


    #[test]
    fn test_handle_idle_spi_enable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
    }

    #[test]
    fn test_repeat_token() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        assert_eq!(s.is_repeat_token(254), false);
        assert_eq!(s.is_repeat_token(0), true);
    }

    #[test]
    fn test_repeat_nop() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(100);
        s.handle_byte(commands::CMD_NOP);
        assert_eq!(s.state, State::Idle);
    }

    #[test]
    fn test_repeat_sleep() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(100);
        s.handle_byte(commands::CMD_SLEEP);
        assert_eq!(s.state, State::Idle);
    }

    #[test]
    fn test_handle_spi_enable_enable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_SPIENABLE);
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
    }

    #[test]
    fn test_handle_spi_transfer() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let out: u8 = 200;
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.state, State::SPITransfer);
        s.handle_byte(out);
        assert_eq!(s.spi.out_reg, out);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
    }

    #[test]
    fn test_handle_spi_transfer_repeat() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let rep: u8 = 2;
        let out: u8 = 200;
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
        s.handle_byte(rep);
        assert_eq!(s.state, State::ExpectRepeatCommand);
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.state, State::SPITransfer);
        s.handle_byte(out);
        assert_eq!(s.spi.out_reg, out);
        assert_eq!(s.state, State::SPITransfer);
        s.handle_byte(out);
        assert_eq!(s.spi.out_reg, out);
        assert_eq!(s.state, State::SPIEnable);
        s.handle_byte(out);
        assert_eq!(s.spi.out_reg, out);
        assert_eq!(s.state, State::SPIEnable);
    }

    #[test]
    fn test_handle_spi_disable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
        s.handle_byte(commands::CMD_SPIDISABLE);
        assert_eq!(s.state, State::Idle);
        assert_eq!(s.spi.enable, false);    
    }

    #[test]
    fn test_handle_spi_transfer_disable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.state, State::SPITransfer);
        s.handle_byte(commands::CMD_SPIDISABLE);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
    }

    #[test]
    fn test_handle_spi_transfer_repeat_disable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let rep: u8 = 2;
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
        s.handle_byte(rep);
        assert_eq!(s.state, State::ExpectRepeatCommand);
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.state, State::SPITransfer);
        s.handle_byte(commands::CMD_SPIDISABLE);
        assert_eq!(s.spi.out_reg, commands::CMD_SPIDISABLE);
        assert_eq!(s.state, State::SPITransfer);
    }

    #[test]
    fn test_spi_config() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let div: u8 = 2;
        let mode: u8 = 3;
        let frame: u8 = 4;
        let role: u8 = 5;
        s.handle_byte(commands::CMD_SPISETCLOCKDIVISOR);
        s.handle_byte(div);
        assert_eq!(s.spi.clock_speed_divisor, div);
        assert_eq!(s.state, State::Idle);
        s.handle_byte(commands::CMD_SPISETMODE);
        s.handle_byte(mode);
        assert_eq!(s.spi.mode, mode);
        assert_eq!(s.state, State::Idle);
        s.handle_byte(commands::CMD_SPISETFRAME);
        s.handle_byte(frame);
        assert_eq!(s.spi.frame, frame);
        assert_eq!(s.state, State::Idle);
        s.handle_byte(commands::CMD_SPISETROLE);
        s.handle_byte(role);
        assert_eq!(s.spi.role, role);
        assert_eq!(s.state, State::Idle);
    }

    #[test]
    fn test_handle_i2c_enable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
    }

    #[test]
    fn test_handle_i2c_write() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let out: u8 = 100;
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
        s.handle_byte(commands::CMD_I2CWRITE);
        assert_eq!(s.state, State::I2CWrite);
        s.handle_byte(out);
         assert_eq!(s.i2c.out_reg, out);
        assert_eq!(s.state, State::I2CEnable);
    }

    #[test]
    fn test_handle_i2c_write_repeat() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let out: u8 = 200;
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
        let repeat: u8 = 5;
        s.handle_byte(repeat);
        assert_eq!(s.state, State::ExpectRepeatCommand);
        s.handle_byte(commands::CMD_I2CWRITE);
        assert_eq!(s.state, State::I2CWrite);
        for i in range(0, repeat-1) {
            s.handle_byte(out);
            assert_eq!(s.i2c.out_reg, out);
            assert_eq!(s.state, State::I2CWrite);
        }
        s.handle_byte(out);
        assert_eq!(s.i2c.out_reg, out);
        assert_eq!(s.state, State::I2CEnable);
    }

    #[test]
    fn test_handle_i2c_read() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
        s.handle_byte(commands::CMD_I2CREAD);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
    }

    #[test]
    fn test_handle_i2c_read_repeat() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
        let repeat: u8 = 5;
        s.handle_byte(repeat);
        assert_eq!(s.state, State::ExpectRepeatCommand);
        s.handle_byte(commands::CMD_I2CREAD);
        assert_eq!(s.state, State::I2CRead);
        for i in range(0, repeat-1) {
            s.handle_byte(0);
            assert_eq!(s.state, State::I2CRead);
        }
        s.handle_byte(200);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
    }

    #[test]
    fn test_handle_i2c_disable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
        s.handle_byte(commands::CMD_I2CDISABLE);
        assert_eq!(s.state, State::Idle);
        assert_eq!(s.i2c.enable, false);
    }

     #[test]
    fn test_handle_i2c_write_disable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
        s.handle_byte(commands::CMD_I2CWRITE);
        assert_eq!(s.state, State::I2CWrite);
        s.handle_byte(commands::CMD_I2CDISABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
    }

    #[test]
    fn test_handle_i2c_write_repeat_disable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let repeat: u8 = 2;
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
        s.handle_byte(repeat);
        assert_eq!(s.state, State::ExpectRepeatCommand);
        s.handle_byte(commands::CMD_I2CWRITE);
        assert_eq!(s.state, State::I2CWrite);
        s.handle_byte(commands::CMD_SPIDISABLE);
        assert_eq!(s.i2c.out_reg, commands::CMD_SPIDISABLE);
        assert_eq!(s.state, State::I2CWrite);
    }

    #[test]
    fn test_i2c_config() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let slave_address: u8 = 17;
        let mode: u8 = 3;
        s.handle_byte(commands::CMD_I2CSETSLAVEADDRESS);
        s.handle_byte(slave_address);
        assert_eq!(s.i2c.slave_address, slave_address);
        assert_eq!(s.state, State::Idle);
        s.handle_byte(commands::CMD_I2CSETMODE);
        s.handle_byte(mode);
        assert_eq!(s.i2c.mode, mode);
        assert_eq!(s.state, State::Idle);
    }

    #[test]
    fn test_handle_uart_enable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_UARTENABLE);
        assert_eq!(s.state, State::UARTEnable);
        assert_eq!(s.uart.enable, true);
    }

    #[test]
    fn test_handle_uart_transfer() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let out: u8 = 200;
        s.handle_byte(commands::CMD_UARTENABLE);
        assert_eq!(s.state, State::UARTEnable);
        assert_eq!(s.uart.enable, true);
        s.handle_byte(commands::CMD_UARTTRANSFER);
        assert_eq!(s.state, State::UARTTransfer);
        s.handle_byte(out);
        assert_eq!(s.uart.out_reg, out);
        assert_eq!(s.state, State::UARTEnable);
        assert_eq!(s.uart.enable, true);
    }

    #[test]
    fn test_handle_uart_disable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_UARTENABLE);
        assert_eq!(s.state, State::UARTEnable);
        assert_eq!(s.uart.enable, true);
        s.handle_byte(commands::CMD_UARTDISABLE);
        assert_eq!(s.state, State::Idle);
        assert_eq!(s.uart.enable, false);
    }

     #[test]
    fn test_handle_uart_write_disable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_UARTENABLE);
        assert_eq!(s.state, State::UARTEnable);
        assert_eq!(s.uart.enable, true);
        s.handle_byte(commands::CMD_UARTTRANSFER);
        assert_eq!(s.state, State::UARTTransfer);
        s.handle_byte(commands::CMD_UARTDISABLE);
        assert_eq!(s.state, State::UARTEnable);
        assert_eq!(s.uart.enable, true);
    }

    #[test]
    fn test_handle_uart_write_repeat_disable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let repeat: u8 = 2;
        s.handle_byte(commands::CMD_UARTENABLE);
        assert_eq!(s.state, State::UARTEnable);
        assert_eq!(s.uart.enable, true);
        s.handle_byte(repeat);
        assert_eq!(s.state, State::ExpectRepeatCommand);
        s.handle_byte(commands::CMD_UARTTRANSFER);
        assert_eq!(s.state, State::UARTTransfer);
        s.handle_byte(commands::CMD_UARTDISABLE);
        assert_eq!(s.state, State::UARTTransfer);
    }

      #[test]
    fn test_uart_config() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        let baudrate: u8 = 7;
        let stop_bits: u8 = 6;
        let parity: u8 = 5;
        let data_bits: u8 = 4;
        s.handle_byte(commands::CMD_UARTSETBAUDRATE);
        s.handle_byte(baudrate);
        assert_eq!(s.uart.baudrate, baudrate);
        assert_eq!(s.state, State::Idle);
        s.handle_byte(commands::CMD_UARTSETSTOPBITS);
        s.handle_byte(stop_bits);
        assert_eq!(s.uart.stop_bits, stop_bits);
        assert_eq!(s.state, State::Idle);
        s.handle_byte(commands::CMD_UARTSETPARITY);
        s.handle_byte(parity);
        assert_eq!(s.uart.parity, parity);
        assert_eq!(s.state, State::Idle);
        s.handle_byte(commands::CMD_UARTSETDATABITS);
        s.handle_byte(data_bits);
        assert_eq!(s.uart.data_bits, data_bits);
        assert_eq!(s.state, State::Idle);
    }

    #[test]
    fn test_handle_spi_write_while_uart_enable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_UARTENABLE);
        assert_eq!(s.state, State::UARTEnable);
        assert_eq!(s.uart.enable, true);
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.state, State::UARTEnable);
        assert_eq!(s.uart.enable, true);
    }

    #[test]
    fn test_handle_spi_transfer_while_idle() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.state, State::Idle);
        assert_eq!(s.spi.enable, false);
    }

    #[test]
    fn test_handle_spi_enable_while_i2c_enable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::I2CEnable);
        assert_eq!(s.i2c.enable, true);
    }

    #[test]
    fn test_zero_repeat_in_spi_enable() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
        s.handle_byte(0);
        assert_eq!(s.state, State::SPIEnable);
        assert_eq!(s.spi.enable, true);
    }

    #[test]
    fn test_valid_state() {
        let mut m = MockMCU::new();
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0, spi: &mut m.spi, i2c: &mut m.i2c, uart: &mut m.uart};
        assert_eq!(s.is_valid_repeat_state(), false);
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.is_valid_repeat_state(), true);
        assert_eq!(s.spi.enable, true);
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.is_valid_repeat_state(), false);
    }
}
