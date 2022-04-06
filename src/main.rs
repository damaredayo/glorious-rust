use clap::Parser;
use hidapi::*;
use libc;

#[derive(Parser)]
#[clap(version = "1.0", author = "Tyler Thompson", about = "Glorious Model O Configuration software written in Rust.")]
struct Args {
    #[clap(short, long)]
    debounce_time: u8
}

fn main() {
    check_root();
    let args = Args::parse();

    let device = init_hid();

    println!("{:?}", device.get_product_string().unwrap());

    check_debounce_time(args.debounce_time);
    println!("dbt: {}ms", get_debounce_time(&device));
    set_debounce_time(&device, args.debounce_time);
}

// initalize hid connection
fn init_hid() -> HidDevice {
    let api = HidApi::new().unwrap();
    let mut devices = api.device_list();
    let supported_devices = devices.find(|d| d.vendor_id() == 0x258a && (d.product_id() == 0x27 || d.product_id() == 0x33 || d.product_id() == 0x36));
    if supported_devices.is_none() {
        panic!("No supported devices found");
    }

    let device = supported_devices.unwrap();

    if device.product_id() == 0x27 {
        println!("Found Dream Machines DM5");
    } else if device.product_id() == 0x33 {
        println!("Found Model D");
    } else if device.product_id() == 0x36 {
        println!("Found Model O");
    }

    // open device
    let device_inst = api.open_path(&device.path()).unwrap();
    
    return device_inst;
}

// function to set debounce time
fn set_debounce_time(device: &HidDevice, debounce_time: u8) {
    // make buffer and append debounce time
    let mut buffer = [0u8; 6];
    buffer[0] = 0x5;
    buffer[1] = 0x1a;
    buffer[2] = debounce_time/2;

    let err = device.send_feature_report(&mut buffer).err().unwrap();
    eprintln!("{}", err.to_string());
}

// function to get current debounce time
fn get_debounce_time(device: &HidDevice) -> u8 {

    let mut buffer = [0u8; 6];
    buffer[0] = 0x5;
    buffer[1] = 0x1a;

    match device.send_feature_report(&mut buffer){
        Ok(_) => {
            match device.get_feature_report(&mut buffer) {
                Ok(dev) => {
                    println!("dev: {:?}", dev);
                    return buffer[2]*2;
                }

                Err(err) => {
                    eprintln!("error: {}", err.to_string());
                    return 0;
                }
            }
        },
        Err(e) => {
            eprintln!("error: {}", e.to_string());
            return 0;
        }
    }
}

// check we are root
fn check_root() {
    if unsafe { libc::geteuid() } != 0 {
        // exit quietly if not root
        println!("Not running as root, exiting.");
        std::process::exit(0);
    }
}

// check debounce time is a sensible number
fn check_debounce_time(debounce_time: u8) {
    if debounce_time < 2 || debounce_time > 16 {
        panic!("Debounce time must be between 2 and 16");
    }
}
