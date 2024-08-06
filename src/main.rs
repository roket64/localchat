use localchat::server;
use localchat::test;

fn main() {
  let _ = server::run_server().unwrap();
  let _ = test::test_multithreading();
}