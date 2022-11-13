use std::io;

use tokio::net::{TcpListener, TcpStream};

async fn process(socket: TcpStream) {
    // Copy data here
    let mut buf = [0; 1024];
    loop {
        socket.readable().await.unwrap();

        match socket.try_read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("read {} bytes", n);
                let message = &buf[0..n];
                // println!("message: {:?}", message);
                let mut sending_counter = n;
                while sending_counter > 0 {
                    match socket.try_write(&message) {
                        Ok(0) => return,
                        Ok(n_sent) => {
                            sending_counter -= n_sent;
                            println!("sent back {} bytes", n);
                        }
                        Err(e) => {
                            println!("error {:?}", e);
                            break;
                        }
                    }
                }
                buf.iter_mut().for_each(|m| *m = 0)
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("error {:?}", e);
                break;
            }
        }
    }
    println!("somebody disconnected");
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = "0.0.0.0:6142";
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on: {}", addr);
    loop {
        let (socket, _) = listener.accept().await?;
        println!("received connection");
        tokio::spawn(async move { process(socket).await });
    }
}
