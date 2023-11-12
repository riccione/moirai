use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::io::{self, Read, BufRead, Write};
use std::str;
use std::thread;

const HOST: &str = "0.0.0.0:8080";
const MSG_SIZE: usize = 1024;
const DST: [&str; 2] = ["0.0.0.0:10001", "0.0.0.0:10002"];

fn main() {
    thread::spawn( || {
        let _ = tcp_listener();
    });
    let _ = udp_listener();
}

fn tcp_listener() -> io::Result<()> {
    let listener = TcpListener::bind(HOST)?;

    for stream in listener.incoming() {
        match stream {
            Ok(x) => {
                read_data(x);
            }
            Err(e) => {
                // connection failed
                eprintln!("{e}");
            }
        }
    }

    Ok(())
}

fn udp_listener() {
    let socket = UdpSocket::bind(HOST)
        .expect("Couldn't bind to addr");

    let mut buf = [0; MSG_SIZE];
    // print_type_of(&buf);
    loop {
        let (amt, src) = socket.recv_from(&mut buf)
            .expect("Read from socket");
    }
}

fn read_data(mut x: TcpStream) {
    let mut rx_bytes = [0u8; MSG_SIZE];
    let mut reader = io::BufReader::new(&mut x);
    
    match reader.fill_buf() {
        Ok(x) => {
            print_bytes(x);
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

fn _forward(host: SocketAddr, msg: &[u8]) -> io::Result<()> {
    let mut stream = TcpStream::connect(host)?;

    // write_all() will return Err(io::Error(io::ErrorKind::Interrupted))
    // if it is unable to queue all bytes
    stream.write_all(msg)?;
    stream.flush();

    Ok(())
}
