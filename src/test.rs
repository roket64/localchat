use std::io::Write;
use std::net;
use std::thread;

pub fn test_multithreading() {
    let mut handles = vec![];
    for i in 0..5 {
        handles.push(thread::spawn(move || {
            let mut client = net::TcpStream::connect("127.0.0.1:8080").unwrap();
            client
                .write_all(format!("test message from client {}", i).as_bytes())
                .unwrap();

            client.shutdown(net::Shutdown::Write).unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
