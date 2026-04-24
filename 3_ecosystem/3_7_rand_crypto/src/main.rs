use std::{fs, io};
use sha3::{Sha3_256, Digest};
use rand::prelude::*;
use rand::rng;
use std::path::Path;
use argon2::{self, Config};

fn select_rand_val<T>(slice: &[T]) -> Option<&T> {
    let mut rng = rng();
    slice.choose(&mut rng)
}

fn generate_password(length: usize, symbols_set: &str) -> String {
    let mut rng = rng();
    let chars: Vec<char> = symbols_set.trim().chars().collect();
    if chars.is_empty() {
        return String::new();
    }

    (0..length).map(|_| chars.choose(&mut rng).unwrap()).collect()
}

fn new_access_token() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz\
                             ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             0123456789";
    const TOKEN_LEN: usize = 64;
    let mut rng = rng();

    (0..TOKEN_LEN)
        .map(|_| *CHARSET.choose(&mut rng).unwrap() as char)
        .collect()
}

fn get_file_hash<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha3_256::new();

    let mut buffer = [0u8; 8192];
    loop {
        let count = io::Read::read(&mut file, &mut buffer)?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }

    let hash_bytes = hasher.finalize();

    Ok(hash_bytes.iter().map(|b| format!("{:02x}", b)).collect())
}

fn hash_password(password: &[u8]) -> String {
    let mut salt = [0u8; 16];
    rng().fill_bytes(&mut  salt);
    let cfg = Config::default();
    argon2::hash_encoded(password, &salt, &cfg).unwrap()
}

fn main() {
    // generate_password
    let length: usize = 10;
    let set = "f ";
    let password = generate_password(length, set);
    println!("{}", password);

    // select_rand_val
    let slice = vec![1, 2, 3, 4];
    let selected_val = select_rand_val(&slice);
    match selected_val {
        Some(val) => println!("{}", val),
        None => println!("No value"),
    }

    // new_access_token
    let token = new_access_token();
    println!("{}", token);

    // get_file_hash
    let path = "example.txt".to_string();
    match get_file_hash(path) {
        Ok(hash) => println!("SHA-3 Hash: {}", hash),
        Err(e) => eprintln!("Error reading file: {}", e),
    }

    // hash_password
    let password = "anything";
    println!("password: {}", password);
    let hashed = hash_password(password.as_bytes());
    println!("hashed password: {}", hashed);
    let matches = argon2::verify_encoded(&hashed, password.as_bytes()).unwrap();
    println!("matches: {}", matches);
}