#![feature(step_by)]

extern crate blars;

use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write, Read, Seek, SeekFrom};
use std::env;
use std::str::from_utf8_unchecked;
use blars::util::*;
use blars::cmap::{ CollisionMap, Entry };
use std::collections::{BinaryHeap};
use std::borrow::Borrow;
use std::path::Path;
use std::sync::atomic::{AtomicUsize};
use std::sync::atomic::Ordering as AOrdering;

const FEATURE_WIDTH : usize = 24;
const ALPHABET_WIDTH : usize = 8;
const NGRAM_WIDTH : usize = 4;
const BLOCK_WIDTH : usize = 64;

const COLLISION_BITS : usize = 11;
const KEY_SIZE : usize = 4;

fn main() {
    let mut args = env::args();
    args.next();
    let datafile = args.next().unwrap();
    let inpath = Path::new(&datafile);
    println!("inpath={:?}", inpath);
    let seed : usize= args.next().unwrap().parse().unwrap();
    let scoresp = Path::new("scores.log");
    let mut ifile = File::open(&inpath).unwrap();
    let filesize = ifile.seek(SeekFrom::End(0)).unwrap();
    ifile.seek(SeekFrom::Start(0));
    let mut infile = BufReader::new(ifile);
    let mut sfile = BufWriter::new(File::create(&scoresp).unwrap());
    let mut genome = Vec::<u8>::with_capacity(1_000_000);
    let mut codons = Vec::<Entry<u32>>::with_capacity(1_000_000);
    let mut scores = BinaryHeap::<&Entry<u32>>::with_capacity(1_000_000);
    let mut buf : [u8; BLOCK_WIDTH] = [0; BLOCK_WIDTH];
    let mut i = 0usize;
    let projections = generate_normal_projection(ALPHABET_WIDTH, FEATURE_WIDTH, seed);
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

    println!("Generating Codons");
    //TODO need to make KEY_SIZE drive the type of int value created
    //then it will be fully generic
    for g in (0 .. genome.len() - KEY_SIZE).step_by(KEY_SIZE) {
        let val : u32 = slice_to_int(&genome[g .. g + KEY_SIZE]).unwrap();
        codons.push(Entry {id: g as u64, hash: val, score: 0.0, total: AtomicUsize::new(0), count: AtomicUsize::new(0)});
    }

    {
        let mut map = [CollisionMap::new(COLLISION_BITS, seed),
                       CollisionMap::new(COLLISION_BITS, seed + 1),
                       CollisionMap::new(COLLISION_BITS, seed + 2)];

        //This is annoying that I have to do this is a separate loop
        for c in codons.iter() {
            for i in map.iter_mut() {
                i.insert(&c);
            }
        }

        println!("Scoring codon distances");

        for i in map.iter() {
            i.score();
        }
    }

    println!("Sorting scored codons");
    for c in codons.iter_mut() {
        c.score = c.total.load(AOrdering::Relaxed) as f64 / c.count.load(AOrdering::Relaxed) as f64;
        scores.push(c);
    }

    let mut ifile = File::open(&inpath).unwrap();

    for i in scores.iter().take(30) {
        let mut buf = &mut[0u8; 100];
        ifile.seek(SeekFrom::Start(i.id - 50));
        ifile.read(buf);
        let line = unsafe { from_utf8_unchecked(buf) };
        writeln!(&mut sfile, "codon: {:?} -- offset: {} -- {}\n", i, i.id, line);
    }
}
