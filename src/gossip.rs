#![feature(phase)]
#[phase(plugin, link)]
extern crate log;

#[deriving(Copy, Eq, PartialEq, Clone, Show)]
enum State {
    Idle,
    SPIEnable, 
    SPITransfer,
    I2CEnable,
    I2CTransfer,
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
    pub const CMD_I2CENABLE: u8 =       0xa0;
    pub const CMD_I2CTRANSFER: u8 =     0xa1;
    pub const CMD_UARTENABLE: u8 =      0xb0;
    pub const CMD_UARTTRANSFER: u8 =    0xb1;
    pub const CMD_UARTRECEIVE: u8 =     0xb2;
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

                    self.repeat_remaining-= 1;

                    if (self.repeat_remaining == 0) {
                        self.state = State::SPIEnable;
                    }
                }

                _ => debug!("Fuck yeah Rust is exhausting")
            } 
        }
        else if self.repeat_remaining > 0 {
            debug!("Repeating this shit!");
            match (self.state, byte) {
                 (State::SPIEnable, commands::CMD_SPITRANSFER) => {
                    // spi_transfer(byte);
                    self.state = State::SPIEnable;
                },
                _ => nop(),
            }

            self.repeat_remaining-=1;
        }

        // If this is a repeat command
        else if self.is_repeat_token(byte) {
            // Set the number of times to repeat
            self.repeat_remaining = byte;
            // Set the state to be expecting the command to repeat
            self.state = State::ExpectRepeatCommand;
            return
        }
        // This is not a repeated command
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
    fn test_handle_i2c_enable() {
        let mut s = IOStateMachine{state:State::Idle, repeat_remaining : 0};
        
    }
}
