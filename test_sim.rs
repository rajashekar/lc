fn main() {
    let q = vec![0.85, 0.15, 0.0, 0.0, 0.0];
    let q_norm = (q.iter().map(|x| x * x).sum::<f64>()).sqrt();

    let vecs = vec![
        ("Artificial intelligence research", vec![0.9, 0.1, 0.0, 0.0, 0.0]),
        ("Machine learning algorithms", vec![0.8, 0.2, 0.0, 0.0, 0.0]),
        ("Web development with JavaScript", vec![0.0, 0.0, 0.9, 0.1, 0.0]),
        ("Database design principles", vec![0.0, 0.0, 0.1, 0.9, 0.0]),
        ("Cooking Italian cuisine", vec![0.0, 0.0, 0.0, 0.0, 1.0]),
    ];

    for (t, v) in vecs {
        let dot: f64 = q.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
        let v_norm = (v.iter().map(|x| x * x).sum::<f64>()).sqrt();
        let sim = dot / (q_norm * v_norm);
        println!("{}: {}", t, sim);
    }
}
