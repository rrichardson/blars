use std::vec::Vec;
use std::iter::{ repeat, Iterator, Map};
use std::collections::hash_map::{HashMap, Entry};
use xxhash::hash;
use rand::{StdRng, Rng, SeedableRng};
use rand::distributions::normal::StandardNormal;
use std::string::{String};
use std::iter::MinMaxResult::{MinMax};
use num::PrimInt;

/// the maximum number of bits allowable in a utf16 char
static MAX_WIDTH : usize = 8;
///
/// Calculates simple moving average for an array of ints
/// beginning at 1 item up to window size
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
    let t : f64 = vals.iter().map(|x| x * x).sum();
    t.sqrt()
}

pub fn normalize(vals: &[f64]) -> Vec<f64> {
    let mag = magnitude(vals);
    vals.iter().map(|x| *x / mag).collect()
}

pub fn dot_product(a: &[f64], b: &[f64]) -> f64 {

    a.iter().zip(b.iter()).fold(0.0, |x, (a, b)| x + (a * b))
}

pub fn feature_hash_string(s : &[u8], window: usize, width: usize) -> Option<Vec<f64>> {

    let mut v : Vec<u32> = repeat(0).take(width).collect();
    if window > s.len() { return None; }

    for x in (0 .. (s.len() - window)) {
        let key = (hash(&s[x .. x + window]) % width as u64) as usize;
        v[key] += 1;
    }
    //center the vector
    if let MinMax(mn, mx) = v.iter().min_max() {
        let cntr = ((*mx - *mn) as f64 / 2.0) + *mn as f64;
        Some(v.iter().map(|x| *x as f64 - cntr ).collect())
    }
    else {
        None
    }
}

///
/// Produce an 8 bit integer whose bits are set by
/// the result of the dot product of the provided feature hash
/// with the random projection vectors
///
pub fn locality_hash_vector(invec : &Option<Vec<f64>>, width : usize, proj_vecs: &Vec<Vec<f64>>) -> u8 {
    if width > MAX_WIDTH { panic!("width cannot exceed {}", MAX_WIDTH); }

    let mut r = 0u8;
    if let Some(ref v) = *invec {
        for i in (0 .. width) {
            if dot_product(proj_vecs[i].as_slice(), v.as_slice()) > 0.0 {
                r |= 1 << (width - 1 - i)
            }
        }
        r
    } else {
        0
    }
}

///
/// Create a vector of vector normals in a random distribution
/// This function mallocs a lot, but it should only be run at initialization time
///
pub fn generate_normal_projection(alphabet_width: usize, feature_width: usize, seed: usize) -> Vec<Vec<f64>> {
    let sd = [seed];
    let mut rng = StdRng::from_seed(&sd);
    (0 .. alphabet_width).map(|_| {
        let v : Vec<f64> = (0 .. feature_width).map(|_| rng.gen::<StandardNormal>().0 ).collect();
        normalize(v.as_slice())
    }).collect()
}

pub fn generate_binary_projection(alphabet_width: usize, feature_width: usize, seed: usize) -> Vec<Vec<f64>> {
    let sd = [seed];
    let mut rng = StdRng::from_seed(&sd);
    let choices = [-1.0, 1.0];
    (0 .. alphabet_width).map(|_| {
        (0 .. feature_width).map(|_| *rng.choose(&choices).unwrap() ).collect()
    }).collect()
}

pub fn generate_codon(genome: &Vec<u8>, width: usize) -> (Vec<Vec<u8>>, HashMap<Vec<u8>, usize>) {
    let num_keys = genome.len() - width + 1;
    let mut counts = HashMap::<Vec<u8>, usize>::with_capacity(num_keys);
    let mut codons = Vec::<Vec<u8>>::with_capacity(num_keys);

    for i in (0 .. num_keys) {
        let key = genome.as_slice()[i .. i + width].to_vec();
        match counts.entry(key.clone()) {
            Entry::Vacant(view) => {
                view.insert(1);
            },
            Entry::Occupied(mut view) => {
                let v = view.get_mut();
                *v += 1;
            }
        }
        codons.push(key);
    }
    (codons, counts)
}

pub fn score_codon(counts: &HashMap<Vec<u8>, usize>,
                   codon_width: usize,
                   genome_length: usize,
                   normalize: bool) -> HashMap<Vec<u8>, f64> {

    let mut scores = HashMap::<Vec<u8>, f64>::with_capacity(counts.len());
    let mut result = HashMap::<Vec<u8>, f64>::with_capacity(counts.len());
    let mut min = 1000000.0;
    let mut max = -1000000.0;

    counts.iter().inspect(| &(k, v)| {
        let score = -(*v as f64 / (genome_length - codon_width + 1) as f64).log2();
        scores.insert((*k).clone(), score);
        if min > score { min = score; }
        if max < score { max = score; }
    }).count();

    if normalize {
        scores.iter().inspect(| &(k,v)| { result.insert((*k).clone(), (*v - min) / max); }).count();
    } else {
        scores.iter().inspect(| &(k,v)| { result.insert((*k).clone(), *v - min); } ).count();
    }
    result
}

#[inline]
pub fn slice_to_int<I : PrimInt>(s : &[u8]) -> Option<I> {
    use std::raw::Slice;
    use std::mem;
    if s.len() < mem::size_of::<I>() { return None }

    let res : I = unsafe {
        let tmp : Slice<u8> = mem::transmute(s);
        *(tmp.data as *const I)
    };

    Some(res)
}

#[test]
fn it_works() {
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 8, 7, 6, 5, 4, 3, 2, 1];
    println!("{:?}", moving_average(&arr, 5));
    println!("{:?}", moving_average(&arr, 5));
}
