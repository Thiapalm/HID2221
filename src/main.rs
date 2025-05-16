use embedded_hal::blocking::i2c::Read;
use std::{process::exit, thread::sleep, time::Duration};
mod powmon;
mod tools;

use clap::{Parser, Subcommand};
use powmon::*;
use tools::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start ONT
    Start,
    /// Stop ONT
    Stop,
    /// Restart ONT
    Restart,
    /// ONT Status
    Status,
    ///Ethernet connection handling
    Ethernet {
        //valid actions: Start, Stop, Restart
        action: Actions,
    },
    ///Fiber connection handling
    Fiber {
        //valid actions: Start, Stop, Restart
        action: Actions,
    },
    /// Used to read Voltage, Current and Power
    Powmon { readings: PowRead },
    /// Search for device
    Scan,
}

#[derive(clap::ValueEnum, Copy, Clone, PartialEq, Eq, Debug)]
enum Actions {
    /// Start
    Start,
    /// Stop
    Stop,
    ///Restart
    Restart,
    ///Status
    Status,
}

#[derive(clap::ValueEnum, Copy, Clone, PartialEq, Eq, Debug)]
enum PowRead {
    /// Check device status
    Status,
    /// Voltage read
    Voltage,
    /// Current read
    Amps,
    /// Power read
    Power,
    /// Read Shunt voltage
    Shunt,
    /// Reset chip
    Reset,
}

#[derive(Parser, Copy, Clone, PartialEq, Eq, Debug)]
#[command(version, about, long_about = None)]
struct Config {
    mode: Option<u8>,
    vbusct: Option<u8>,
    vscht: Option<u8>,
    average: Option<u8>,
}

fn start_device() -> mcp2221::Handle {
    let mut config = mcp2221::Config::default();
    config.i2c_speed_hz = 400_000;
    config.reset_on_open = false;
    // For talking to a peripheral we might want a higher timeout, but for
    // scanning the bus, a short timeout is good since it allows us to scan all
    // addresses more quickly.
    config.timeout = Duration::from_millis(10);
    let dev = match mcp2221::Handle::open_first(&config) {
        Ok(x) => x,
        Err(err) => {
            println!("Error: {err}, check USB connection");
            exit(1);
        }
    };
    dev
}

fn main() {
    let cli = Cli::parse();
    let mut dev = start_device();
    set_pin_dir(&mut dev, Port::Porta, Pin::Pin3, Direction::Output); // ONT
    set_pin_dir(&mut dev, Port::Porta, Pin::Pin0, Direction::Output); // Ethernet
    set_pin_dir(&mut dev, Port::Porta, Pin::Pin5, Direction::Output); // Fiber

    match &cli.command {
        Some(Commands::Start) => {
            write_pin(&mut dev, Port::Porta, Pin::Pin3, 1);
        }
        Some(Commands::Stop) => {
            write_pin(&mut dev, Port::Porta, Pin::Pin3, 0);
        }
        Some(Commands::Restart) => {
            write_pin(&mut dev, Port::Porta, Pin::Pin3, 0);
            sleep(Duration::from_secs(1));
            write_pin(&mut dev, Port::Porta, Pin::Pin3, 1);
        }
        Some(Commands::Status) => {
            let status = read_pin(&mut dev, Port::Porta, Pin::Pin3);
            match status {
                0 => println!("OFF"),
                _ => println!("ON"),
            }
        }
        Some(Commands::Ethernet { action: x }) => match x {
            Actions::Start => {
                println!("Start Ethernet");
                write_pin(&mut dev, Port::Porta, Pin::Pin0, 1);
            }
            Actions::Stop => {
                println!("Stop Ethernet");
                write_pin(&mut dev, Port::Porta, Pin::Pin0, 0);
            }
            Actions::Restart => {
                println!("Restart Ethernet");
                write_pin(&mut dev, Port::Porta, Pin::Pin0, 0);
                sleep(Duration::from_secs(1));
                write_pin(&mut dev, Port::Porta, Pin::Pin0, 1);
            }
            Actions::Status => {
                let status = read_pin(&mut dev, Port::Porta, Pin::Pin0);
                match status {
                    0 => println!("OFF"),
                    _ => println!("ON"),
                }
            }
        },
        Some(Commands::Fiber { action: x }) => match x {
            Actions::Start => {
                println!("Start Fiber");
                write_pin(&mut dev, Port::Porta, Pin::Pin5, 1);
            }
            Actions::Stop => {
                println!("Stop Fiber");
                write_pin(&mut dev, Port::Porta, Pin::Pin5, 0);
            }
            Actions::Restart => {
                println!("Restart Fiber");
                write_pin(&mut dev, Port::Porta, Pin::Pin5, 0);
                sleep(Duration::from_secs(1));
                write_pin(&mut dev, Port::Porta, Pin::Pin5, 1);
            }
            Actions::Status => {
                let status = read_pin(&mut dev, Port::Porta, Pin::Pin5);
                match status {
                    0 => println!("OFF"),
                    _ => println!("ON"),
                }
            }
        },
        Some(Commands::Scan) => {
            let mut dev = start_device();
            match dev.check_bus() {
                Ok(()) => (),
                Err(x) => {
                    println!("Error: {x}");
                    exit(1);
                }
            }

            println!("{}", dev.get_device_info().unwrap());

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
        }
        Some(Commands::Powmon { readings: x }) => match x {
            PowRead::Status => match verify_hardware(&mut dev, 0x4A) {
                Ok(()) => {
                    println!("Device Found !");
                    let (x, y) = read_configuration(&mut dev, 0x4A);
                    println!("Configuration: {:?} 0b{:16b}", x, y);
                }
                Err(x) => {
                    println!("Error: {}", x);
                }
            },
            PowRead::Voltage => {
                config_hardware(&mut dev, 0x4A);
                calibrate(&mut dev, 0x4A, 5.0);
                let result = read_volts(&mut dev, 0x4A);
                println!("{} V", result);
            }
            PowRead::Amps => {
                config_hardware(&mut dev, 0x4A);
                calibrate(&mut dev, 0x4A, 5.0);
                let result = read_amps(&mut dev, 0x4A);
                println!("{} A", result);
            }
            PowRead::Power => {
                config_hardware(&mut dev, 0x4A);
                calibrate(&mut dev, 0x4A, 5.0);
                let result = read_power(&mut dev, 0x4A);
                println!("{} W", result);
            }
            PowRead::Shunt => {
                config_hardware(&mut dev, 0x4A);
                calibrate(&mut dev, 0x4A, 5.0);
                let result = read_shunt(&mut dev, 0x4A);
                println!("{} V", result);
            }
            PowRead::Reset => {
                reset(&mut dev, 0x4A);
                println!("Reset!");
            }
        },
        None => {}
    }
}

fn set_pin_dir(dev: &mut mcp2221::Handle, port: Port, pin: Pin, dir: Direction) {
    let reg = Register::Iodir;
    let mut result = read_register(dev, reg, port);

    result = match dir {
        Direction::Input => bit_set(result, pin),
        Direction::Output => bit_clear(result, pin),
    };
    write_register(dev, port, reg, result);
}

fn write_pin(dev: &mut mcp2221::Handle, port: Port, pin: Pin, value: u8) {
    let reg = Register::Olat;
    let mut result = read_register(dev, reg, port);
    //println!("result before: 0x{:08b} set/clear: {}", result, value);

    result = match value {
        1 => bit_set(result, pin),
        _ => bit_clear(result, pin),
    };
    //println!("result after: 0x{:08b}", result);
    write_register(dev, port, reg, result);
}

fn read_pin(dev: &mut mcp2221::Handle, port: Port, pin: Pin) -> u8 {
    let reg = Register::Gpio;
    let result = read_register(dev, reg, port);

    bit_read(result, pin)
}
