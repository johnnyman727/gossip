gossip
======

A packet parsing library for inter-microcontroller communications. This library will accept an incoming buffer in the form of the [IO protocol](https://gist.github.com/kevinmehall/f637d99bf837225df6cd), parse into into discrete packets, and execute the actions dictated by those packets. Those actions may be:

* write GPIO value
* read GPIO value
* write GPIO pull value
* read GPIO pull value
* write GPIO direction
* read GPIO Direction
* write a GPIO interrupt
* read a pulse length
* read an analog value
* write an analog value
* write a pwm value


* initialize an I2C port
* write an I2C data buffer
* transfer an I2C data buffer
* read an I2C data buffer

* initialize an SPI port
* write a SPI data buffer
* transfer a SPI data buffer
* read a SPI data buffer

* initialize an UART port
* write a UART buffer

The library will also send a response for each type of packet. All "write" commands will send back an acknowledgement or an error code depending on the result of executing the function. All "read" commands will return the value that was read or an error code. GPIO Interrupts and UART read events can happen at any time. 

setGPIOPull(function *pull_gpio);

setSPITransfer(configure_spi_slave);

PARSING
transfer_spi <data>, i2c_transfer <data>, gpio_pull <dir>
configure_spi_slave(data);

enum {
  GPIO_PULL
  SPI_SEND
}

function arr[] = [<>, <>, ]

setCommandHandler(GPIO_PULL, reach_gpio_pull_func);

set_gpio_pull_func();
set_i2c_transfer_func()l;
set_spi_transfer_fiunc();

1. Parsing the binary packet
2. Set the appropriate command handler
3. Calling the command (threads?)
