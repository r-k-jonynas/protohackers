use primes;
use std::io;
use serde_json::{json, Value};

use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

enum ResponseWrapped {
    Malformed(serde_json::value::Value),
    Conforming(serde_json::value::Value),
}

fn get_malformed_response() -> ResponseWrapped {
    ResponseWrapped::Malformed(json!({"method":"isPrime","prime":1}))
}

fn get_response_for_number(n: u64) -> ResponseWrapped {
    let is_prime = primes::is_prime(n);
    match is_prime {
        true => ResponseWrapped::Conforming(json!({"method":"isPrime","prime": is_prime})),
        false => ResponseWrapped::Malformed(json!({"method":"isPrime","prime": is_prime})),
    }
}

fn handle_response(vec: &Vec<u8>) -> ResponseWrapped {
    let json: serde_json::value::Value = match serde_json::from_slice(&vec) {
        Ok(json) => json,
        Err(_) => return get_malformed_response(),
    };
    println!("Received json {:?}", json);
    match (json.get("method"), json.get("number")) {
        (Some(method), Some(number)) => match (method, number) {
            (Value::String(ref method), Value::Number(ref number)) => {
                match (method.as_str(), number.as_u64()) {
                    ("isPrime", Some(n)) => {
                        println!("Received number {:?}", n);
                        return get_response_for_number(n);
                    }
                    (_, _) => return get_malformed_response(),
                }
            }
            (_, _) => return get_malformed_response(),
        },
        (_, _) => return get_malformed_response(),
    }
}

async fn process(mut socket: TcpStream) {
    let mut vec = Vec::<u8>::new();
    let mut buf = [0; 1024];
    let mut _r_counter = 0;
    loop {
        socket.readable().await.unwrap();
        let n = match socket.try_read(&mut buf) {
            Ok(0) => return (),
            Ok(n) => {
                _r_counter += n;
                println!("read {} bytes", n);
                n
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("error {:?}", e);
                return ();
            }
        };
        // Collect sent bytes
        for i in 0..n {
            vec.push(buf[i]);
        }
        println!("message: {:?}", &buf[0..n]);
        match n > 1 {
            false => (),
            true => match buf[n - 1] {
                10 => {
                    println!("received \\n character.");
                    // let full = std::str::from_utf8(&vec).unwrap();
                    // HANDLE
                    let response: ResponseWrapped = handle_response(&vec);
                    socket.writable().await.unwrap();
                    match response {
                        ResponseWrapped::Conforming(resp) => {
                            let str_of_json = serde_json::to_string(&resp).unwrap();
                            socket.write_all(str_of_json.as_bytes()).await.unwrap();
                            vec = Vec::<u8>::new();
                        }
                        ResponseWrapped::Malformed(resp) => {
                            let str_of_json = serde_json::to_string(&resp).unwrap();
                            socket.write_all(str_of_json.as_bytes()).await.unwrap();
                            break;
                        }
                    }
                }
                _ => {
                    println!("not yet full");
                    continue;
                }
            },
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
