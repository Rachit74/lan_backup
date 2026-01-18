use std::io::prelude::*;
use std::net::TcpStream;
use ssh2::Session;
use std::env;
use dotenv::dotenv;
fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenv().ok();
    let password = env::var("PASS").expect("Password must be set in .env");

    let tcp = TcpStream::connect("localhost:22")?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Authenticate with a username and password
    sess.userauth_password("rachit", &password)?;

    // Check if we are authenticated
    println!("Authentication successful!");

    // Open a channel and execute a command
    let mut channel = sess.channel_session()?;
    channel.exec("whoami")?; // Example command


    channel.wait_close()?;
    println!("Channel exit status: {}", channel.exit_status()?);

    Ok(())
}
