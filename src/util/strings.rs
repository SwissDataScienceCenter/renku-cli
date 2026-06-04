use rand::Rng;

const CHARSET_ALPHA_NUM: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

const CHARSET_ALPHA: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn random(length: usize, charset: &str) -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Uniform::from(0..charset.len()))
        .take(length)
        .map(|i| charset.chars().nth(i).unwrap())
        .collect()
}

pub fn random_alpha_num(length: usize) -> String {
    random(length, CHARSET_ALPHA_NUM)
}

pub fn random_alpha(length: usize) -> String {
    random(length, CHARSET_ALPHA)
}
