use std::time::Instant;

fn cosine_similarity_precomputed_zip(a: &[f64], b: &[f64], norm_a: f64) -> f64 {
    let mut dot_product = 0.0f64;
    let mut norm_b_sq = 0.0f64;

    for (av, bv) in a.iter().zip(b.iter()) {
        dot_product += av * bv;
        norm_b_sq += bv * bv;
    }

    let norm_b = norm_b_sq.sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

fn cosine_similarity_fast(a: &[f64], b: &[f64], norm_a: f64, norm_b: f64) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }

    if a.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0f64;

    for (av, bv) in a.iter().zip(b.iter()) {
        dot_product += av * bv;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

fn main() {
    let a: Vec<f64> = (0..1536).map(|x| x as f64 * 0.1).collect();
    let b: Vec<f64> = (0..1536).map(|x| x as f64 * 0.2).collect();
    let norm_a = 1.0;
    let norm_b = 2.0;

    let iters = 1_000_000;

    let start = Instant::now();
    for _ in 0..iters {
        std::hint::black_box(cosine_similarity_precomputed_zip(&a, &b, norm_a));
    }
    let zip_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iters {
        std::hint::black_box(cosine_similarity_fast(&a, &b, norm_a, norm_b));
    }
    let fast_time = start.elapsed();

    println!("Zip precomputed time: {:?}", zip_time);
    println!("Fast time (both precomputed): {:?}", fast_time);
}
