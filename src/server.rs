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
    ClientConnection,
    ClientDisconnection,
    NewMessage(Vec<u8>),
}

impl fmt::Display for Notification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Notification::ClientConnection => {
                writeln!(f, "Notification::ClientConnection")
            }
            Notification::ClientDisconnection => {
                writeln!(f, "Notification::ClientDisconnection")
            }
            Notification::NewMessage(v) => {
                writeln!(f, "Notifiaciton::NewMessage: {:?}", v)
            }
        }
    }
}

fn server(rx: mpsc::Receiver<Notification>) -> Result<(), ()> {
    loop {
        match rx.recv() {
            Ok(msg) => {
                println!("{}", msg);
            }
            Err(err) => {
                eprintln!("ERROR: failed to receive the data from sender: {:?}", err);
            }
        }
    }
    Ok(())
}

// TODO: stream may should be MutexGuard<> or Arc<Mutex<>>
fn client(stream: Arc<TcpStream>, tx: mpsc::Sender<Notification>) -> Result<(), ()> {
    tx.send(Notification::ClientConnection).map_err(|err| {
        eprintln!(
            "ERROR: failed to send \"Notification::ClientConnection\" message to the server: {:?}",
            err
        );
    })?;

    let mut buf: Vec<u8> = Vec::new();
    buf.resize(128, 0);

    loop {
        let len = stream.as_ref().read(&mut buf).unwrap();

        if len == 0 {
            let _ = tx.send(Notification::ClientDisconnection).map_err(|err| {
                eprintln!("ERROR: failed to send \"Notification::ClientDisconnection\" to the server: {:?}", err);
            });
            break;
        }

        // handling message sending process
        let _ = tx
            .send(Notification::NewMessage(buf[0..len].to_vec()))
            .map_err(|err| {
                eprintln!(
                    "ERROR: failed to send \"Notification::NewMessage\" message to the server: {}",
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
            let _ = server(rx).map_err(|err| {
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

                        let _ = client(stream, tx).unwrap();
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
