extern crate blars;

use std::old_io::{File, BufferedReader, BufferedWriter };
use std::env;
use blars::util::*;

static FEATURE_WIDTH : usize = 32;
static ALPHABET_WIDTH : usize = 8;
static WORD_WIDTH : usize = 4;

fn main() {
    let mut args = env::args();
    args.next();
    let inpath = Path::new(args.next().unwrap());
    println!("inpath={:?}", inpath);
    let scores = Path::new("scores.log");
    let counts = Path::new("counts.log");
    let mut infile = BufferedReader::new(File::open(&inpath));
    let mut sfile = BufferedWriter::new(File::create(&scores));
    let mut cfile = BufferedWriter::new(File::create(&counts));
    let projections = generate_projection_vectors(ALPHABET_WIDTH, FEATURE_WIDTH);

    println!("Generating genome");
    let genome : Vec<u16> = infile.lines().enumerate().map(|(i, line)| {
        if i % 100000 == 0 { println!("line #{}", i); }
        locality_hash_vector(
            &feature_hash_string(line.unwrap_or(String::from_str("heyoo")).as_slice(),
                                 WORD_WIDTH,
                                 FEATURE_WIDTH),
            ALPHABET_WIDTH,
            &projections)
    }).collect();


    println!("Generating codon for genome of size: {}", genome.len());
    let (codon, counts) = generate_codon(&genome, WORD_WIDTH);

    for (k, v) in counts.iter() {
        cfile.write_line(format!("key: {}, score: {}", k, v).as_slice()).unwrap();
    }

    println!("scoring codon counts of size: {}", counts.len());
    let scores = score_codon(&counts, WORD_WIDTH, genome.len(), true);

    for (k, v) in scores.iter() {
        sfile.write_line(format!("key: {}, score: {}", k, v).as_slice()).unwrap();
    }
}
