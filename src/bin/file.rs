extern crate blars;

use std::old_io::BufferedReader;
use std::old_io::File;
use std::env;
use blars::util::*;

static ALPHABET_WIDTH : usize = 5;
static FEATURE_WIDTH : usize = 30;
static VECTOR_WIDTH : usize = 16;

fn main() {
    let mut args = env::args();
    args.next();
    let path = Path::new(args.next().unwrap());
    let mut file = BufferedReader::new(File::open(&path));

    let projections = generate_projection_vectors(ALPHABET_WIDTH, FEATURE_WIDTH);

    let genome = file.lines().map(|line| {
        locality_hash_vector(feature_hash_string(line.unwrap().as_slice(),
                                                 VECTOR_WIDTH,
                                                 FEATURE_WIDTH),
                             ALPHABET_WIDTH,
                             &projections)
    }).collect();

    let (codon, counts) = generate_codon(&genome, VECTOR_WIDTH);

    let scores = score_codon(&counts, VECTOR_WIDTH, genome.len(), true);

}
