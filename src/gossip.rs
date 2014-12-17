trait SPI {
    fn enable(&mut self);
    fn transfer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint;
    fn disable(&mut self);
    fn set_clock_speed_divisor(&mut self, divisor: u8);
    fn set_mode(&mut self, mode: u8);
    fn set_role(&mut self, role: u8);
    fn set_frame(&mut self, frame: u8);
}

trait I2C {
    fn enable(&mut self);
    fn write(&mut self, byte: u8);
    fn read(&mut self) -> u8;
    fn disable(&mut self);
    fn set_slave_address(&mut self, address: u8);
    fn set_mode(&mut self, mode: u8);
}

trait UART {
    fn enable(&mut self);
    fn transfer(&mut self, byte: u8);
    fn disable(&mut self);
    fn set_baudrate(&mut self, baudrate: u8);
    fn set_data_bits(&mut self, data_bits: u8);
    fn set_parity(&mut self, parity: u8);
    fn set_stop_bits(&mut self, stop_bits: u8);
}

trait GPIO {
    fn set_pull(&mut self, pull: u8);
    fn set_direction(&mut self, direction: u8);
    fn write_digital_value(&mut self, value: u8);
    fn write_analog_value(&mut self, value: u8);
    fn write_pwm_value(&mut self, value: u8);
    fn get_pull(&mut self) -> u8;
    fn get_direction(&mut self) -> u8;
    fn read_digital_value(&mut self) -> u8;
    fn read_analog_value(&mut self) -> u8;
    fn read_pulse_length(&mut self) -> u8;
    fn set_interrupt(&mut self, interrupt: u8);
}

mod command {
    // Base Addr
    pub const BASE: u8 =                            0x80;

    // General Ops
    pub const NOP: u8 =                             0x00;
    pub const SLEEP: u8 =                           0x10 | BASE;

    // SPI          
    pub const SPIENABLE: u8 =                       0x20 | BASE;
    pub const SPITRANSFER: u8 =                     0x21 | BASE;
    pub const SPIDISABLE: u8 =                      0x22 | BASE;
    pub const SPISETCLOCKDIVISOR: u8 =              0x23 | BASE;
    pub const SPISETMODE: u8 =                      0x24 | BASE;
    pub const SPISETROLE: u8 =                      0x25 | BASE;
    pub const SPISETFRAME: u8 =                     0x26 | BASE;

    // I2C          
    pub const I2CENABLE: u8 =                       0x30 | BASE;
    pub const I2CWRITE: u8 =                        0x31 | BASE;
    pub const I2CREAD: u8 =                         0x32 | BASE;
    pub const I2CDISABLE: u8 =                      0x33 | BASE;
    pub const I2CSETMODE: u8 =                      0x34 | BASE;
    pub const I2CSETSLAVEADDRESS: u8 =              0x35 | BASE;

    // UART
    pub const UARTENABLE: u8 =                      0x40 | BASE;
    pub const UARTTRANSFER: u8 =                    0x41 | BASE;
    pub const UARTRECEIVE: u8 =                     0x42 | BASE;
    pub const UARTDISABLE: u8 =                     0x43 | BASE;
    pub const UARTSETBAUDRATE: u8 =                 0x44 | BASE;
    pub const UARTSETDATABITS: u8 =                 0x45 | BASE;
    pub const UARTSETPARITY: u8 =                   0x46 | BASE;
    pub const UARTSETSTOPBITS: u8 =                 0x47 | BASE;

    // GPIO
    pub const GPIO_SET_PULL: u8 =                   0x50 | BASE;
    pub const GPIO_SET_STATE: u8 =                  0x51 | BASE;
    pub const GPIO_WRITE_PWM_VALUE: u8 =            0x52 | BASE;
    pub const GPIO_GET_PULL: u8 =                   0x53 | BASE;
    pub const GPIO_GET_STATE: u8 =                  0x54 | BASE;
    pub const GPIO_READ_PULSE_LENGTH: u8 =          0x55 | BASE;
    pub const GPIO_SET_INTERRUPT: u8 =              0x56 | BASE;
}

#[deriving(Copy, Eq, PartialEq, Clone, Show)]
pub enum CommState {
    Enable,
    Idle,
}

pub struct SPIStateMachine<'a, S: 'a> {
    pub spi: &'a mut S,
    pub state: CommState,
    pub transfer_length: u8,
}

impl<'a, S> SPIStateMachine<'a, S> where S: SPI {
    fn handle_buffer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint {

        let command = incoming[0];
        println!("Command: {}", command);
        match (self.state, command) {
            (CommState::Idle, command::SPIENABLE) => {
                self.spi.enable();
                self.state = CommState::Enable;
                outgoing[0] = command::SPIENABLE;
                1 as uint
            },
            (CommState::Idle, command::SPIDISABLE) => {
                outgoing[0] = command::SPIDISABLE;
                1 as uint
            },
            (CommState::Enable, command::SPIENABLE) => {
                outgoing[0] = command::SPIENABLE;
                1 as uint
            },
            (CommState::Enable, command::SPITRANSFER) => {
                let length = incoming[1];
                let payload = incoming.slice_from(2);
                outgoing[0] = command::SPITRANSFER;
                outgoing[1] = length;
                self.spi.transfer(payload, outgoing.slice_from_mut(2)) + 2u
            },
            (CommState::Enable, command::SPIDISABLE) => {
                self.spi.disable();
                self.state = CommState::Idle;
                outgoing[0] = command::SPIDISABLE;
                1 as uint
            },
            (_, command::SPISETCLOCKDIVISOR) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.spi.set_clock_speed_divisor(param);
                outgoing[0] = command::SPISETCLOCKDIVISOR;
                outgoing[1] = param;
                2 as uint
            },
            (_, command::SPISETMODE) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.spi.set_mode(param);
                outgoing[0] = command::SPISETMODE;
                outgoing[1] = param;
                2 as uint
            },
            (_, command::SPISETROLE) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.spi.set_role(param);
                outgoing[0] = command::SPISETROLE;
                outgoing[1] = param;
                2 as uint
            },
            (_, command::SPISETFRAME) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.spi.set_frame(param);
                outgoing[0] = command::SPISETFRAME;
                outgoing[1] = param;
                2 as uint
            },
            _ => 0 as uint
        }
    }
}

pub struct CommandRouter<'a, S: 'a> {
    pub spi : &'a mut SPIStateMachine<'a, S>
}

impl<'a, S> CommandRouter<'a, S> where S: SPI {
    pub fn handle_buffer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint {
        let command = incoming[0];
        if command >= command::SPIENABLE || command <= command::SPISETFRAME {
            self.spi.handle_buffer(incoming, outgoing)
        }
        else {
            0
        }
    }
}

//#[cfg(test)]
pub mod test {
    use super::CommState;
    use super::CommandRouter;
    use super::command;
    use super::SPI;
    // use super::I2C;
    // use super::UART;
    // use super::GPIO;

    #[deriving(Copy, Eq, PartialEq, Clone, Show)]
    pub struct MockSPI {
        pub enable: bool,
        pub clock_speed_divisor: u8,
        pub out_reg: u8,
        pub mode: u8,
        pub frame: u8,
        pub role: u8,
    }

    impl SPI for MockSPI {
        fn transfer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint {
            if self.enable {
                for x in range(0u, incoming.len()) {
                    outgoing[x] = incoming[x];
                }
            }
            incoming.len()
        }
        fn enable(&mut self) {
            println!("Enabled!!!");
            self.enable = true;
        }
        fn disable(&mut self) {
            self.enable = false;
        }
        fn set_clock_speed_divisor(&mut self, divisor: u8) {
            self.clock_speed_divisor = divisor;
        }
        fn set_mode(&mut self, mode: u8) {
            self.mode = mode;
        }
        fn set_role(&mut self, role: u8) {
            self.role = role;
        }
        fn set_frame(&mut self, frame: u8) {
            self.frame = frame;
        }
    }
}