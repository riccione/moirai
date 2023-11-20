use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::io::{self, Read, BufRead, Write};
use std::str;
use std::thread;
use toml::Table;
use std::fs::{File, OpenOptions};
use serde::Deserialize;
use std::collections::HashMap;

const HOST: &str = "0.0.0.0:8080";
const MSG_SIZE: usize = 1024;
const DST: [&str; 2] = ["0.0.0.0:10001", "0.0.0.0:10002"];
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Deserialize)]
struct Config {
    server: HashMap<String, Server>,
}

#[derive(Debug, Deserialize)]
struct Server {
    dst: String,
}

fn main() {
    read_config();
    thread::spawn( || {
        let _ = tcp_listener();
    });
    let _ = udp_listener();
}

fn read_config() {
    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(CONFIG_FILE) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error opening or creating config file: {e}");
                return;
            }
        };
    let mut c = String::new();
    file.read_to_string(&mut c)
        .expect("Failed to read file");

    let content = if c.is_empty() {
        let default_toml = "[servers]\n\
                            [server.src]\n\
                            dst = \"0.0.0.0:8080\"\n\
                            \n\
                            [server.alpha]\n\
                            dst = \"0.0.0.0:10001\"\n\
                            [server.beta]\n\
                            dst = \"0.0.0.0:10002\"\n\
                            ";
        file.write_all(default_toml.as_bytes())
            .expect("Failed to write config toml ");
        default_toml
    } else {
        &c
    }.to_string();
    // println!("{c}");

    let config: Config = toml::from_str(&content)
        .expect("Failed to deserialize");
  
    println!("{}", config.server["src"].dst);
    print_type_of(&config.server);
    for (k, v) in &config.server {
        println!("Server {k} dst {0}", v.dst);
    }
    println!("{:?}", config.server);
    println!("{:#?}", config);
    /*
    // parse toml file
    let parsed = c.parse::<Table>().unwrap();
    let it = parsed.iter();
    for val in it {
        println!("{:?}", val);
    }
    let src = parsed["source"]["src"].as_str().unwrap();
    //let dst = parsed["server"]["1"]["dst"].as_str().unwrap();
    
    let dst = &parsed["server"].as_table();
    
    let p: Destination = match dst.try_into() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    
    println!("{:?}", p);

    println!("{}", src);
    match dst {
        Some(x) => {
            for v in x.iter() {
                println!("{:?}", v);
            }

            // println!("{:?}", print_type_of(x));
        },
        _ => {
            println!("Dst is not defined");
        }
    }
    //println!("{:?}", dst.as_table());
    //println!("{}", dst);
    */
}

fn tcp_listener() -> io::Result<()> {
    let listener = TcpListener::bind(HOST)?;
    
    let mut i: usize = 0;
    for stream in listener.incoming() {
        match stream {
            Ok(x) => {
                read_data(x, i);
            }
            Err(e) => {
                // connection failed
                eprintln!("{e}");
            }
        }
        i += 1;
    }

    Ok(())
}

fn udp_listener() {
    let socket = UdpSocket::bind(HOST)
        .expect("Couldn't bind to addr");

    let mut buf = [0; MSG_SIZE];
    // print_type_of(&buf);
    loop {
        let (_amt, _src) = socket.recv_from(&mut buf)
            .expect("Read from socket");
    }
}

fn read_data(mut x: TcpStream, i: usize) {
    // let mut rx_bytes = [0u8; MSG_SIZE];
    let mut reader = io::BufReader::new(&mut x);
    
    match reader.fill_buf() {
        Ok(x) => {
            // for debug only
            // print_bytes(x);
            if x.len() > 0 {
                let _ = forward(x, i);
            }
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
}

fn print_bytes(buf: &[u8]) {
    let received = str::from_utf8(&buf);
    match received {
        Ok(x) => {
            println!("{x}");
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
}

// for debug only
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn round_robin(i: usize) -> &'static str {
    DST[i % DST.len()]
}

fn forward(msg: &[u8], i: usize) -> io::Result<()> {
    let host = DST[0];
    let mut stream = TcpStream::connect(host)?;

    // write_all() will return Err(io::Error(io::ErrorKind::Interrupted))
    // if it is unable to queue all bytes
    stream.write_all(msg)?;
    let _ = stream.flush();

    Ok(())
}
