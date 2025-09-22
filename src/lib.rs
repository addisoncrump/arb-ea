#![no_std]

extern crate alloc;

pub mod tuples;

use crate::tuples::Dom;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

pub fn fast_non_dominated_sort<T>(popululation: &[T]) -> (Vec<usize>, Vec<Box<[usize]>>)
where
    T: Dom<T>,
{
    let mut s = vec![Vec::new(); popululation.len()];
    let mut n = vec![0usize; popululation.len()];
    let mut ranks = vec![usize::MAX; popululation.len()];

    let mut front = Vec::with_capacity(popululation.len());
    for (p, first) in popululation.iter().enumerate() {
        for q in (p + 1)..popululation.len() {
            let other = &popululation[q];
            let (dominator, dominated) = match first.dominates(other).unwrap_or(Ordering::Equal) {
                Ordering::Less => (p, q),
                Ordering::Equal => continue,
                Ordering::Greater => (q, p),
            };
            s[dominator].push(dominated);
            n[dominated] += 1;
        }
        if n[p] == 0 {
            ranks[p] = 0;
            front.push(p);
        }
    }

    let front = front.into_boxed_slice();
    let mut remaining = popululation.len() - front.len();
    let mut fronts = vec![front];
    for i in 0.. {
        if remaining == 0 {
            break;
        }

        let front = &fronts[i];
        let mut next_front = Vec::with_capacity(remaining);
        for &p in front {
            for &q in &s[p] {
                n[q] -= 1;
                if n[q] == 0 {
                    ranks[q] = i + 1;
                    next_front.push(q);
                }
            }
        }

        remaining -= next_front.len();
        fronts.push(next_front.into_boxed_slice());
    }

    (ranks, fronts)
}
