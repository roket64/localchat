use core::fmt;
use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};
use std::{str, thread};

use chrono;
use threadpool::ThreadPool;

const SAFE_MODE: bool = true;
const LOCALHOST: &str = "127.0.0.1:8080";

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
    msg: String,
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

fn run_server(rx: Receiver<Notification>) -> Result<(), ()> {
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

fn run_client(stream: Arc<TcpStream>, tx: Sender<Notification>) -> Result<(), ()> {
    let _ = tx
        .send(Notification::ClientConnection(stream.clone()))
        .unwrap();

    let mut buf = [0; 1024];

    loop {
        // manipulating stream
        match stream.as_ref().read(&mut buf) {
            Ok(0) => {
                let _ = tx
                    .send(Notification::ClientConnection(stream.clone()))
                    .unwrap();
                break;
            }

            Ok(n) => {
                let client_message = ClientMessage {
                    author: Arc::new(stream.peer_addr().unwrap()),
                    date: chrono::offset::Utc::now(),
                    msg: String::from_utf8_lossy(&buf[..n]).to_string(),
                };

                let _ = tx.send(Notification::NewMessage(client_message)).unwrap();
            }

            Err(err) => {
                eprintln!("ERROR: failed to read stream: {:?}", err);
                return Err(());
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), ()> {
    if let Ok(connection) = TcpListener::bind(LOCALHOST) {
        println!("connection established to {}", LOCALHOST);

        let (tx, rx) = channel::<Notification>();

        // all client thrad must reference this
        let tx = Arc::new(Mutex::new(tx));

        let _ = thread::spawn(|| {
            let _ = run_server(rx).unwrap();
        });

        let pool = ThreadPool::new(4);

        for (_, stream) in connection.incoming().enumerate() {
            match stream {
                Ok(stream) => {
                    // TODO: should stream be wrapped like this?
                    // let stream = Arc::new(Mutex::new(stream));

                    let stream = Arc::new(stream);

                    let tx = Arc::clone(&tx);
                    let _ = pool.execute(move || {
                        let locked = tx.lock().unwrap().clone();
                        let _ = run_client(stream, locked);
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
