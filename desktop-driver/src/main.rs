extern crate pnet;
extern crate serialport;

use pnet::datalink::{self, NetworkInterface};

use std::{
    collections::HashMap,
    env::{self},
    os::raw::c_char,
    time::Duration,
    usize, vec,
};

use serialport::*;

const ARDUINO_INITIALIZE_HEADER: [c_char; 8] = [
    'f' as c_char,
    'e' as c_char,
    'r' as c_char,
    'n' as c_char,
    'o' as c_char,
    'c' as c_char,
    'a' as c_char,
    't' as c_char,
];
const ARDUINO_FIRST_CHAR: c_char = 'f' as c_char;
const ARDUINO_LAST_CHAR: c_char = 't' as c_char;

fn main() {
    println!("Found interfaces:");
    let interfaces = datalink::interfaces();
    for iface in &interfaces {
        println!("{} ({}):{:?}", iface.name, iface.description, iface.ips);
    }

    let arguments: Vec<String> = env::args()
        .filter(|s| !s.is_empty() && !s.contains("desktop-driver"))
        .collect();

    let interfaces_to_send: Vec<NetworkInterface> = if !arguments.is_empty() {
        println!("Filtering for {:?}", arguments);
        let args: Vec<String> = arguments.iter().map(|f| f.to_lowercase()).collect();

        interfaces
            .iter()
            .filter(|&e| args.contains(&e.name.to_lowercase()))
            .cloned()
            .collect()
    } else {
        interfaces
    };

    println!("Interfaces to send: {}", &interfaces_to_send.len());

    let ports = serialport::available_ports().expect("No ports found!");
    let mut serialports: HashMap<String, Box<dyn SerialPort>> = HashMap::new();

    loop {
        // Iterate through the registered ports
        send_to_serialports(&mut serialports, &interfaces_to_send);

        get_serialports(&ports, &mut serialports);
    }
}

fn get_serialports(
    ports: &[SerialPortInfo],
    serialports: &mut HashMap<String, Box<dyn SerialPort>>,
) {
    'portLoop: for port_info in ports {
        let port_opt = &serialports.get(&port_info.port_name);

        match port_opt {
            None => {
                let port_result = serialport::new(&port_info.port_name, 9600)
                    .timeout(Duration::from_millis(10))
                    .open();

                match port_result {
                    Ok(result) => {
                        serialports.insert(port_info.port_name.clone(), result);
                    }
                    Err(_) => {
                        continue 'portLoop;
                    }
                }
            }
            Some(_) => continue 'portLoop,
        };
    }
}

fn send_ip(port: &mut Box<dyn serialport::SerialPort>, interfaces: &[NetworkInterface]) {
    for interface in interfaces {
        println!("Sending interface {:?}!", interface.ips);

        for ip in &interface.ips {
            let ip_str = ip.ip().to_string() + "\n";

            println!("Sending ip {}", ip.ip().to_string());
            let output = ip_str.as_bytes();
            if let Err(e) = port.write_all(output) {
                eprintln!("Error while writing to Arduino {}", &e);
            }
        }
    }
}

fn send_to_serialports(
    serialports: &mut HashMap<String, Box<dyn SerialPort>>,
    interfaces: &[NetworkInterface],
) {
    let mut to_remove = Vec::<String>::new();

    'serialLoop: for port_pair in serialports.iter_mut() {
        let port_name = port_pair.0.clone();
        let port = port_pair.1;

        let mut serial_buftemp: Vec<u8> = vec![0; ARDUINO_INITIALIZE_HEADER.len() + 30];

        let mut start: usize;
        let mut end: usize;

        // Read block
        'readLoop: loop {
            if port.read(serial_buftemp.as_mut_slice()).is_err() {
                to_remove.push(port_name);
                continue 'serialLoop;
            }

            // Get all the data, remove the useless 0 junk
            serial_buftemp = serial_buftemp
                .iter()
                .filter(|&e| *e != 0_u8)
                .copied()
                .collect();

            // Find the first char
            let start_opt = serial_buftemp
                .iter()
                .rposition(|&e| e as c_char == ARDUINO_FIRST_CHAR);

            // If char not found, read again
            match start_opt {
                Some(p) => start = p,
                None => continue 'readLoop,
            }

            // If there is no end char, just skip
            if !serial_buftemp.contains(&(ARDUINO_LAST_CHAR as u8)) {
                continue 'readLoop;
            }

            // Reset the vector to start at the char
            let vec = (&serial_buftemp[start..]).to_vec();
            serial_buftemp = vec;

            start = 0;
            end = 0;

            // Find the last char
            if serial_buftemp.contains(&(ARDUINO_LAST_CHAR as u8)) {
                end = serial_buftemp
                    .iter()
                    .rposition(|&e| e == '\n' as c_char as u8)
                    .unwrap_or(1)
                    - 1;
            }

            // If the last char exists and is greater than start, we have the full ping
            if end != 0 && end > start {
                println!("Got the request ping");
                let vec = (&serial_buftemp[..end]).to_vec();
                serial_buftemp = vec;

                break 'readLoop;
            } else {
                continue 'readLoop;
            }
        }

        let serial_buf_full: Vec<c_char> = serial_buftemp.iter().map(|&f| f as c_char).collect();

        // Validate we have the full ping
        if serial_buf_full == ARDUINO_INITIALIZE_HEADER {
            send_ip(port, interfaces);
        }
    }

    for remove in to_remove {
        serialports.remove(&remove);
    }
}
