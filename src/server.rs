use core::fmt;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
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

enum Notification {
    ClientConnection,
    ClientDisconnection,
    NewMessage(Vec<u8>),
}

fn server(_msg: mpsc::Receiver<Notification>) -> Result<(), ()> {
    todo!()
}

fn client(mut stream: MutexGuard<TcpStream>, tx: mpsc::Sender<Notification>) -> Result<(), ()> {
    tx.send(Notification::ClientConnection).map_err(|err| {
        eprintln!("ERROR: failed to send message to the server: {:?}", err);
    })?;

    let mut buf: Vec<u8> = Vec::new();
    buf.resize(128, 0);

    loop {
        let len = stream.read(&mut buf).map_err(|_| {
            let _ = tx.send(Notification::ClientDisconnection);
        })?;

        let _ = tx
            .send(Notification::NewMessage(buf[0..len].to_vec()))
            .map_err(|err| {
                eprintln!("ERROR: failed to send message to the server: {}", err);
            })?;
    }

    Ok(())
}

fn main() -> Result<(), ()> {
    if let Ok(connection) = TcpListener::bind(LOCALHOST) {
        println!("connection established to {}", LOCALHOST);

        let (tx, rx) = mpsc::channel::<Notification>();

        thread::spawn(|| {
            let _ = server(rx).map_err(|err| {
                eprintln!("ERROR: failed to establish connection to server: {:?}", err);
            });
        });

        for stream in connection.incoming() {
            match stream {
                Ok(stream) => {
                    let tx = tx.clone();

                    let stream = Arc::new(Mutex::new(stream));

                    thread::spawn(move || {
                        let mut stream = stream.lock().unwrap();
                        let _ = client(stream, tx).map_err(|err|{
                            eprintln!("ERROR: failed to {}", err);
                        });
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
