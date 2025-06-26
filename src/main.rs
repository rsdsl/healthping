use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const MAGIC: [u8; 4] = [0x32, 0x7f, 0xfe, 0x4c];
const RESP_OK: [u8; 5] = [0x32, 0x7f, 0xfe, 0x4c, 0x00];
const RESP_NORMAL: [u8; 5] = [0x32, 0x7f, 0xfe, 0x4c, 0x01];
const PING_INTERVAL: Duration = Duration::from_secs(12);
const TCP_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_RETRY: i32 = 30;

const EXIT_SUCCESS: i32 = 0;
const EXIT_USAGE: i32 = 1;
const EXIT_PING: i32 = 2;
const EXIT_BUG: i32 = 3;
const EXIT_IO: i32 = 4;

fn main() {
    let mut args = std::env::args().skip(1);

    let addr = match args.next() {
        Some(addr) => addr,
        None => {
            eprintln!("Usage: rsdsl_healthping host:port");
            std::process::exit(EXIT_USAGE);
        }
    };
    let addr: SocketAddr = match addr.parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("invalid address: {}", e);
            std::process::exit(EXIT_USAGE);
        }
    };

    let mut lasterr = None;

    print!("Pinging");
    for _ in 0..MAX_RETRY {
        print!(".");
        match io::stdout().flush() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("flush stdout: {}", e);

                std::thread::sleep(PING_INTERVAL);
                std::process::exit(EXIT_IO);
            }
        }

        let mut conn = match TcpStream::connect_timeout(&addr, TCP_TIMEOUT) {
            Ok(conn) => conn,
            Err(e) => {
                lasterr = Some(e);

                std::thread::sleep(PING_INTERVAL);
                continue;
            }
        };

        match conn.set_read_timeout(Some(TCP_TIMEOUT)) {
            Ok(_) => {}
            Err(e) => {
                lasterr = Some(e);

                std::thread::sleep(PING_INTERVAL);
                continue;
            }
        }

        match conn.set_write_timeout(Some(TCP_TIMEOUT)) {
            Ok(_) => {}
            Err(e) => {
                lasterr = Some(e);

                std::thread::sleep(PING_INTERVAL);
                continue;
            }
        }

        match conn.write_all(&MAGIC) {
            Ok(_) => {}
            Err(e) => {
                lasterr = Some(e);

                std::thread::sleep(PING_INTERVAL);
                continue;
            }
        }

        let mut buf = [0; 5];
        match conn.read_exact(&mut buf) {
            Ok(_) => {}
            Err(e) => {
                lasterr = Some(e);

                std::thread::sleep(PING_INTERVAL);
                continue;
            }
        }

        match buf {
            RESP_OK => success(),
            RESP_NORMAL => success_unexpected(),
            _ => eprintln!("got invalid response {:?}", buf),
        }

        std::thread::sleep(PING_INTERVAL);
    }

    println!();

    match lasterr {
        Some(e) => {
            eprintln!("{}", e);
            std::process::exit(EXIT_PING)
        }
        None => {
            eprintln!("max retries exceeded without error");
            std::process::exit(EXIT_BUG)
        }
    }
}

fn success() {
    println!();
    println!("Success");

    std::process::exit(EXIT_SUCCESS);
}

fn success_unexpected() {
    println!();
    println!("Host is not waiting for healthcheck ping");

    std::process::exit(EXIT_SUCCESS);
}
