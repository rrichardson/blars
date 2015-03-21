extern crate blars;

use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write, Read};
use std::env;
use blars::util::*;

static FEATURE_WIDTH : usize = 16;
static ALPHABET_WIDTH : usize = 8;
static WORD_WIDTH : usize = 4;

fn main() {
    let mut args = env::args();
    args.next();
    let inpath = Path::new(args.next().unwrap());
    println!("inpath={:?}", inpath);
    let scores = Path::new("scores.log");
    let counts = Path::new("counts.log");
    let projs =  Path::new("projections.log");
    let mut infile = BufReader::new(File::open(&inpath).unwrap());
    let mut sfile = BufWriter::new(File::create(&scores).unwrap());
    let mut cfile = BufWriter::new(File::create(&counts).unwrap());
    let mut pfile = BufWriter::new(File::create(&projs).unwrap());
    let projections = generate_projection_vectors(ALPHABET_WIDTH, FEATURE_WIDTH);
    let mut lines = Vec::<usize>::with_capacity(1_000_000);
    let mut genome = Vec::<u16>::with_capacity(1_000_000);
    let mut buf = String::with_capacity(512);
    let mut i = 0usize;

    println!("Generating genome");
    lines.push(0);
    loop {
        if i % 100_00 == 0 { println!("line #{}", i); }
        i += 1;
        buf.clear();
        match infile.read_line(&mut buf) {
            Ok(0) => { break },
            Ok(sz) => {
                lines.push(sz);
                genome.push(
                    locality_hash_vector(
                        &feature_hash_string(
                            buf.as_slice(),
                            WORD_WIDTH,
                            FEATURE_WIDTH),
                        ALPHABET_WIDTH,
                        &projections));
            },
            Err(e) => {println!("Error: {}", e)}
        }
    }
    for p in projections.iter() {
        writeln!(&mut pfile, "{:?}", p).unwrap();
    }

    println!("Generating codon for genome of size: {}", genome.len());
    let (codon, counts) = generate_codon(&genome, WORD_WIDTH);

    for (k, v) in counts.iter() {
        writeln!(&mut cfile, "key: {}, score: {}", k, v).unwrap();
    }

    println!("scoring codon counts of size: {}", counts.len());
    let scores = score_codon(&counts, WORD_WIDTH, genome.len(), true);

    for (k, v) in scores.iter() {
       writeln!(&mut sfile, "key: {}, score: {}", k, v).unwrap();
    }


}
