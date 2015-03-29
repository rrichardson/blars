extern crate blars;
extern crate stats;

use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write, Read, Seek, SeekFrom};
use std::env;
use std::str::from_utf8_unchecked;
use blars::util::*;
use stats::OnlineStats;
use blars::cmap::{ CollisionMap, Entry };
use std::collections::{BinaryHeap};


const FEATURE_WIDTH : usize = 24;
const ALPHABET_WIDTH : usize = 8;
const NGRAM_WIDTH : usize = 4;
const BLOCK_WIDTH : usize = 64;

const COLLISION_BITS : usize = 11;
const KEY_SIZE : usize = 4;

fn main() {
    let mut args = env::args();
    args.next();
    let inpath = Path::new(args.next().unwrap());
    println!("inpath={:?}", inpath);
    let seed : usize= args.next().unwrap().parse().unwrap();
    let scores = Path::new("scores.log");
    let mut ifile = File::open(&inpath).unwrap();
    let filesize = ifile.seek(SeekFrom::End(0)).unwrap();
    ifile.seek(SeekFrom::Start(0));
    let mut infile = BufReader::new(ifile);
    let mut sfile = BufWriter::new(File::create(&scores).unwrap());
    let mut genome = Vec::<u8>::with_capacity(1_000_000);
    let mut buf : [u8; BLOCK_WIDTH] = [0; BLOCK_WIDTH];
    let mut i = 0usize;
    let projections = generate_normal_projection(ALPHABET_WIDTH, FEATURE_WIDTH, seed);
    let mut map = [CollisionMap::new(COLLISION_BITS, seed),
        CollisionMap::new(COLLISION_BITS, seed + 1),
        CollisionMap::new(COLLISION_BITS, seed + 2)];
    //let projections = generate_binary_projection(ALPHABET_WIDTH, FEATURE_WIDTH, seed);

    println!("Generating genome");
    loop {
        if i % 100_000 == 0 { println!("offset #{} out of {}", i * BLOCK_WIDTH, filesize); }
        i += 1;
        match infile.read(&mut buf) {
            Ok(0) => { break },
            Ok(sz) => {
                genome.push(
                    locality_hash_vector(
                        &feature_hash_string( &buf, NGRAM_WIDTH, FEATURE_WIDTH),
                        ALPHABET_WIDTH,
                        &projections));
            },
            Err(e) => {println!("Error: {}", e)}
        }
    }

    println!("Generating collision map");
    //TODO need to make KEY_SIZE drive the type of int value created
    //then it will be fully generic
    for g in (0 .. genome.len() - KEY_SIZE).step_by(KEY_SIZE) {
        for i in map.iter_mut() {
            let val : u32 = slice_to_int(&genome[g .. g + KEY_SIZE]).unwrap();
            i.insert(val, g as u64)
        }
    }

    println!("Scoring genome distances");
    let mut scores = BinaryHeap::<Entry>::with_capacity(1_000_000);
    for i in map.iter() {
        i.score(&mut scores);
    }

    let mut ifile = File::open(&inpath).unwrap();

    for i in scores.iter().take(30) {
        let mut buf = &mut[0u8; 100];
        ifile.seek(SeekFrom::Start(i.0-50));
        ifile.read(buf);
        let line = unsafe { from_utf8_unchecked(buf) };
        writeln!(&mut sfile, "score: {} -- offset: {} -- {}\n", i.1, i.0, line);
    }
}
