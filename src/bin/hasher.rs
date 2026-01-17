
use sha2::{Sha256, Digest};


fn main() {
    let result = Sha256::digest(b"file_name.txt");
    println!("{:x}", result);
}