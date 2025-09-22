//! These tests taken from moo-rs, and bear the corresponding license.
//! See: https://github.com/andresliszt/moo-rs/blob/decdd829c9d96c52c13cfaac9cb65802f6093abf/LICENSE

use arb_ea::fast_non_dominated_sort;
use arb_ea::tuples::{Dom, EvaluateTuple};
use std::cmp::Ordering;
use tuple_list::tuple_list;

#[test]
fn dominates() {
    // Test case 1: The first vector dominates the second
    let a = tuple_list![1.0, 2usize, 3u32, false];
    let b = tuple_list![2.0, 3usize, 4u32, true];
    assert_eq!(a.dominates(&b), Some(Ordering::Less));

    // Test case 2: The second vector dominates the first
    let a = tuple_list![3.0, 4usize, 5u32, true];
    let b = tuple_list![3.0, 3usize, 3u32, false];
    assert_eq!(a.dominates(&b), Some(Ordering::Greater));

    // Test case 3: Neither vector dominates the other
    let a = tuple_list![1.0, 2usize, 3u32, false];
    let b = tuple_list![2.0, 1usize, 3u32, true];
    assert_eq!(a.dominates(&b), None);

    // Test case 4: Equal vectors
    let a = tuple_list![1.0, 2usize, 3u32, false];
    let b = tuple_list![1.0, 2usize, 3u32, false];
    assert_eq!(a.dominates(&b), Some(Ordering::Equal));
}

#[test]
fn test_fast_non_dominated_sorting() {
    // Define the fitness values of the population
    let population_fitness = vec![
        tuple_list![1.0, 4u32], // Individual 0
        tuple_list![2.0, 2u32], // Individual 1
        tuple_list![1.5, 3u32], // Individual 2
        tuple_list![3.0, 8u32], // Individual 3 (dominated by everyone)
        tuple_list![4.0, 6u32], // Individual 4 (dominated by everyone)
    ];

    // Perform fast non-dominated sorting with min_survivors = 5
    let fronts = fast_non_dominated_sort(&population_fitness);

    // Expected Pareto fronts:
    // Front 1: Individuals 0, 1, 2
    // Front 2: Individuals 3, 4 (the entire front is included when min_survivors is reached)
    let expected_fronts = vec![
        vec![0, 1, 2].into_boxed_slice(), // Front 1
        vec![3, 4].into_boxed_slice(),    // Front 2
    ];

    assert_eq!(fronts.1, expected_fronts);
}

struct A {
    value: usize,
}

#[test]
fn mapping() {
    let source_fitness = vec![
        (1.0, 4), // Individual 0
        (2.0, 2), // Individual 1
        (1.5, 3), // Individual 2
        (3.0, 8), // Individual 3 (dominated by everyone)
        (4.0, 6), // Individual 4 (dominated by everyone)
    ];

    // rust formatter is garbage here...
    #[rustfmt::skip]
    let mut fitness_functions = tuple_list!(
        |a: &A| { source_fitness[a.value].0 },
        |a: &A| { source_fitness[a.value].1 },
    );

    // produce a "population" (A { value: 0..5 }) and instantly evaluate it
    let population_fitness = (0..source_fitness.len())
        .map(|value| A { value })
        .map(|a| fitness_functions.evaluate(&a))
        .collect::<Vec<_>>();

    // Perform fast non-dominated sorting with min_survivors = 5
    let fronts = fast_non_dominated_sort(&population_fitness);

    // Expected Pareto fronts:
    // Front 1: Individuals 0, 1, 2
    // Front 2: Individuals 3, 4 (the entire front is included when min_survivors is reached)
    let expected_fronts = vec![
        vec![0, 1, 2].into_boxed_slice(), // Front 1
        vec![3, 4].into_boxed_slice(),    // Front 2
    ];

    assert_eq!(fronts.1, expected_fronts);
}
