use core::fmt;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc};
use std::thread;

const SAFE_MODE: bool = true;
const LOCALHOST: &str = "127.0.0.1:6974";

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    NewMessage(Vec<u8>),
}

impl fmt::Display for Notification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Notification::ClientConnection(_) => {
                writeln!(f, "Notification::ClientConnection")
            }
            Notification::ClientDisconnection(_) => {
                writeln!(f, "Notification::ClientDisconnection")
            }
            Notification::NewMessage(v) => {
                writeln!(f, "Notifiaciton::NewMessage: {:?}", v)
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
                        "[SERVER] received \'Notification::ClientConnection\': {:?}",
                        addr
                    );
                }
                Notification::ClientDisconnection(stream) => {
                    let addr = stream.peer_addr().unwrap();
                    println!(
                        "[SERVER] received \'Notification::ClientDisconnection\': {:?}",
                        addr
                    );
                }
                Notification::NewMessage(v) => {
                    println!("[SERVER] received \'Notification::NewMessage\': {:?}", v);
                }
            },
            Err(err) => {
                eprintln!("ERROR: failed to receive data from the sender: {}", err);
                // match kind {
                //     mpsc::TryRecvError::Empty => {
                //         eprintln!("ERROR: failed to receive data from the sender; channel is currently empty");
                //     }
                //     mpsc::TryRecvError::Disconnected => {
                //         eprintln!("ERROR: failed to receive data from the sender; the sender is disconnected");
                //     }
                // }
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
            "ERROR: failed to send \'Notification::ClientConnection\' message to the server: {:?}",
            err
        );
        })?;

    let mut buf: Vec<u8> = Vec::new();
    buf.resize(128, 0);

    loop {
        let len = stream.as_ref().read(&mut buf).unwrap();

        if len == 0 {
            let _ = tx.send(Notification::ClientDisconnection(stream.clone())).map_err(|err| {
                eprintln!("ERROR: failed to send \'Notification::ClientDisconnection\' to the server: {:?}", err);
            });
            break;
        }

        // handling message sending process
        let _ = tx
            .send(Notification::NewMessage(buf[0..len].to_vec()))
            .map_err(|err| {
                eprintln!(
                    "ERROR: failed to send \'Notification::NewMessage\' message to the server: {}",
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
