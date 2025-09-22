use alloc::collections::BTreeMap;
use core::cmp::{Ordering, Reverse};
use core::mem;
use core::ops::ControlFlow;

pub trait TupleLen {
    const LEN: usize;
}

impl<H, T> TupleLen for (H, T)
where
    T: TupleLen,
{
    const LEN: usize = T::LEN + 1;
}

impl TupleLen for () {
    const LEN: usize = 0;
}

pub trait TuplePartialOrd<Rhs> {
    type Output;

    fn partial_cmp_many(&self, other: &Rhs) -> Self::Output;
}

impl<H1, H2, T1, T2> TuplePartialOrd<(H2, T2)> for (H1, T1)
where
    H1: PartialOrd<H2>,
    T1: TuplePartialOrd<T2>,
{
    type Output = (Option<Ordering>, T1::Output);

    fn partial_cmp_many(&self, other: &(H2, T2)) -> Self::Output {
        (
            self.0.partial_cmp(&other.0),
            self.1.partial_cmp_many(&other.1),
        )
    }
}

impl TuplePartialOrd<()> for () {
    type Output = ();

    fn partial_cmp_many(&self, _other: &()) -> Self::Output {
        ()
    }
}

pub trait Reducer<I1, I2> {
    type Output;

    fn apply(&self, v1: I1, v2: I2) -> Self::Output;
}

impl<I1, I2, O, F> Reducer<I1, I2> for F
where
    F: Fn(I1, I2) -> O,
{
    type Output = O;

    fn apply(&self, v1: I1, v2: I2) -> Self::Output {
        self(v1, v2)
    }
}

pub trait TupleFold<F, T> {
    type Output;

    fn fold(self, reducer: F, with: T) -> Self::Output;
}

impl<F, H, T, U> TupleFold<F, U> for (H, T)
where
    T: TupleFold<F, F::Output>,
    F: Reducer<U, H>,
{
    type Output = T::Output;

    fn fold(self, reducer: F, with: U) -> Self::Output {
        let res = reducer.apply(with, self.0);
        self.1.fold(reducer, res)
    }
}

impl<F, U> TupleFold<F, U> for () {
    type Output = U;

    fn fold(self, _reducer: F, with: U) -> Self::Output {
        with
    }
}

pub trait TupleReduce<F> {
    type Output;

    fn reduce(self, reducer: F) -> Self::Output;
}

impl<F, H, T> TupleReduce<F> for (H, T)
where
    T: TupleFold<F, H>,
{
    type Output = T::Output;

    fn reduce(self, reducer: F) -> Self::Output {
        self.1.fold(reducer, self.0)
    }
}

pub trait Dom<Rhs = Self> {
    fn dominates(&self, other: &Rhs) -> Option<Ordering>;
}

struct DomReducer;

impl Reducer<ControlFlow<(), Ordering>, Option<Ordering>> for DomReducer {
    type Output = ControlFlow<(), Ordering>;

    #[inline(always)]
    fn apply(&self, v1: ControlFlow<(), Ordering>, v2: Option<Ordering>) -> Self::Output {
        match v1 {
            ControlFlow::Continue(mut v1) => {
                let ordering = match v2 {
                    None => v1, // no way to compare on v2, so we just keep going
                    Some(mut v2) => {
                        if v2 < v1 {
                            mem::swap(&mut v1, &mut v2)
                        }
                        match (v1, v2) {
                            (Ordering::Less, Ordering::Greater) => return ControlFlow::Break(()),
                            (Ordering::Less, Ordering::Equal | Ordering::Less) => Ordering::Less,
                            (Ordering::Equal, Ordering::Equal) => Ordering::Equal,
                            (Ordering::Equal | Ordering::Greater, Ordering::Greater) => {
                                Ordering::Greater
                            }
                            _ => unreachable!("Unreachable by construction"),
                        }
                    }
                };
                ControlFlow::Continue(ordering)
            }
            b => b,
        }
    }
}

// we implement this over tuples to disambiguate and avoid hitting the child rule
impl<H, T1, T2> Dom<T2> for (H, T1)
where
    (H, T1): TuplePartialOrd<T2>,
    <(H, T1) as TuplePartialOrd<T2>>::Output:
        TupleFold<DomReducer, ControlFlow<(), Ordering>, Output = ControlFlow<(), Ordering>>,
{
    fn dominates(&self, other: &T2) -> Option<Ordering> {
        self.partial_cmp_many(other)
            .fold(DomReducer, ControlFlow::Continue(Ordering::Equal))
            .continue_value()
    }
}

impl Dom<()> for () {
    fn dominates(&self, _other: &()) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl<T> Dom<Reverse<T>> for Reverse<T>
where
    T: Dom<T>,
{
    fn dominates(&self, other: &Reverse<T>) -> Option<Ordering> {
        self.0.dominates(&other.0).map(|o| o.reverse())
    }
}

impl<K, V> Dom<BTreeMap<K, V>> for BTreeMap<K, V>
where
    K: Ord,
    V: PartialOrd,
{
    fn dominates(&self, other: &BTreeMap<K, V>) -> Option<Ordering> {
        let mut first_iter = self.iter();
        let mut second_iter = other.iter();

        let mut first_greater = false;
        let mut second_greater = false;

        let mut maybe_second = second_iter.next();
        while let Some(first) = first_iter.next() {
            loop {
                if let Some(second) = maybe_second {
                    match first.0.cmp(second.0) {
                        Ordering::Less => {
                            // we need to advance first
                            if second_greater {
                                return None; // neither dominates
                            }
                            first_greater = true;
                            break;
                        }
                        Ordering::Equal => {
                            // compare
                            match first.1.partial_cmp(second.1) {
                                Some(Ordering::Greater) => {
                                    if second_greater {
                                        return None; // neither dominates
                                    }
                                    first_greater = true;
                                }
                                Some(Ordering::Less) => {
                                    if first_greater {
                                        return None; // neither dominates
                                    }
                                    second_greater = true;
                                }
                                None => {
                                    return None; // cannot evaluate, so neither dominates
                                }
                                _ => {}
                            }
                            // advance both
                            maybe_second = second_iter.next();
                            break;
                        }
                        Ordering::Greater => {
                            // we need to advance second
                            if first_greater {
                                return None; // neither dominates
                            }
                            second_greater = true;
                            maybe_second = second_iter.next();
                        }
                    }
                } else if second_greater {
                    // second was greater, but shorter; neither dominates
                    return None;
                } else {
                    // second was not greater and shorter
                    return Some(Ordering::Greater);
                }
            }
        }
        if maybe_second.is_some() {
            if first_greater {
                // first was greater, but shorter; neither dominates
                None
            } else {
                // first was not greater and shorter
                Some(Ordering::Less)
            }
        } else if first_greater {
            // first is greater
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal) // we have scanned all elements; they are equal
        }
    }
}

pub struct DominationOrd<T>(pub T);

impl<T> PartialEq<DominationOrd<T>> for DominationOrd<T>
where
    T: Dom<T>,
{
    fn eq(&self, other: &DominationOrd<T>) -> bool {
        self.0.dominates(&other.0).map_or(false, Ordering::is_eq)
    }
}

impl<T> PartialOrd<DominationOrd<T>> for DominationOrd<T>
where
    T: Dom<T>,
{
    fn partial_cmp(&self, other: &DominationOrd<T>) -> Option<Ordering> {
        self.0.dominates(&other.0)
    }
}

pub trait Evaluator<I> {
    type Output;

    fn evaluate(&mut self, input: &I) -> Self::Output;
}

impl<I, O, F> Evaluator<I> for F
where
    F: FnMut(&I) -> O,
{
    type Output = O;

    fn evaluate(&mut self, input: &I) -> Self::Output {
        self(input)
    }
}

pub trait EvaluateTuple<I> {
    type Output;

    fn evaluate(&mut self, value: &I) -> Self::Output;
}

impl<T> EvaluateTuple<T> for () {
    type Output = ();

    fn evaluate(&mut self, _value: &T) -> Self::Output {}
}

impl<T, F1, FT> EvaluateTuple<T> for (F1, FT)
where
    FT: EvaluateTuple<T>,
    F1: Evaluator<T>,
{
    type Output = (F1::Output, FT::Output);

    fn evaluate(&mut self, value: &T) -> Self::Output {
        (self.0.evaluate(value), self.1.evaluate(value))
    }
}
