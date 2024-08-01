use core::fmt;
use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{mpsc, Arc};
use std::thread;

use chrono;

const SAFE_MODE: bool = true;
const LOCALHOST: &str = "127.0.0.1:8080";

fn truncate_to_nonzeros(vec: &mut Vec<u8>) -> Result<Vec<u8>, ()> {
    let len = vec.iter().position(|&x| x == 0).unwrap_or(vec.len());
    vec.truncate(len);
    Ok(vec.to_vec())
}

struct Sensitive<T> {
    value: T,
}

impl<T> Sensitive<T> {
    #[allow(unused)]
    fn new(value: T) -> Self {
        Sensitive { value }
    }
}

impl<T: fmt::Display> fmt::Display for Sensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if SAFE_MODE {
            writeln!(f, "[CENSORED]")
        } else {
            writeln!(f, "{}", self.value)
        }
    }
}

#[derive(Debug)]
enum Notification {
    ClientConnection(Arc<TcpStream>),
    ClientDisconnection(Arc<TcpStream>),
    NewMessage(ClientMessage),
}

#[derive(Debug)]
struct ClientMessage {
    author: Arc<SocketAddr>,
    date: chrono::DateTime<chrono::Utc>,
    msg: Vec<u8>,
}

impl fmt::Display for ClientMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "author: {:?}\ndate: {:?}\nmsg: {:?}",
            self.author, self.date, self.msg
        )
    }
}

impl fmt::Display for Notification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // using debug for now
            Notification::ClientConnection(client) => {
                writeln!(f, "Notification::ClientConnection: {:?}", client)
            }

            Notification::ClientDisconnection(client) => {
                writeln!(f, "Notification::ClientDisconnection: {:?}", client)
            }

            Notification::NewMessage(msg) => {
                writeln!(f, "Notifiaciton::NewMessage: {}", msg)
            }
        }
    }
}

fn run_server(rx: mpsc::Receiver<Notification>) -> Result<(), ()> {
    loop {
        let inner = rx.recv();
        match inner {
            Ok(kind) => match kind {
                Notification::ClientConnection(stream) => {
                    let addr = stream.peer_addr().unwrap();
                    println!(
                        "[SERVER] received `Notification::ClientConnection`:\n{:?}",
                        addr
                    );
                }

                Notification::ClientDisconnection(stream) => {
                    let addr = stream.peer_addr().unwrap();
                    println!(
                        "[SERVER] received `Notification::ClientDisconnection`:\n{:?}",
                        addr
                    );
                }

                Notification::NewMessage(msg) => {
                    println!("[SERVER] received `Notification::NewMessage`:\n{}", msg);
                }
            },

            Err(err) => {
                eprintln!("ERROR: failed to receive data from the sender: {}", err);
            }
        }
    }
    Ok(())
}

// TODO: stream may should be MutexGuard<> or Arc<Mutex<>>
fn run_client(stream: Arc<TcpStream>, tx: mpsc::Sender<Notification>) -> Result<(), ()> {
    tx.send(Notification::ClientConnection(stream.clone()))
        .map_err(|err| {
            eprintln!(
            "ERROR: failed to send `Notification::ClientConnection` message to the server: {:?}",
            err
        );
        })?;

    // let mut buf: Vec<u8> = Vec::new();
    // buf.resize(128, 0);
    let mut buf = [0; 1024];

    loop {
        let len = stream.as_ref().read(&mut buf).unwrap();

        if len == 0 {
            let _ = tx.send(Notification::ClientDisconnection(stream.clone())).map_err(|err| {
                eprintln!("ERROR: failed to send `Notification::ClientDisconnection` to the server: {:?}", err);
            });
            break;
        }

        let client_message = ClientMessage {
            author: Arc::new(stream.peer_addr().unwrap()),
            date: chrono::offset::Utc::now(),
            msg: truncate_to_nonzeros(&mut buf.clone().to_vec()).unwrap(),
        };

        let new_message = Notification::NewMessage(client_message);

        let _ = tx.send(new_message).map_err(|err| {
            eprintln!(
                "ERROR: failed to send `Notification::NewMessage` message to the server: {}",
                err
            );
        })?;
    }

    Ok(())
}

fn main() -> Result<(), ()> {
    if let Ok(connection) = TcpListener::bind(LOCALHOST) {
        println!("connection established to {}", LOCALHOST);

        let (tx, rx) = mpsc::channel::<Notification>();

        let _ = thread::spawn(|| {
            let _ = run_server(rx).map_err(|err| {
                eprintln!("ERROR: failed to establish connection to server: {:?}", err);
            });
        });

        for stream in connection.incoming() {
            match stream {
                Ok(stream) => {
                    let tx = tx.clone();

                    // let stream = Arc::new(Mutex::new(stream));
                    let stream = Arc::new(stream);

                    let _ = thread::spawn(move || {
                        // Mutex's lock() method returns MutexGuard,
                        // ensuring that only one thread can access the data at a time
                        // let mut stream = stream.lock().unwrap();

                        let _ = run_client(stream, tx).unwrap();
                    });
                }

                Err(err) => {
                    eprintln!("ERROR: failed to accept the stream: {:?}", err);
                    return Err(());
                }
            }
        }
    } else {
        eprintln!("ERROR: failed to establish connection to {}", LOCALHOST);
        return Err(());
    }

    Ok(())
}
