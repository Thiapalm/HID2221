use embedded_hal::blocking::i2c::Read;
//use std::time::Duration;
use std::{thread::sleep, time::Duration};
mod tools;

use tools::*;

fn main() {
    if let Err(error) = run() {
        println!("Error: {}", error);
    }
}

fn run() -> mcp2221::Result<()> {
    let mut config = mcp2221::Config::default();
    config.i2c_speed_hz = 400_000;
    // For talking to a peripheral we might want a higher timeout, but for
    // scanning the bus, a short timeout is good since it allows us to scan all
    // addresses more quickly.
    config.timeout = Duration::from_millis(10);
    let mut dev = mcp2221::Handle::open_first(&config)?;

    // Set GPIO pin 0 high. This is useful if your I2C bus goes through a level
    // shifter and you need to enable that level shifter in order to use the I2C
    // bus. It also serves as an example of using GPIO.
    // let mut gpio_config = mcp2221::GpioConfig::default();
    // gpio_config.set_direction(0, mcp2221::Direction::Output);
    // gpio_config.set_value(0, true);
    // dev.configure_gpio(&gpio_config)?;

    // Before we start, SDA and SCL should be high. If they're not, then either
    // the pull-up resistors are missing, the bus isn't properly connected or
    // something on the bus is holding them low. In any case, we won't be able
    // to operate.
    dev.check_bus()?;

    println!("{}", dev.get_device_info()?);

    for base_address in (0..=127).step_by(16) {
        for offset in 0..=15 {
            let address = base_address + offset;
            match dev.read(address, &mut [0u8]) {
                Ok(_) => print!("0x{:02x}", address),
                Err(_) => print!(" -- "),
            }
        }
        println!();
    }
    let mut value = 0;

    dev.write_read_address(
        0x20,
        &set_pin(Port::Porta, Pin::Pin0, Direction::Output),
        &mut [0u8],
    );
    loop {
        //dev.write_read_address(0x20, &write_pin(Port::Porta, Pin::Pin0, value), &mut [0u8]);
        println!("value: {value} Pin3");
        write_pin(&mut dev, Port::Porta, Pin::Pin3, value);
        println!("value: {value} Pin4"); //todo quando escrevo em outro pino, está limpando o anterior
        //write_pin(&mut dev, Port::Porta, Pin::Pin4, value);
        sleep(Duration::from_secs(1));
        value ^= 0b0000_0001;
    }
}

fn cancel_current() -> [u8; 5] {
    let mut buffer: [u8; 5] = [0; 5];
    buffer[0] = 0x10;
    buffer[1] = 0x00;
    buffer[2] = 0x10;
    buffer[3] = 0x00;
    buffer[4] = 0x00;

    buffer
}

fn set_pin(port: Port, pin: Pin, dir: Direction) -> [u8; 2] {
    let mut buffer: [u8; 2] = [0; 2];

    buffer[0] = Register::Iodir as u8 | port as u8;
    buffer[1] = dir as u8;

    buffer
}

fn read_register(dev: &mut mcp2221::Handle, reg: Register, port: Port) -> u8 {
    let mut read: [u8; 1] = [0; 1];
    let mut result: [u8; 1] = [0; 1];
    read[0] = reg as u8 | port as u8;
    dev.write_read_address(0x20, &mut read, &mut result);

    result[0]
}

fn write_register(dev: &mut mcp2221::Handle, port: Port, reg: Register, value: u8) {
    let mut result: [u8; 2] = [0; 2];
    result[0] = reg as u8 | port as u8;
    result[1] = value;
    match dev.write_read_address(0x20, &mut result, &mut [0u8]) {
        Ok(x) => (),
        Err(x) => println!("Error: {}", x),
    }
}

fn write_pin(dev: &mut mcp2221::Handle, port: Port, pin: Pin, value: u8) {
    let mut buffer: [u8; 2] = [0; 2];
    let reg = Register::Olat;
    let mut result = read_register(dev, reg, port);
    println!("result before: 0x{:08b} set/clear: {}", result, value);

    result = match value {
        1 => bit_set(result, pin),
        _ => bit_clear(result, pin),
    };
    println!("result after: 0x{:08b}", result);
    write_register(dev, port, reg, result);
}

/*
use std::{thread::sleep, time::Duration};

use hidapi::{HidApi, HidError};

static DEVICE_NAME: &str = "MCP2221";
static DEFAULT_PID: u16 = 0x00dd;
static DEFAULT_VID: u16 = 0x04d8;
static MCP23017_ADDRESS: u8 = 0x20;
static MCP23017_IODIRA: u8 = 0x00;
static MCP23017_GPIOA: u8 = 0x12;

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

enum Port {
    Porta = 0x00,
    Portb = 0x01,
}

enum Pin {
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

enum Direction {
    Output = 0x00,
    Input = 0xFF,
}

enum I2cSpeed {
    _400kHz = 400_000,
}

fn transfer(&mut self, buffer: &[u8]) -> Result<()> {
    self.handle
        .write_interrupt(MCP_WRITE_ENDPOINT, buffer, self.config.timeout)?;

    let size =
        self.handle
            .read_interrupt(MCP_READ_ENDPOINT, &mut self.recv, self.config.timeout)?;
    if size != MCP_TRANSFER_SIZE {
        return Err(Error::ShortRead);
    }
    Ok(())
}

fn cmd(&mut self, command: u8, arg1: u8, arg2: u8, arg3: u8, data: &[u8]) -> Result<()> {
    assert!(data.len() <= MCP_MAX_DATA_SIZE);
    let mut buffer = [0u8; MCP_TRANSFER_SIZE];
    buffer[0] = command;
    buffer[1] = arg1;
    buffer[2] = arg2;
    buffer[3] = arg3;
    buffer[MCP_HEADER_SIZE..MCP_HEADER_SIZE + data.len()].copy_from_slice(data);
    self.transfer(&buffer)
}

fn seek_device() -> Result<(u16, u16), HidError> {
    match HidApi::new() {
        Ok(api) => {
            for device in api.device_list() {
                if device.vendor_id() == DEFAULT_VID && device.product_id() == DEFAULT_PID
                    || device.product_string().unwrap_or("").contains(DEVICE_NAME)
                {
                    println!("Device found!");
                    println!(
                        "VID: {:04x}, PID: {:04x}, Product name: {}, Interface: {}",
                        device.vendor_id(),
                        device.product_id(),
                        match device.product_string() {
                            Some(s) => s,
                            _ => "<COULD NOT FETCH>",
                        },
                        device.interface_number()
                    );
                    return Ok((device.vendor_id(), device.product_id()));
                }
            }
            Err(HidError::HidApiErrorEmpty)
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(e);
        }
    }
}

fn set_speed(speed: I2cSpeed) -> Result<()> {
    let divider: u8 = ((12_000_000 / speed as u32) - 2).try_into().unwrap();
    println!("Divider: {divider}");
    let mut buffer: [u8; 6] = [0; 6];
    buffer[1] = 0x10;
    buffer[2] = 0x00;
    buffer[3] = 0x00;
    buffer[4] = 0x20;
    buffer[5] = divider;

    buffer
}

fn set_pin(port: Port, pin: Pin, dir: Direction) -> [u8; 7] {
    let mut buffer: [u8; 7] = [0; 7];
    buffer[0] = 0x00;
    buffer[1] = 0x90;
    buffer[2] = 0x02;
    buffer[3] = 0x00;
    buffer[4] = MCP23017_ADDRESS << 1;
    buffer[5] = Register::Iodir as u8 | port as u8;
    buffer[6] = dir as u8;

    buffer
}

fn write_pin(port: Port, pin: Pin, value: u8) -> [u8; 7] {
    let mut buffer: [u8; 7] = [0; 7];
    buffer[0] = 0x00;
    buffer[1] = 0x90;
    buffer[2] = 0x02;
    buffer[3] = 0x00;
    buffer[4] = MCP23017_ADDRESS << 1;
    buffer[5] = Register::Gpio as u8 | port as u8;
    buffer[6] = match value {
        1 => 0xFF,
        _ => 0x00,
    };

    buffer
}

fn main() {
    println!("Printing all available hid devices:");
    let (vid, pid) = seek_device().unwrap();
    println!("VID: 0x{:04x}, PID: 0x{:04x}", vid, pid);
    println!(
        "Write Address: 0x{:02x} Read Address: 0x{:02x}",
        MCP23017_ADDRESS << 1,
        MCP23017_ADDRESS << 1 | 0x01,
    );
    let api = HidApi::new().expect("Failed to create API instance");
    let mcp23017 = api.open(vid, pid).expect("Failed to open device");

    let buf = set_speed(I2cSpeed::_400kHz);
    let res = mcp23017.write(&buf).unwrap();

    let buf = set_pin(Port::Porta, Pin::Pin0, Direction::Output);
    let res = mcp23017.write(&buf).unwrap();

    let buf = write_pin(Port::Porta, Pin::Pin0, 1);
    let res = mcp23017.write(&buf).unwrap();

    let mut value: u8 = 0;

    loop {
        let buf = write_pin(Port::Porta, Pin::Pin0, value);
        let res = mcp23017.write(&buf).unwrap();
        println!("value: {value} - res: {res}");
        sleep(Duration::from_secs(1));
        value ^= 0b0000_0001;
    }
}
*/
