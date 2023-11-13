use std::{
    net::{TcpListener, TcpStream, SocketAddr},
    sync::mpsc,
    thread,
    io::{Read, ErrorKind, Write},
};

struct Message {
    sender: SocketAddr,
    content: String,
}

fn main() {
    const BUF_SIZE: usize = 1024;

    let host = std::env::args()
        .nth(1)
        .unwrap_or(String::from("127.0.0.1:8080"));

    println!("Starting raum-server on {host}...");

    let listener = TcpListener::bind(host)
        .expect("Failed to bind to host.");

    listener
        .set_nonblocking(true)
        .expect("Failed to set listener as non-blocking.");

    println!("Started.");

    let mut clients: Vec<TcpStream> = vec![];
    let (tx, rx) = mpsc::channel::<Message>();

    loop {
        if let Ok((mut sock, addr)) = listener.accept() {
            println!("Client {} connected.", addr);

            clients.push(sock.try_clone()
                .expect("Failed to clone socket."));

            let tx = tx.clone();
            thread::spawn(move || loop {
                let mut buf = vec![0; BUF_SIZE];
                match sock.read_exact(&mut buf) {
                    Ok(_) => {
                        let content = buf
                            .into_iter()
                            .take_while(|&x| x != 0)
                            .collect::<Vec<_>>();

                        let content = String::from_utf8(content)
                            .expect("Failed to convert message.");

                        println!("Client {addr} sent message: {}", content.trim());

                        let msg = Message { 
                            sender: addr,
                            content 
                        };

                        tx.send(msg)
                            .expect("Failed to send to mpsc channel.");
                    },
                    Err(e) if e.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        eprintln!("Client {addr} disconnected.");
                        break;
                    },
                }
            });
        }

        if let Ok(msg) = rx.try_recv() {
            let mut buf = msg.content
                .clone()
                .into_bytes();

            buf.resize(BUF_SIZE, 0);

            for mut client in &clients {
                let addr = client
                    .peer_addr()
                    .expect("Failed to get Address of client.");

                if addr.port() != msg.sender.port() || addr.ip() != msg.sender.ip() {
                    client
                        .write_all(&buf)
                        .expect("Failed to write to stream.");
                }
            }
        }
    }
}
