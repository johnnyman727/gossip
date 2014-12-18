pub trait SPI {
    fn enable(&mut self);
    fn transfer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint;
    fn disable(&mut self);
    fn set_clock_speed_divisor(&mut self, divisor: u8);
    fn set_mode(&mut self, mode: u8);
    fn set_role(&mut self, role: u8);
    fn set_frame(&mut self, frame: u8);
}

pub trait I2C {
    fn enable(&mut self);
    fn write(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint;
    fn read(&mut self, length: u8, outgoing: &mut [u8]) -> uint;
    fn disable(&mut self);
    fn set_slave_address(&mut self, address: u8);
    fn set_mode(&mut self, mode: u8);
}

pub trait UART {
    fn enable(&mut self);
    fn transfer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint;
    fn disable(&mut self);
    fn set_baudrate(&mut self, baudrate: u32);
    fn set_data_bits(&mut self, data_bits: u8);
    fn set_parity(&mut self, parity: u8);
    fn set_stop_bits(&mut self, stop_bits: u8);
}

pub trait GPIO {
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
    fn set_interrupt_mode(&mut self, interrupt: u8);
    fn get_interrupt_mode(&mut self) -> u8;
}

pub mod command {
    // Base Addr
    pub const BASE: u8 =                            0x80;

    // General Ops
    pub const NOP: u8 =                             0x00;
    pub const SLEEP: u8 =                           0x10;

    // SPI
    pub const SPICMDBASE: u8 =                      0x20;
    pub const SPIENABLE: u8 =                       0x20;
    pub const SPITRANSFER: u8 =                     0x21;
    pub const SPIDISABLE: u8 =                      0x22;
    pub const SPISETCLOCKDIVISOR: u8 =              0x23;
    pub const SPISETMODE: u8 =                      0x24;
    pub const SPISETROLE: u8 =                      0x25;
    pub const SPISETFRAME: u8 =                     0x26;

    // I2C
    pub const I2CCMDBASE: u8 =                      0x30;
    pub const I2CENABLE: u8 =                       0x30;
    pub const I2CWRITE: u8 =                        0x31;
    pub const I2CREAD: u8 =                         0x32;
    pub const I2CDISABLE: u8 =                      0x33;
    pub const I2CSETMODE: u8 =                      0x34;
    pub const I2CSETSLAVEADDRESS: u8 =              0x35;

    // UART
    pub const UARTCMDBASE: u8 =                     0x40;
    pub const UARTENABLE: u8 =                      0x40;
    pub const UARTTRANSFER: u8 =                    0x41;
    pub const UARTRECEIVE: u8 =                     0x42;
    pub const UARTDISABLE: u8 =                     0x43;
    pub const UARTSETBAUDRATE: u8 =                 0x44;
    pub const UARTSETDATABITS: u8 =                 0x45;
    pub const UARTSETPARITY: u8 =                   0x46;
    pub const UARTSETSTOPBITS: u8 =                 0x47;

    // GPIO
    pub const GPIOCMDBASE: u8 =                     0x50;
    pub const GPIOSETPULL: u8 =                     0x50;
    pub const GPIOSETSTATE: u8 =                    0x51;
    pub const GPIOWRITEPWMVALUE: u8 =               0x52;
    pub const GPIOGETPULL: u8 =                     0x53;
    pub const GPIOGETSTATE: u8 =                    0x54;
    pub const GPIOREADPULSELENGTH: u8 =             0x55;
    pub const GPIOSETINTERRUPTMODE: u8 =            0x56;
    pub const GPIOGETINTERRUPTMODE: u8 =            0x57;
}

const NO_CHANGE: u8 = 0xFF;

#[deriving(Copy, Eq, PartialEq, Clone, Show)]
pub enum CommState {
    Enable,
    Idle,
}

pub struct SPIStateMachine<'a, S: 'a> {
    pub spi: &'a mut S,
    pub state: CommState,
}

impl<'a, S> SPIStateMachine<'a, S> where S: SPI {
    pub fn handle_buffer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint {

        let command = incoming[0];
        println!("SPI Command: {0:x}", command);
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
                1 as uint
            },
            (_, command::SPISETMODE) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.spi.set_mode(param);
                outgoing[0] = command::SPISETMODE;
                1 as uint
            },
            (_, command::SPISETROLE) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.spi.set_role(param);
                outgoing[0] = command::SPISETROLE;
                1 as uint
            },
            (_, command::SPISETFRAME) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.spi.set_frame(param);
                outgoing[0] = command::SPISETFRAME;
                1 as uint
            },
            _ => 0 as uint
        }
    }
}

pub struct I2CStateMachine<'a, I: 'a> {
    pub i2c: &'a mut I,
    pub state: CommState,
}

impl<'a, I> I2CStateMachine<'a, I> where I: I2C {
    pub fn handle_buffer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint {
        let command = incoming[0];
        println!("I2C Command: {}", command);
        match (self.state, command) {
            (CommState::Idle, command::I2CENABLE) => {
                self.i2c.enable();
                self.state = CommState::Enable;
                outgoing[0] = command;
                1 as uint
            },
            (CommState::Idle, command::I2CDISABLE) => {
                outgoing[0] = command;
                1 as uint
            },
            (CommState::Enable, command::I2CENABLE) => {
                outgoing[0] = command;
                1 as uint
            },
            (CommState::Enable, command::I2CWRITE) => {
                let length = incoming[1];
                let payload = incoming.slice_from(2);
                outgoing[0] = command;
                outgoing[1] = length;
                self.i2c.write(payload, outgoing.slice_from_mut(2)) + 2u
            },
            (CommState::Enable, command::I2CREAD) => {
                let length = incoming[1];
                outgoing[0] = command;
                outgoing[1] = length;
                self.i2c.read(length, outgoing.slice_from_mut(2)) + 2u
            },
            (CommState::Enable, command::I2CDISABLE) => {
                self.i2c.disable();
                self.state = CommState::Idle;
                outgoing[0] = command::I2CDISABLE;
                1 as uint
            },
            (_, command::I2CSETMODE) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.i2c.set_mode(param);
                outgoing[0] = command::I2CSETMODE;
                1 as uint
            },
            (_, command::I2CSETSLAVEADDRESS) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.i2c.set_slave_address(param);
                outgoing[0] = command::I2CSETSLAVEADDRESS;
                1 as uint 
            },
            _ => 0 as uint
        }
    }
}

pub struct UARTStateMachine<'a, U: 'a> {
    pub uart: &'a mut U,
    pub state: CommState,
}

impl<'a, U> UARTStateMachine<'a, U> where U: UART {
    pub fn handle_buffer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint {
        let command = incoming[0];
        println!("UART Command: {}", command);
        match (self.state, command) {
            (CommState::Idle, command::UARTENABLE) => {
                self.uart.enable();
                self.state = CommState::Enable;
                outgoing[0] = command;
                1 as uint
            },
            (CommState::Idle, command::UARTDISABLE) => {
                outgoing[0] = command;
                1 as uint
            },
            (CommState::Enable, command::UARTENABLE) => {
                outgoing[0] = command;
                1 as uint
            },
            (CommState::Enable, command::UARTTRANSFER) => {
                let length = incoming[1];
                let payload = incoming.slice_from(2);
                outgoing[0] = command;
                outgoing[1] = length;
                self.uart.transfer(payload, outgoing.slice_from_mut(2)) + 2u
            },
            (CommState::Enable, command::UARTRECEIVE) => {
                // TODO - handle async uart receives
                0 as uint
            },
            (CommState::Enable, command::UARTDISABLE) => {
                self.uart.disable();
                self.state = CommState::Idle;
                outgoing[0] = command;
                1 as uint
            },
            (_, command::UARTSETBAUDRATE) => {
                let payload = incoming.slice_from(1);
                let param: u32 = payload[0] as u32 | (payload[1] as u32 << 8) | (payload[2] as u32 << 16) | (payload[3] as u32 << 24);
                self.uart.set_baudrate(param);
                outgoing[0] = command;
                1 as uint
            },
            (_, command::UARTSETDATABITS) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.uart.set_data_bits(param);
                outgoing[0] = command;
                1 as uint
            },
            (_, command::UARTSETPARITY) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.uart.set_data_bits(param);
                outgoing[0] = command;
                1 as uint
            },
            (_, command::UARTSETSTOPBITS) => {
                let payload = incoming.slice_from(1);
                let param = payload[0];
                self.uart.set_stop_bits(param);
                outgoing[0] = command;
                1 as uint
            },
            _ => 0 as uint
        }
    }
}

pub struct GPIOStateMachine<'a, G: 'a> {
    pub gpios : &'a mut [G],
}

impl<'a, G> GPIOStateMachine<'a, G> where G: GPIO {
    pub fn handle_buffer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint {
        let command = incoming[0];
        let gpioIndex = incoming[1];
        let ref mut gpio = self.gpios[gpioIndex as uint];
         println!("GPIO Command: {0:x}", command);
        match command {
            command::GPIOSETPULL => {
                gpio.set_pull(incoming[2]);
                outgoing[0] = command;
                1 as uint
            },
            command::GPIOSETSTATE => {
                let new_value = incoming[2];
                let new_direction = incoming[3];
                println!("Setting {} {}", new_value, new_direction);
                if new_value != NO_CHANGE {
                    gpio.write_digital_value(incoming[2]);
                }

                if new_direction != NO_CHANGE {
                    gpio.set_direction(incoming[3]);
                }
                
                outgoing[0] = command;
                1 as uint
            },
            command::GPIOWRITEPWMVALUE => {
                gpio.write_pwm_value(incoming[2]);
                outgoing[0] = command;
                1 as uint
            },
            command::GPIOGETPULL => {
                outgoing[0] = command;
                outgoing[1] = gpio.get_pull();
                2 as uint
            },
            command::GPIOGETSTATE => {
                outgoing[0] = command;
                println!("returning {} {}", gpio.read_digital_value(), gpio.get_direction());
                outgoing[1] = gpio.read_digital_value();
                outgoing[2] = gpio.get_direction();
                3 as uint
            },
            command::GPIOREADPULSELENGTH => {
                outgoing[0] = command;
                outgoing[1] = gpio.read_pulse_length();
                2 as uint
            },
            command::GPIOSETINTERRUPTMODE => {
                gpio.set_interrupt_mode(incoming[2]);
                outgoing[0] = command;
                1 as uint
            },
            command::GPIOGETINTERRUPTMODE => {
                outgoing[0] = command;
                outgoing[1] = gpio.get_interrupt_mode();
                2 as uint
            },
            _ => 0 as uint
        }
    }
}

//#[cfg(test)]
pub mod test {
    use super::CommState;
    use super::command;
    use super::SPI;
    use super::I2C;
    use super::UART;
    use super::GPIO;

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

    #[deriving(Copy, Eq, PartialEq, Clone, Show)]
    pub struct MockI2C {
        pub enable : bool,
        pub out_reg : u8,
        pub slave_address: u8,
        pub mode : u8,
    }

    impl I2C for MockI2C {
        fn enable(&mut self) {
            self.enable = true;
        }
        fn write(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint {
            if self.enable {
                for x in range(0u, incoming.len()) {
                    outgoing[x] = incoming[x];
                }
            }
            incoming.len()
        }
        fn read(&mut self, length: u8, outgoing: &mut [u8]) -> uint {
            for x in range(0u, length as uint) {
                outgoing[x] = x as u8;
            }
            length as uint
        }
        fn disable(&mut self) {
            self.enable = false;
        }
        fn set_slave_address(&mut self, address: u8) {
            self.slave_address = address;
        }
        fn set_mode(&mut self, mode: u8) {
            self.mode = mode;
        }
    }

    #[deriving(Copy, Eq, PartialEq, Clone, Show)]
    pub struct MockUART {
        pub enable : bool,
        pub out_reg : u8,
        pub baudrate: u32,
        pub parity : u8,
        pub stop_bits: u8,
        pub data_bits : u8,
    }

    impl UART for MockUART {
        fn enable(&mut self) {
            self.enable = true;
        }
        fn transfer(&mut self, incoming: &[u8], outgoing: &mut [u8]) -> uint {
            if self.enable {
                for x in range(0u, incoming.len()) {
                    outgoing[x] = incoming[x];
                }
            }
            incoming.len()
        }
        fn disable(&mut self) {
            self.enable = false;
        }
        fn set_baudrate(&mut self, baudrate: u32) {
            self.baudrate = baudrate;
        }
        fn set_data_bits(&mut self, data_bits: u8) {
            self.data_bits = data_bits;
        }
        fn set_parity(&mut self, parity: u8) {
            self.parity = parity;
        }
        fn set_stop_bits(&mut self, stop_bits: u8) {
            self.stop_bits = stop_bits;
        }
    }

    #[deriving(Copy, Eq, PartialEq, Clone, Show)]
    pub struct MockGPIO {
        pub pull : u8,
        pub direction: u8,
        pub digital_value : u8,
        pub analog_value: u8,
        pub pwm_value : u8,
        pub interrupt : u8,
    }

    impl GPIO for MockGPIO {
        fn set_pull(&mut self, pull: u8) {
            self.pull = pull;
        }
        fn set_direction(&mut self, direction: u8) {
            self.direction = direction;
            println!("You set the direction of this pin to: {}", self.direction);
        }
        fn write_digital_value(&mut self, value: u8) {
            self.digital_value = value;
            println!("You set the state of this pin to: {}", self.digital_value);
        }
        fn write_analog_value(&mut self, value: u8) {
            self.analog_value = value;
        }
        fn write_pwm_value(&mut self, value: u8) {
            self.pwm_value = value;
        }
        fn get_pull(&mut self) -> u8 {
            self.pull
        }
        fn get_direction(&mut self) -> u8 {
            self.direction
        }
        fn read_digital_value(&mut self) -> u8 {
            self.digital_value
        }
        fn read_analog_value(&mut self) -> u8 {
            self.analog_value
        }
        fn read_pulse_length(&mut self) -> u8 {
            self.pwm_value
        }
        fn set_interrupt_mode(&mut self, interrupt: u8) {
            self.interrupt = interrupt;
        }
        fn get_interrupt_mode(&mut self) -> u8 {
            self.interrupt
        }
    }
}
