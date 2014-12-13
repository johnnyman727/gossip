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

