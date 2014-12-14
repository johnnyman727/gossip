#![feature(phase)]
#[phase(plugin, link)]
extern crate log;

#[deriving(Copy, Eq, PartialEq, Clone, Show)]
enum State {
    Idle,
    SPIEnable, 
    SPITransfer,
    I2CEnable,
    I2CWrite,
    I2CRead,
    UARTEnable,
    UARTTransfer,
    UARTReceive,
    ExpectRepeatCommand,
}

mod commands {
    pub const CMD_NOP: u8 =             0x00;
    pub const CMD_SLEEP: u8 =           0x80;
    pub const CMD_SPIENABLE: u8 =       0x90;
    pub const CMD_SPITRANSFER: u8 =     0x91;
    pub const CMD_SPIDISABLE: u8 =      0x92;
    pub const CMD_I2CENABLE: u8 =       0xa0;
    pub const CMD_I2CWRITE: u8 =        0xa1;
    pub const CMD_I2CREAD: u8 =         0xa2;
    pub const CMD_I2CDISABLE: u8 =      0xa3;
    pub const CMD_UARTENABLE: u8 =      0xb0;
    pub const CMD_UARTTRANSFER: u8 =    0xb1;
    pub const CMD_UARTRECEIVE: u8 =     0xb2;
    pub const CMD_UARTDISABLE: u8 =     0xb3;
}


struct IOStateMachine {
    state : State,
    repeat_remaining : u8,
}

impl IOStateMachine {

    fn is_repeat_token(&mut self, byte: u8) -> bool {
        byte < 0b10000000
    }

    fn handle_byte(&mut self, byte: u8) {
        debug!("Received byte {}", byte);
        // If we are awaiting the command to repeat
        if self.repeat_remaining != 0 && self.state == State::ExpectRepeatCommand {
            debug!("Nah, we're going to set the command to repeat {}", byte);
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
                }
                commands::CMD_I2CWRITE => {
                    self.state = State::I2CWrite;
                }
                commands::CMD_I2CREAD => {
                    self.state = State::I2CRead;
                }

                _ => debug!("Fuck yeah Rust is exhausting")
            } 
        }
        // A repeated command has been set and we are executing it
        else if self.repeat_remaining > 0 {

            self.repeat_remaining-=1;

            debug!("Repeating this shit! {}", self.repeat_remaining);

            match (self.state, byte) {
                 (State::SPITransfer, _) => {

                    // let ret = spi.transfer(byte);

                    if (self.repeat_remaining == 0) {
                        self.state = State::SPIEnable;
                    }
                },
                (State::I2CWrite, _) => {

                    // i2c.write(byte);


                    if (self.repeat_remaining == 0) {
                        debug!("Setting back to enable!");
                        self.state = State::I2CEnable;
                    }
                },
                (State::I2CRead, _) => {

                    // let ret = i2c.read(byte);


                    if (self.repeat_remaining == 0) {
                        debug!("Setting back to enable!");
                        self.state = State::I2CEnable;
                    }
                },
                _ => nop(),
            }
        }

        // If this is a repeat command
        else if self.is_repeat_token(byte) && (self.state == State::SPIEnable 
                                                || self.state == State::I2CEnable
                                                || self.state == State::UARTEnable) {
            // Set the number of times to repeat
            self.repeat_remaining = byte;
            // Set the state to be expecting the command to repeat
            self.state = State::ExpectRepeatCommand;
            debug!("going to repeat {} times", byte);
            return
        }
        // This is not a repeated command (common case)
        else {

            match (self.state, byte) {
                (State::Idle, commands::CMD_NOP) => nop(),
                (State::Idle, commands::CMD_SLEEP) => sleep(),
                (State::Idle, commands::CMD_SPIENABLE) => { 
                    // spi.enable();
                    self.state = State::SPIEnable;
                },
                (State::SPIEnable, commands::CMD_SPITRANSFER) => {
                    self.state = State::SPITransfer;
                },
                (State::SPITransfer, _) => {
                    // spi.transfer(byte);
                    self.state = State::SPIEnable;  
                },
                (State::SPIEnable, commands::CMD_SPIDISABLE) => {
                    // spi.disable();
                    self.state = State::Idle;
                },
                (State::Idle, commands::CMD_I2CENABLE) => {
                    // i2c.enable();
                    self.state = State::I2CEnable;
                },
                (State::I2CEnable, commands::CMD_I2CWRITE) => {
                    self.state = State::I2CWrite;
                }
                (State::I2CEnable, commands::CMD_I2CREAD) => {
                    // let ret = i2c.read();

                }
                (State::I2CWrite, _) => {
                    // i2c.write(byte);
                    self.state = State::I2CEnable;
                }
                (State::I2CEnable, commands::CMD_I2CDISABLE) => {
                    // i2c.disable();
                    self.state = State::Idle;
                }
                _ => nop(),
            }
        }

    }

    // fn return_byte(&self, ) {

    // }
}

fn nop() {
    // debug!("Fuck CMD_NOPPPPIINNNGGG!"); 
}

fn sleep() {
    // debug!("SHUT THE F UP I'M CMD_SLEEPING");
}



trait SPI {
    fn enable() -> bool;
    fn transfer(i: u8) -> u8;
    fn disable() -> bool;
}

//#[cfg(test)]
mod test {
    use super::State;
    use super::IOStateMachine;
    use super::commands;


    #[test]
    fn test_handle_idle_spi_enable() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
    }

    #[test]
    fn test_repeat_token() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        assert_eq!(s.is_repeat_token(254), false);
        assert_eq!(s.is_repeat_token(0), true);
    }

    #[test]
    fn test_repeat_nop() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(100);
        s.handle_byte(commands::CMD_NOP);
        assert_eq!(s.state, State::Idle);
    }

    #[test]
    fn test_repeat_sleep() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(100);
        s.handle_byte(commands::CMD_SLEEP);
        assert_eq!(s.state, State::Idle);
    }

    #[test]
    fn test_handle_spi_enable_enable() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_SPIENABLE);
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
    }

    #[test]
    fn test_handle_spi_transfer() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.state, State::SPITransfer);
        s.handle_byte(200);
        assert_eq!(s.state, State::SPIEnable);
    }

    #[test]
    fn test_handle_spi_transfer_repeat() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        s.handle_byte(2);
        assert_eq!(s.state, State::ExpectRepeatCommand);
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.state, State::SPITransfer);
        s.handle_byte(200);
        assert_eq!(s.state, State::SPITransfer);
        s.handle_byte(200);
        assert_eq!(s.state, State::SPIEnable);
        s.handle_byte(200);
        assert_eq!(s.state, State::SPIEnable);
    }

    #[test]
    fn test_handle_spi_disable() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        s.handle_byte(commands::CMD_SPIDISABLE);
        assert_eq!(s.state, State::Idle);

    }

    #[test]
    fn test_handle_spi_transfer_disable() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.state, State::SPITransfer);
        s.handle_byte(commands::CMD_SPIDISABLE);
        assert_eq!(s.state, State::SPIEnable);

    }

    #[test]
    fn test_handle_spi_transfer_repeat_disable() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_SPIENABLE);
        assert_eq!(s.state, State::SPIEnable);
        s.handle_byte(2);
        assert_eq!(s.state, State::ExpectRepeatCommand);
        s.handle_byte(commands::CMD_SPITRANSFER);
        assert_eq!(s.state, State::SPITransfer);
        s.handle_byte(commands::CMD_SPIDISABLE);
        assert_eq!(s.state, State::SPITransfer);
    }

    #[test]
    fn test_handle_i2c_enable() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
    }

    #[test]
    fn test_handle_i2c_write() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        s.handle_byte(commands::CMD_I2CWRITE);
        assert_eq!(s.state, State::I2CWrite);
        s.handle_byte(100);
        assert_eq!(s.state, State::I2CEnable);
    }

    #[test]
    fn test_handle_i2c_write_repeat() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        let repeat: u8 = 5;
        s.handle_byte(repeat);
        assert_eq!(s.state, State::ExpectRepeatCommand);
        s.handle_byte(commands::CMD_I2CWRITE);
        assert_eq!(s.state, State::I2CWrite);
        for i in range(0, repeat-1) {
            debug!("Sending again {}", i);
            s.handle_byte(i);
            assert_eq!(s.state, State::I2CWrite);
        }
        s.handle_byte(200);
        assert_eq!(s.state, State::I2CEnable);
    }

    #[test]
    fn test_handle_i2c_read() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        s.handle_byte(commands::CMD_I2CREAD);
        assert_eq!(s.state, State::I2CEnable);
    }

    #[test]
    fn test_handle_i2c_read_repeat() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
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
    }

    #[test]
    fn test_handle_i2c_disable() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        s.handle_byte(commands::CMD_I2CDISABLE);
        assert_eq!(s.state, State::Idle);
    }

     #[test]
    fn test_handle_i2c_write_disable() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        s.handle_byte(commands::CMD_I2CWRITE);
        assert_eq!(s.state, State::I2CWrite);
        s.handle_byte(commands::CMD_I2CDISABLE);
        assert_eq!(s.state, State::I2CEnable);

    }

    #[test]
    fn test_handle_i2c_write_repeat_disable() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        s.handle_byte(commands::CMD_I2CENABLE);
        assert_eq!(s.state, State::I2CEnable);
        s.handle_byte(2);
        assert_eq!(s.state, State::ExpectRepeatCommand);
        s.handle_byte(commands::CMD_I2CWRITE);
        assert_eq!(s.state, State::I2CWrite);
        s.handle_byte(commands::CMD_SPIDISABLE);
        assert_eq!(s.state, State::I2CWrite);
    }
}
