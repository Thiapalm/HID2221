#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Register {
    Iodir = 0x00,
    Ipol = 0x02,
    Gpinten = 0x04,
    Defval = 0x06,
    Intcon = 0x08,
    Iocon = 0x0A,
    Gppu = 0x0C,
    Intf = 0x0E,
    Intcap = 0x10,
    Gpio = 0x12,
    Olat = 0x14,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Port {
    Porta = 0x00,
    Portb = 0x01,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Pin {
    Pin0 = 0x01,
    Pin1 = 0x02,
    Pin2 = 0x04,
    Pin3 = 0x08,
    Pin4 = 0x10,
    Pin5 = 0x20,
    Pin6 = 0x40,
    Pin7 = 0x80,
    Invalid = 0x00,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Setclear {
    Clear,
    Set,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Direction {
    Output = 0x00,
    Input = 0xFF,
}

static MCP23017_ADDRESS: u8 = 0x20;
static MCP23017_IODIRA: u8 = 0x00;
static MCP23017_GPIOA: u8 = 0x12;

/**
 * This function is used to set a given bit. It must receive the byte to be changed
 * and the pin number to set
 */
pub fn bit_set(byte: u8, pin: Pin) -> u8 {
    byte | pin as u8
}

/**
 * This function is used to clear a given bit. It must receive the byte to be changed
 * and the pin number to be cleared
 */
pub fn bit_clear(byte: u8, pin: Pin) -> u8 {
    byte & !(pin as u8)
}

/**
 * This function reads a given bit from a byte. It must receive the byte and
 * the pin number to be read
 */
pub fn bit_read(byte: u8, pin: Pin) -> u8 {
    (byte & pin as u8) >> (pin as u8)
}
