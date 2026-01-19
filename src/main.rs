use ssh2::Session;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use walkdir::WalkDir;
use std::env;
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
    session.handshake()?;

    session.userauth_password(username, &password)?;
    assert!(session.authenticated());

    let sftp = session.sftp()?;

    for entry in WalkDir::new(local_root) {
        let entry = entry?;
        let path = entry.path();

        let relative = path.strip_prefix(local_root)?;
        let remote_path = remote_root.join(relative);

        if path.is_dir() {
            let _ = sftp.mkdir(&remote_path, 0o755);
        } else if path.is_file() {

            let local_size = path.metadata()?.len();

            // Try to stat remote file
            if let Ok(remote_meta) = sftp.stat(&remote_path) {
                if remote_meta.size == Some(local_size) {
                    println!("Skipping (unchanged): {:?}", relative);
                    continue;
                }
            }

            println!("Uploading: {:?}", relative);

            let mut local_file = File::open(path)?;
            let mut remote_file = sftp.create(&remote_path)?;

            let mut buffer = [0u8; 8192];
            loop {
                let n = local_file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                remote_file.write_all(&buffer[..n])?;
            }
        }
    }


    println!("Transfer complete.");
    Ok(())
}
