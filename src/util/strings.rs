use rand::Rng;

pub fn random_alpha_num(length: usize) -> String {
    const CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    rand::thread_rng()
        .sample_iter(&rand::distributions::Uniform::from(0..CHARSET.len()))
        .take(length)
        .map(|i| CHARSET.chars().nth(i).unwrap())
        .collect()
}
