use ssh2::Session;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use::std::env;
use dotenv::dotenv;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // load env
    dotenv().ok();

    // configs
    let local_root = Path::new("/home/rachit/tool_test/src_dir");
    let remote_root = Path::new("/home/rachit/tool_test/des_dir");

    let host = "192.168.1.11:22";
    let username = "rachit";
    let password = env::var("PASS").expect("Password not found in env");

    // tcp connection
    let tcp = TcpStream::connect(host)?;

    // ssh session
    let mut session = Session::new()?;

    session.set_tcp_stream(tcp);
    session.handshake();

    session.userauth_password(username, &password)?;

    Ok(())



}