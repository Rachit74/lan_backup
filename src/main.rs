use ssh2::Session;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use walkdir::WalkDir;
use std::env;
use dotenv::dotenv;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
struct FileRecord {
    hash: String,
    size: u64,
}

type Index = HashMap<String, FileRecord>;

// use buffer to read file in chunks and hash rather than reading whole file into memory
fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())

}

fn load_index(path: &Path) -> Index {
    if let Ok(data) = std::fs::read_to_string(path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

fn save_index(path: &Path, index: &Index) -> std::io::Result<()> {
    let data = serde_json::to_string_pretty(index).unwrap();
    std::fs::write(path, data)
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let index_path = Path::new("backup_index.json");
    let mut index = load_index(index_path);


    // load env
    dotenv().ok();

    // configs
    let local_root = Path::new("/home/rachit/dev/projects/project-inventory");
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
            let relative_str = relative.to_string_lossy().to_string();

            // Compute hash
            let local_hash = hash_file(path)?;

            // Check index
            if let Some(record) = index.get(&relative_str) {
                if record.hash == local_hash {
                    println!("Skipping (unchanged): {:?}", relative);
                    continue;
                }
            }
            // Update index
            index.insert(relative_str,FileRecord {hash: local_hash, size: local_size,},);

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

    save_index(index_path, &index)?;
    println!("Transfer complete.");
    Ok(())
}
