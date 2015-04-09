
use std::vec::Vec;
use std::collections::{VecDeque, BinaryHeap};
use std::collections::vec_deque::Iter as DIter;
use std::slice::Iter as VIter;
use num::{PrimInt, NumCast, Num};
use std::intrinsics::{ctpop8, ctpop16, ctpop32, ctpop64};
use rand::{StdRng, Rng, SeedableRng};
use std::mem;
use std::fmt;
use std::cmp::{Ord, PartialOrd};
use std::cmp::Ordering as COrdering;
use std::sync::atomic::{AtomicUsize};
use std::sync::atomic::Ordering as AOrdering;

/// Collision map
/// Specialized structure which holds lists of candidate close items
///

#[derive(Debug)]
pub struct CollisionMap<'a, I>
where I : PrimInt + 'a,
      <I as Num>::FromStrRadixErr : 'a
{
    cells: Vec< VecDeque<&'a Entry<I>> >,
    mask: I,
    len: usize
}

impl<'a, I> CollisionMap<'a, I>
where I : PrimInt + 'a,
      <I as Num>::FromStrRadixErr : 'a
{
    pub fn new(num_bits : usize, seed : usize) -> CollisionMap<'a, I> {
        let size : usize = 2.pow(num_bits as u32);
        CollisionMap {
            cells: (0 .. size).map(|_| VecDeque::with_capacity(30)).collect(),
            mask : gen_mask(num_bits, seed).unwrap(),
            len : 0
        }
    }

    pub fn insert (&mut self, val: &'a Entry<I>) {
        self.cells[collapse(val.hash, self.mask)].push_back(val);
        self.len += 1;
    }

    pub fn score(&self) -> usize {
        let mut count = 0;
        let mut perc = 0;
        let tenth = self.len / 10;

        for i in self.cells.iter() {
            let sz = i.len();
            if sz < 1 { continue; }
            let mut slf = 0;

            for d in i.iter().take(sz - 1) {
                count += 1;
                if count % tenth == 0 { println!("{}0% done", perc); perc +=1 }
                for e in i.iter().skip(slf + 1) {
                    let dst = distance(d.hash, e.hash) as usize;
                    d.total.fetch_add(dst, AOrdering::SeqCst);
                    d.count.fetch_add(1, AOrdering::SeqCst);
                    e.total.fetch_add(dst, AOrdering::SeqCst);
                    e.count.fetch_add(1, AOrdering::SeqCst);
                }
                slf += 1;
            }
        }
        count
    }
}

pub struct Entry<I : PrimInt> {
    pub id : u64,
    pub score : f64,
    pub hash : I,
    pub total : AtomicUsize,
    pub count : AtomicUsize
}

impl<I> PartialEq for Entry<I> where I : PrimInt {
    fn eq(&self,  other: &Entry<I>) -> bool {
        self.score == other.score
    }
}

impl<I> Eq for Entry<I> where I : PrimInt {
}

impl<I> PartialOrd for Entry<I> where I : PrimInt {
    fn partial_cmp(&self, other: &Entry<I>) -> Option<COrdering> {
        if self.score > other.score {
            Some(COrdering::Greater)
        } else if self.score < other.score {
            Some(COrdering::Less)
        } else {
            Some(COrdering::Equal)
        }
    }
}

impl<I> Ord for Entry<I> where I : PrimInt {
    fn cmp(&self, other: &Entry<I>) -> COrdering {
        self.partial_cmp(other).unwrap()
    }
}


// this is a really dumb generator with no actual upper bound on execution
// time, but it'll probably end up being faster than a sampling algo
fn gen_mask<I : PrimInt>(num_bits : usize, seed : usize) -> Option<I> {
    let mut rng = StdRng::from_seed(&[seed]);
    if (num_bits / 8) > mem::size_of::<I>() {
        return None;
    }

    loop {
        let item : I = NumCast::from(rng.next_u64() & (I::max_value().to_u64().unwrap())).unwrap();
        if item.count_ones() == num_bits as u32 {
            return Some(item)
        }
    }

    return None;
}


// a perfect hash algo of sorts to place the value
#[inline]
fn collapse<I : PrimInt>(num: I, mask: I) -> usize {
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
fn distance<I : PrimInt>(a : I, b : I) -> u32 {
    (a ^ b).count_ones()
}

impl<I> fmt::Debug for Entry<I> where I : PrimInt {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "Entry{{ id: {}, score: {}, hash: {:b}, total: {}, count: {} }}",
                 self.id, self.score, self.hash.to_usize().unwrap(),
                 self.total.load(AOrdering::Relaxed),
                 self.count.load(AOrdering::Relaxed))
    }
}
/*
impl<I> fmt::Debug for CollisionMap<I> where I : PrimInt {
    fn fmt(&self, f : &mut fmt::Formatter)  -> Result<(), fmt::Error> {
        try!(writeln!(f, "CV [ mask: {:b}", self.mask.to_u64().unwrap()));
        try!(writeln!(f, "Vecs ["));
        for i in self.cells.iter() {
            try!(writeln!(f, "{:?}", *i))
        }
        writeln!(f, "]]")
    }
}*/
