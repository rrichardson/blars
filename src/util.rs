use std::vec::Vec;
use std::iter::AdditiveIterator;
use std::iter::repeat;
use std::num::Float;
use xxhash::hash;
use rand::{thread_rng, ThreadRng, Rng};
use rand::distributions::normal::StandardNormal;
use unicode::char::from_u32;

/// the maximum number of bits allowable in a unicode char
static MAX_WIDTH : usize = 20;

///
/// Calculates simple moving average for an array of ints
/// beginning at 1 item up to make window
///
pub fn moving_average(vals: &[usize], window: usize, ) -> Vec<f64> {
    let mut result = Vec::with_capacity(vals.len());
    let mut total : usize = 0;
    let mut backidx : i64;

    for (i,v) in vals.iter().enumerate() {
        total += *v;
        backidx = i as i64 - window as i64;
        if backidx >= 0 {
            total -= vals[backidx as usize];
            result.push(total as f64 / window as f64);
        } else {
            result.push(total as f64 / (i + 1) as f64);
        }
    }
    result
}

fn magnitude(vals: &[f64]) -> f64 {
    let t = vals.iter().map(|x| x * x).sum();
    t.sqrt()
}

pub fn normalize(vals: &[f64]) -> Vec<f64> {
    let mag = magnitude(vals);
    vals.iter().map(|x| *x / mag).collect()
}

pub fn dot_product(a: &[f64], b: &[f64]) -> f64 {

    a.iter().zip(b.iter()).fold(0.0, |x, (a, b)| x + (a * b))
}

pub fn feature_hash_string(s : &str, window: usize, width: usize) -> Vec<f64> {
    if width > MAX_WIDTH { panic!("width cannot exceed {}", MAX_WIDTH); }

    let mut v : Vec<f64> = repeat(0.0).take(width).collect();

    for x in (0 .. (s.len() - window)) {
        let key = (hash(&s[x .. x + window]) % width as u64) as usize;
        v[key] += 1.0;
    }

    v
}

///
/// Produce a 16 bit integer whose bits are set by
/// the result of the dot product of the provided feature hash
/// with the random projection vectors
///
pub fn locality_hash_vector(v : Vec<f64>, width : usize, proj_vecs: Vec<Vec<f64>>) -> char {
    if width > MAX_WIDTH { panic!("width cannot exceed {}", MAX_WIDTH); }

    let mut r = 0u32;
    for i in (0 .. width) {
        if dot_product(proj_vecs[i].as_slice(), v.as_slice()) > 0.0 {
            r |= 1 << (width - 1 - i)
        }
    }

    //validating the number of bits *should* guarantee that this unwrap is safe
    from_u32(r).unwrap()
}

///
/// Create a vector of vector normals in a random distribution
/// This function mallocs a lot, but it should only be run at initialization time
///
pub fn gen_projection_vectors(alphabet_width: usize, feature_width: usize) -> Vec<Vec<f64>> {
    (0 .. alphabet_width).map(|_| {
        let v : Vec<f64> = (0 .. feature_width).map(|_| thread_rng().gen::<StandardNormal>().0 ).collect();
        normalize(v.as_slice())
    }).collect()
}


#[test]
fn it_works() {
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 8, 7, 6, 5, 4, 3, 2, 1];
    println!("{:?}", moving_average(&arr, 5));
    println!("{:?}", moving_average(&arr, 5));
}
