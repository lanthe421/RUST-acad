use crossbeam_channel::bounded;
use rand::RngExt;
use rayon::prelude::*;
use std::sync::Arc;
use std::thread;

const SIZE: usize = 4096;
const MATRIX_SIZE: usize = SIZE * SIZE;

#[derive(Clone)]
struct Matrix {
    data: Arc<Vec<u8>>,
}

impl Matrix {
    fn generate_matrix() -> Self {
        let mut data = vec![0u8; MATRIX_SIZE];
        rand::rng().fill(data.as_mut_slice());
        Self { 
            data: Arc::new(data)
        }
    }

    fn sum_matrix(&self) -> u64 {
        self.data
            .par_iter()
            .map(|&x| x as u64)
            .sum()
    }
}



fn main() {
    let (tx, rx) = bounded::<Matrix>(1);

    let producer = thread::Builder::new()
        .name("producer".to_string())
        .spawn(move || {
            loop {
                let matrix = Matrix::generate_matrix();
                if tx.send(matrix).is_err() {
                    break;
                }
            }
        })
        .expect("Failed to spawn producer thread");

    let consumers: Vec<_> = (0..2)
        .map(|id| {
            let rx = rx.clone();
            thread::Builder::new()
                .name(format!("consumer-{}", id))
                .spawn(move || {
                    while let Ok(matrix) = rx.recv() {
                        let sum = matrix.sum_matrix();
                        println!("consumer {}: sum is {}", id, sum);
                    }
                })
                .unwrap_or_else(|_| panic!("Failed to spawn consumer thread {}", id))
        })
        .collect();

    producer.join().unwrap();
    for (id, consumer) in consumers.into_iter().enumerate() {
        consumer.join().unwrap_or_else(|_| panic!("Consumer {} panicked", id));
    }
}
