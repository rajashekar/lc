fn main() {
    let mut similarities: Vec<(&str, f64)> = vec![
        ("Cooking Italian cuisine", 0.0),
        ("Database design principles", 0.0),
        ("Machine learning algorithms", 0.9975295184354295),
        ("Web development with JavaScript", 0.0),
        ("Artificial intelligence research", 0.9979517409161514),
    ];
    let limit = 3;
    if limit < similarities.len() {
        let (_, _, _) = similarities.select_nth_unstable_by(limit, |a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        similarities[..limit]
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(limit);
    }
    for s in similarities {
        println!("{:?}", s);
    }
}
