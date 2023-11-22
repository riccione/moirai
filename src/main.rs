use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::io::{self, Read, BufRead, Write};
use std::str;
use std::thread;
use toml::Table;
use std::fs::{File, OpenOptions};
use serde::Deserialize;
use std::sync::Arc;

const HOST: &str = "127.0.0.1:8081"; // remove
const MSG_SIZE: usize = 1024;
const DST: [&str; 2] = ["0.0.0.0:10001", "0.0.0.0:10002"]; // remove
const CONFIG_FILE: &str = "config.toml";
const DEFAULT_TOML: &str = "[main]\n\
                            verbose = false\n\
                            \n\
                            [source]\n\
                            host = \"0.0.0.0\"\n\
                            port = 8080\n\
                            protocol = \"tcp\"\n\
                            \n\
                            [[server]]\n\
                            host = \"0.0.0.0\"\n\
                            port = 10001\n\
                            \n\
                            [[server]]\n\
                            host = \"0.0.0.0\"\n\
                            port = 10002\n\
                            ";

#[derive(Debug, Deserialize)]
struct Config {
    #[allow(dead_code)]
    main: Main,
    #[allow(dead_code)]
    source: Source,
    #[allow(dead_code)]
    server: Vec<Server>,
}

#[derive(Debug, Deserialize)]
struct Main {
    verbose: bool,
}

#[derive(Debug, Deserialize)]
struct Source {
    host: String,
    port: u16,
    protocol: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Server {
    host: String,
    port: u16,
}

fn main() {
    let conf = Arc::new(read_config());
    let tcp_conf = Arc::clone(&conf);
    println!("{:#?}", conf);
    match conf.source.protocol.as_deref() {
        Some("tcp") => {
            let _ = tcp_listener(tcp_conf);
        }
        Some("udp") => {
            let _ = udp_listener();
        }
        _ => {
            thread::spawn(move || {
                let _ = tcp_listener(tcp_conf);
            });
            
            println!("{:#?}", conf);
            let _ = udp_listener();
        },
    }    
}

fn read_config() -> Config {
    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(CONFIG_FILE) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error opening or creating config file: {e}");
                panic!("No config file");
            }
        };
    let mut c = String::new();
    file.read_to_string(&mut c)
        .expect("Failed to read file");

    let content = if c.is_empty() {
        file.write_all(DEFAULT_TOML.as_bytes())
            .expect("Failed to write config toml ");
        DEFAULT_TOML
    } else {
        &c
    }.to_string();

    let config: Config = toml::from_str(&content)
        .expect("Failed to deserialize");
    
    // println!("{:#?}", config);
    // let src = format!("{}:{}", 
    //    config.source.host, 
    //    config.source.port);

    // println!("{src}");
    config
}

fn tcp_listener(conf: Arc<Config>) -> io::Result<()> {
    let listener = TcpListener::bind(HOST)
        .expect("Failed to bind to TCP");
    
    println!("Listen {}", HOST);
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
    
    println!("Listen UDP");
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
    println!("{:?}", msg);
    let host = DST[0];
    let mut stream = TcpStream::connect(host)?;

    // write_all() will return Err(io::Error(io::ErrorKind::Interrupted))
    // if it is unable to queue all bytes
    stream.write_all(msg)?;
    let _ = stream.flush();

    Ok(())
}
