use std::io;
use std::str;

use tokio::net::{TcpListener, TcpStream};

async fn process(socket: TcpStream) {
    let mut vec = Vec::<&str>::new();
    let mut buf = [0; 1024];
    let mut r_counter = 0;
    let mut last_received = Option::<usize>::default();
    loop {
        socket.readable().await.unwrap();
        {
            match socket.try_read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    r_counter += n;
                    println!("read {} bytes", n);
                    last_received = Some(n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    last_received = None;
                    continue;
                }
                Err(e) => {
                    println!("error {:?}", e);
                    break;
                }
            };
        }
        match last_received {
            None => (),
            Some(k) => {
                let bytes = &buf[0..k];
                println!("message: {:?}", bytes);
                vec.push(str::from_utf8(bytes.clone()).unwrap());
                //     match (&buf[k - 2], &buf[k - 1]) {
                //         (b'\x5c', b'\x6e') => {}
            }
        }
    }
    println!("somebody disconnected");
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let debug = true;
    let ip = match debug {
        true => "127.0.0.1",
        false => "0.0.0.0",
    };
    let port = "6142";
    let addr = format!("{}:{}", ip, port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", &addr);
    loop {
        let (socket, _) = listener.accept().await?;
        println!("received connection");
        tokio::spawn(async move { process(socket).await });
    }
}
