use rand::prelude::*;

const CHARSET_ALPHA_NUM: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const CHARSET_LOWER_ALPHA_NUM: &str = "abcdefghijklmnopqrstuvwxyz0123456789";
const CHARSET_ALPHA: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn random(length: usize, charset: &str) -> String {
    let distr = rand::distr::Uniform::new(0, charset.len()).unwrap();
    rand::rng()
        .sample_iter(distr)
        .take(length)
        .map(|i| charset.chars().nth(i).unwrap())
        .collect()
}

pub fn random_alpha_num(length: usize) -> String {
    random(length, CHARSET_ALPHA_NUM)
}

pub fn random_lower_alpha_num(length: usize) -> String {
    random(length, CHARSET_LOWER_ALPHA_NUM)
}

pub fn random_alpha(length: usize) -> String {
    random(length, CHARSET_ALPHA)
}
