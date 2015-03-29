
use std::vec::Vec;
use std::collections::{VecDeque, BinaryHeap};
use std::collections::vec_deque::Iter as DIter;
use std::slice::Iter as VIter;
use std::num::{Int, NumCast};
use std::intrinsics::{ctpop8, ctpop16, ctpop32, ctpop64};
use rand::{StdRng, Rng, SeedableRng};
use std::mem;
use std::fmt;
use std::cmp::{Ord, PartialOrd, Ordering };

/// Collision map
/// Specialized structure which holds lists of candidate close items
///


#[derive(Debug)]
pub struct CollisionMap<I : Int + fmt::Debug> {
    cells: Vec<VecDeque<(I, u64)>>,
    mask: I
}

impl<I> CollisionMap<I> where I : Int + fmt::Debug {
    pub fn new(num_bits : usize, seed : usize) -> CollisionMap<I>{
        let size : usize = 2.pow(num_bits as u32);
        CollisionMap {
            cells: (0 .. size).map(|_| VecDeque::with_capacity(30)).collect(),
            mask : gen_mask(num_bits, seed).unwrap()
        }
    }

    pub fn insert(&mut self, val: I, id: u64) {
        self.cells[collapse(val, self.mask)].push_back((val, id))
    }

    pub fn score(&self, scores: &mut BinaryHeap<Entry>) -> usize {

        let mut ttls : Vec<u32> = (0 .. 1_000_000).map(|_| 0).collect();
        let mut count = 0;
        let mut deq_idx = 0;
        for i in self.cells.iter() {
            let mut slf = 0;
            let sz = i.len();
            if sz < 1 { continue; }
            for d in i.iter().take(sz -1) {
                count += 1;
                if count % 100_000 == 0 { println!("{} * 4 scored", count); }
                let mut oth = slf + 1;
                for e in i.iter().skip(slf+1) {
                    let dst = distance(d.0, e.0);
                    ttls[slf] = ttls[slf] + dst;
                    ttls[oth] = ttls[oth] + dst;
                    oth += 1
                }
                scores.push(Entry (d.1, ttls[slf] as f64 / sz as f64));
                ttls[slf] = 0;
                slf += 1;
            }
            scores.push(Entry (i.back().unwrap().1, ttls[slf] as f64 / sz as f64));
            ttls[slf] = 0;
            deq_idx += 1;
        }

        count
    }
}

#[derive(Debug)]
pub struct Entry (pub u64, pub f64);

impl PartialEq for Entry {
    fn eq(&self,  other: &Entry) -> bool {
        self.1 == other.1
    }
}

impl Eq for Entry {
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Entry) -> Option<Ordering> {
        if self.1 > other.1 {
            Some(Ordering::Greater)
        } else if self.1 < other.1 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Entry) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}


// this is a really dumb generator with no actual upper bound on execution
// time, but it'll probably end up being faster than a sampling algo
fn gen_mask<I : Int>(num_bits : usize, seed : usize) -> Option<I> {
    let mut rng = StdRng::from_seed(&[seed]);
    if (num_bits / 8) > mem::size_of::<I>() {
        return None;
    }

    loop {
        let item : I = NumCast::from(rng.next_u64() & (I::max_value().to_u64().unwrap()) ).unwrap();
        if item.count_ones() == num_bits as u32 {
            return Some(item)
        }
    }

    return None;
}


// a perfect hash algo of sorts to place the value
#[inline]
fn collapse<I : Int>(num: I, mask: I) -> usize {
    let num_bits = mem::size_of::<I>() * 8;
    let mut result = 0;
    let mut i : usize = 0;
    let mut idx : usize = 0;
    while i < num_bits {
        if mask >> i & I::one() == I::one() {
            if num >> i & I::one() == I::one() {
                result |= 1 << idx;
            }
            idx += 1;
        }
        i += 1;
    }

    result
}

#[inline]
fn distance<I : Int>(a : I, b : I) -> u32 {
    (a ^ b).count_ones()
}
/*
impl<I> fmt::Debug for CollisionMap<I> where I : Int {
    fn fmt(&self, f : &mut fmt::Formatter)  -> Result<(), fmt::Error> {
        try!(writeln!(f, "CV [ mask: {:b}", self.mask.to_u64().unwrap()));
        try!(writeln!(f, "Vecs ["));
        for i in self.cells.iter() {
            try!(writeln!(f, "{:?}", *i))
        }
        writeln!(f, "]]")
    }
}*/
