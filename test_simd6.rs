use std::time::Instant;

fn cosine_similarity_simd(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }

    if a.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0f64;
    let mut norm_a_sq = 0.0f64;
    let mut norm_b_sq = 0.0f64;

    let chunk_size = 4;
    let chunks = a.len() / chunk_size;

    for i in 0..chunks {
        let start = i * chunk_size;
        let end = start + chunk_size;

        for j in start..end {
            let av = a[j];
            let bv = b[j];
            dot_product += av * bv;
            norm_a_sq += av * av;
            norm_b_sq += bv * bv;
        }
    }

    for i in (chunks * chunk_size)..a.len() {
        let av = a[i];
        let bv = b[i];
        dot_product += av * bv;
        norm_a_sq += av * av;
        norm_b_sq += bv * bv;
    }

    let norm_a = norm_a_sq.sqrt();
    let norm_b = norm_b_sq.sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

fn cosine_similarity_fast_zip(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }

    if a.is_empty() {
        return 0.0;
    }

    let (dot, a_sq, b_sq) = a.iter().zip(b.iter()).fold(
        (0.0f64, 0.0f64, 0.0f64),
        |(dot, a_sq, b_sq), (&av, &bv)| {
            (dot + av * bv, a_sq + av * av, b_sq + bv * bv)
        },
    );

    let norm_a = a_sq.sqrt();
    let norm_b = b_sq.sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

fn main() {
    let a: Vec<f64> = (0..1536).map(|x| x as f64 * 0.1).collect();
    let b: Vec<f64> = (0..1536).map(|x| x as f64 * 0.2).collect();

    let iters = 1_000_000;

    let start = Instant::now();
    for _ in 0..iters {
        std::hint::black_box(cosine_similarity_simd(&a, &b));
    }
    let manual_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iters {
        std::hint::black_box(cosine_similarity_fast_zip(&a, &b));
    }
    let zip_time = start.elapsed();

    println!("Manual time: {:?}", manual_time);
    println!("Fast zip time: {:?}", zip_time);
}
