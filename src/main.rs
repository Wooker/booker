#![feature(const_option_ops)]
#![feature(const_trait_impl)]

use std::process;

use std::{
    env,
    io::{Write, stdin, stdout},
    path::Path,
    process::exit,
    str::FromStr,
};

use booker::book::Book;

enum Movement {
    Next,
    Previous,
}

impl FromStr for Movement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if (s.len() == 2 && s.chars().nth(1).unwrap() == '\n') || s.len() == 1 {
            match s.chars().nth(0).expect("String is empty") {
                'n' => Ok(Movement::Next),
                'p' => Ok(Movement::Previous),
                _ => Err(String::from("Unknown movement")),
            }
        } else {
            Err(String::from("Unknown movement"))
        }
    }
}

#[cfg(feature = "1.54")]
const WIDTH: usize = 25;
#[cfg(feature = "1.54")]
const HEIGHT: usize = 20;

#[cfg(feature = "4.2")]
const WIDTH: usize = 37;
#[cfg(feature = "4.2")]
const HEIGHT: usize = 50;

const fn parse_usize(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i: usize = 0;
    let mut n: usize = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b < b'0' || b > b'9' {
            panic!("invalid digit in number");
        }
        n = n * 10 + (b - b'0') as usize;
        i += 1;
    }

    n
}
#[cfg(feature = "terminal")]
const WIDTH: usize = parse_usize(option_env!("COLUMNS").unwrap_or("20"));
#[cfg(feature = "terminal")]
const HEIGHT: usize = parse_usize(option_env!("LINES").unwrap_or("10")) - 2;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Should provide file path as an argument");
        println!("Supported flags:");
        println!("\t-s num\t Skip a number of pages");
        exit(1);
    }

    #[cfg(not(feature = "terminal"))]
    if args
        .iter()
        .find(|a| a.contains(&String::from("--port=")))
        .is_none()
    {
        println!("Should provide port device (example /dev/ttyUSB0)");
        println!("--port=    Select serial device");
        exit(1);
    }

    let port = if let Some(value) = args
        .iter()
        .find(|a| a.contains("--port="))
        .unwrap()
        .split_once("=")
    {
        if value.1.is_empty() {
            println!("Must provide value for port");
            exit(1);
        } else {
            value.1
        }
    } else {
        exit(1);
    };

    let skip_pages = if let Some(flag) = &args.get(2)
        && *flag == "-s"
    {
        if let Some(val) = &args.get(3) {
            str::parse(val).unwrap()
        } else {
            println!("Should provide value for flag -s");
            process::exit(2);
        }
    } else {
        0
    };
    let book: Book<WIDTH, HEIGHT> =
        Book::new(Path::new(&args.get(1).unwrap())).expect("No such file");

    let mut input = String::new();

    let pages: Vec<[[u8; WIDTH]; HEIGHT]> = book.into_iter().collect();

    let mut index: usize = skip_pages;
    'main: loop {
        let page = pages.get(index).expect("No such page");
        println!("{}/{}", index, pages.len());

        let mut lines: Vec<_> = vec![];

        // Append data to commands list
        for line_bytes in page.iter() {
            let line = String::from_utf8_lossy(line_bytes)
                .replace("�", " ")
                .to_string();
            lines.push(line.clone());
            #[cfg(feature = "terminal")]
            println!("{}", line);
        }
        #[cfg(not(feature = "terminal"))]
        {
            use nebula_host::Command;
            // lines.push(String::from("show"));
            // Serialize and send commands
            if !lines.is_empty() {
                use eink::Config;
                let mut arr = [0; WIDTH * HEIGHT * 2];

                let conf = Config {
                    pos: eink::Position { x: 0, y: 0 },
                    dir: eink::Direction::XuYiXi,
                    size: eink::Size {
                        width: 200,
                        height: 200,
                    },
                    partial: true,
                };
                println!("Config");
                let payload =
                    nebula_host::postcard::to_slice(&eink::Command::Config(conf), &mut arr)
                        .unwrap();
                let args = nebula_host::comm::handlers::invoke::InvokeArgs::new(
                    port.to_string(),
                    "reade".to_string(),
                    payload.to_vec(),
                    "30".to_string(),
                );
                match nebula_host::comm::handlers::invoke::handle(args) {
                    Ok(()) => {}
                    Err(()) => continue 'main,
                }

                println!("Text");
                let payload = nebula_host::postcard::to_slice(
                    &eink::Command::Text(lines.join("").as_bytes()),
                    &mut arr,
                )
                .unwrap();

                let args = nebula_host::comm::handlers::invoke::InvokeArgs::new(
                    port.to_string(),
                    "reade".to_string(),
                    payload.to_vec(),
                    "50".to_string(),
                );
                match nebula_host::comm::handlers::invoke::handle(args) {
                    Ok(()) => {}
                    Err(()) => continue 'main,
                }

                println!("Show");
                let payload =
                    nebula_host::postcard::to_slice(&eink::Command::Show, &mut arr).unwrap();
                let args = nebula_host::comm::handlers::invoke::InvokeArgs::new(
                    port.to_string(),
                    "reade".to_string(),
                    payload.to_vec(),
                    "20".to_string(),
                );
                match nebula_host::comm::handlers::invoke::handle(args) {
                    Ok(()) => {}
                    Err(()) => continue 'main,
                }
            }
        }

        let mut lock = stdout().lock();

        // Wait for user input
        loop {
            #[cfg(not(feature = "terminal"))]
            {
                println!("{}/{}", index, pages.len());
                use spaceport::packet::{MAX_PAYLOAD_LENGTH, Packet};
                let mut payload = [0; MAX_PAYLOAD_LENGTH];
                let args = nebula_host::comm::handlers::listen::ListenArgs::new(port.to_string());

                if let Some(message) = nebula_host::comm::handlers::listen::read(&args) {
                    match Packet::decode(&message, &mut payload) {
                        Ok(packet) => {
                            print!("{:?}", packet,);
                            if let Ok(msg) = String::from_utf8(packet.payload.to_vec()) {
                                if msg == "booker next" {
                                    if index + 1 < pages.len() {
                                        index += 1;
                                        break;
                                    } else {
                                    }
                                } else if msg == "booker prev" {
                                    if let Some(new_index) = index.checked_sub(1) {
                                        index = new_index;
                                        break;
                                    } else {
                                    }
                                }
                            } else {
                                println!();
                            }
                        }
                        Err(e) => {
                            eprintln!("{:?}", e)
                        }
                    }
                }
            }
            #[cfg(feature = "terminal")]
            {
                write!(lock, "\n>").unwrap();
                lock.flush().unwrap();
                stdin().read_line(&mut input).unwrap();
                if let Ok(m) = input.parse::<Movement>() {
                    match m {
                        Movement::Next => {
                            if index + 1 < pages.len() {
                                index += 1;
                                input.clear();
                                break;
                            } else {
                                write!(lock, "No next page").unwrap();
                                input.clear();
                            }
                        }
                        Movement::Previous => {
                            if let Some(new_index) = index.checked_sub(1) {
                                index = new_index;
                                input.clear();
                                break;
                            } else {
                                write!(lock, "No previous page").unwrap();
                                input.clear();
                            }
                        }
                    }
                } else {
                    write!(lock, "\nUnknown input\n").unwrap();
                    input.clear();
                }
            }

            input.clear();
        }
    }
}
