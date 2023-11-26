use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
// Read
use std::io::{self, BufRead, Write};
use std::str;
use std::thread;
use std::sync::Arc;
use std::time::Duration;

mod config;

const MSG_SIZE: usize = 1024;
//const DST: [&str; 2] = ["0.0.0.0:10001", "0.0.0.0:10002"]; // remove

fn main() {
    //let conf = Arc::new(read_config());
    let conf = Arc::new(config::read_config());

    //let c = config::read_config();
    //println!("{:#?}", c);
    // println!("dst size: {}", conf.server.len());
    
    let c = Arc::clone(&conf);
    
    // for debug only
    println!("{:#?}", conf);
    match conf.source.protocol.as_deref() {
        Some("tcp") => {
            let _ = tcp_listener(c);
        }
        Some("udp") => {
            let _ = udp_listener(c);
        }
        _ => {
            thread::spawn(move || {
                let _ = tcp_listener(conf);
            });
            
            let _ = udp_listener(c);
        },
    } 
}

fn tcp_listener(conf: Arc<config::Config>) -> io::Result<()> {
    let src = format!("{}:{}",
        &conf.source.host,
        &conf.source.port);

    let listener = TcpListener::bind(&src)
        .expect("Failed to bind to TCP");
    
    println!("Listen TCP {}", &src);
    let mut i: usize = 0;
    for stream in listener.incoming() {
        match stream {
            Ok(x) => {
                read_data(x, i, conf.clone());
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

fn udp_listener(conf: Arc<config::Config>) {
    let src = format!("{}:{}",
        &conf.source.host,
        &conf.source.port);
    
    let socket = UdpSocket::bind(&src)
        .expect("Couldn't bind to addr");
    
    println!("Listen UDP {}", &src);
    let mut buf = [0; MSG_SIZE];
    loop {
        match socket.recv_from(&mut buf) {
            Ok(_x) => {
                println!("forward UDP to dst");
            }
            Err(e) => {
                eprintln!("{e}");
            }
        }
    }
}

fn read_data(mut x: TcpStream, i: usize, conf: Arc<config::Config>) {
    let mut reader = io::BufReader::new(&mut x);
    
    match reader.fill_buf() {
        Ok(x) => {
            // for debug only
            if conf.main.verbose {
                println!("{:?}", x);
                // print_bytes(x);
            }
            if x.len() > 0 {
                let _ = forward(x, conf, i);
            }
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
}

// for debug only
// convert bytes to utf8 and prints
fn _print_bytes(buf: &[u8]) {
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
fn _print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn _check_dst_health(host: &str) -> bool {
    let server: SocketAddr = host
        .parse()
        .expect("Unable to parse socket address");
    match TcpStream::connect_timeout(&server, Duration::from_secs(2)) {
        Ok(_) => true,
        Err(_e) => false,
    }
}

fn round_robin(dst: &Vec<config::Server>, i: usize) -> String {// &'static str {
    let x = &dst[i % dst.len()];
    let host = format!("{}:{}",
        x.host,
        x.port);
    host
}

fn forward(msg: &[u8], conf: Arc<config::Config>, i: usize) -> io::Result<()> {
    //println!("{:?}", msg);
    
    let host = round_robin(&conf.server, i);
    println!("Forward to {}", &host);
    let mut stream = TcpStream::connect(host)?;

    // write_all() will return Err(io::Error(io::ErrorKind::Interrupted))
    // if it is unable to queue all bytes
    stream.write_all(msg)?;
    let _ = stream.flush();

    Ok(())
}
