use std::cmp::Ordering;
use std::mem;
use std::ops::ControlFlow;

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

    fn partial_cmp(&self, other: &Rhs) -> Self::Output;
}

impl<H1, H2, T1, T2> TuplePartialOrd<(H2, T2)> for (H1, T1)
where
    H1: PartialOrd<H2>,
    T1: TuplePartialOrd<T2>,
{
    type Output = (Option<Ordering>, T1::Output);

    fn partial_cmp(&self, other: &(H2, T2)) -> Self::Output {
        (self.0.partial_cmp(&other.0), self.1.partial_cmp(&other.1))
    }
}

impl TuplePartialOrd<()> for () {
    type Output = ();

    fn partial_cmp(&self, _other: &()) -> Self::Output {
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
    fn dominates(&self, other: &Rhs) -> Ordering;
}

struct DomReducer;

impl Reducer<ControlFlow<Ordering, Ordering>, Option<Ordering>> for DomReducer {
    type Output = ControlFlow<Ordering, Ordering>;

    #[inline(always)]
    fn apply(&self, v1: ControlFlow<Ordering, Ordering>, v2: Option<Ordering>) -> Self::Output {
        match v1 {
            ControlFlow::Continue(v1) => {
                let (mut v1, mut v2) = (v1, v2.unwrap_or(Ordering::Equal));
                if v2 < v1 {
                    mem::swap(&mut v1, &mut v2)
                }
                match (v1, v2) {
                    (Ordering::Less, Ordering::Greater) => ControlFlow::Break(Ordering::Equal),
                    (Ordering::Less, _) => ControlFlow::Continue(Ordering::Less),
                    (Ordering::Equal, Ordering::Equal) => ControlFlow::Continue(Ordering::Equal),
                    (Ordering::Equal, Ordering::Greater)
                    | (Ordering::Greater, Ordering::Greater) => {
                        ControlFlow::Continue(Ordering::Greater)
                    }
                    _ => unreachable!("Unreachable by construction"),
                }
            }
            b => b,
        }
    }
}

impl<T1, T2> Dom<T2> for T1
where
    T1: TuplePartialOrd<T2>,
    T1::Output: TupleFold<
            DomReducer,
            ControlFlow<Ordering, Ordering>,
            Output = ControlFlow<Ordering, Ordering>,
        >,
{
    fn dominates(&self, other: &T2) -> Ordering {
        match self
            .partial_cmp(other)
            .fold(DomReducer, ControlFlow::Continue(Ordering::Equal))
        {
            ControlFlow::Continue(o) | ControlFlow::Break(o) => o,
        }
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
