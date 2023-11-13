use std::{
    net::TcpStream,
    io::{Write, stdin, Read, ErrorKind},
    sync::mpsc,
    thread,
};

fn main() {
    const BUF_SIZE: usize = 1024;

    let username = std::env::args()
        .nth(1)
        .expect("Usage: cargo run USERNAME [HOST]");

    let host = std::env::args()
        .nth(2)
        .unwrap_or(String::from("127.0.0.1:8080"));

    println!("Connecting to {host} as {username}...");

    let mut stream = TcpStream::connect(host)
        .expect("Failed to connect to host.");

    stream
        .set_nonblocking(true)
        .expect("Failed to set stream as non-blocking.");

    println!("Connected.");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        // Receive
        let mut buf = vec![0; BUF_SIZE];
        match stream.read_exact(&mut buf) {
            Ok(_) => {
                let msg = buf
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<_>>();

                let msg = String::from_utf8(msg)
                    .expect("Failed to convert message.");

                println!("{msg}");
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => (),
            Err(_) => break,
        }

        // Send
        if let Ok(msg) = rx.try_recv() {
            //stream.write_all(msg.as_bytes())
            //    .expect("Failed to write to mpsc channel.");
            let mut buf = msg.clone().into_bytes();
            buf.resize(BUF_SIZE, 0);
            stream.write_all(&buf)
                .expect("Failed to write to stream.");
        }
    });

    // Input
    loop {
        /*print!(">>> ");
        let _ = stdout().flush();*/
        let mut buf = String::new();
        stdin()
            .read_line(&mut buf)
            .expect("Failed to read line from stdin.");

        let msg = buf.trim();
        if msg == "!q" {
            break;
        } else {
            let msg = format!("{username}: {msg}");
            tx.send(msg.to_string())
                .expect("Failed to send message to mspc channel.");
        }
    }
}
