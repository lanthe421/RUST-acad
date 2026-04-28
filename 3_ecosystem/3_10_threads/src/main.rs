use crossbeam_channel::bounded;
use rand::RngExt;
use rayon::prelude::*;
use std::thread;

const SIZE: usize = 4096;

type Matrix = Vec<u8>;

fn generate_matrix() -> Matrix {
    let mut matrix = vec![0u8; SIZE * SIZE];
    rand::rng().fill(matrix.as_mut_slice());
    matrix
}

fn sum_matrix(matrix: &Matrix) -> u64 {
    matrix.par_iter().map(|&x| x as u64).sum()
}

fn main() {
    let (tx, rx) = bounded::<Matrix>(1);

    let producer = thread::spawn(move || {
        loop {
            let matrix = generate_matrix();
            if tx.send(matrix).is_err() {
                break;
            }
        }
    });

    let consumers: Vec<_> = (0..6)
        .map(|id| {
            let rx = rx.clone();
            thread::spawn(move || {
                while let Ok(matrix) = rx.recv() {
                    let sum = sum_matrix(&matrix);
                    println!("consumer {}: sum is {}", id, sum);
                }
            })
        })
        .collect();

    producer.join().unwrap();
    for c in consumers {
        c.join().unwrap();
    }
}
