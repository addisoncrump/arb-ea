//! These tests taken from moo-rs, and bear the corresponding license.
//! See: https://github.com/andresliszt/moo-rs/blob/decdd829c9d96c52c13cfaac9cb65802f6093abf/LICENSE

use arb_ea::fast_non_dominated_sort;
use arb_ea::tuples::Dom;
use std::cmp::Ordering;
use tuple_list::tuple_list;

#[test]
fn dominates() {
    // Test case 1: The first vector dominates the second
    let a = tuple_list![1.0, 2.0, 3.0];
    let b = tuple_list![2.0, 3.0, 4.0];
    assert_eq!(a.dominates(&b), Ordering::Less);

    // Test case 2: The second vector dominates the first
    let a = tuple_list![3.0, 4.0, 5.0];
    let b = tuple_list![3.0, 3.0, 3.0];
    assert_eq!(a.dominates(&b), Ordering::Greater);

    // Test case 3: Neither vector dominates the other
    let a = tuple_list![1.0, 2.0, 3.0];
    let b = tuple_list![2.0, 1.0, 3.0];
    assert_eq!(a.dominates(&b), Ordering::Equal);

    // Test case 4: Equal vectors
    let a = tuple_list![1.0, 2.0, 3.0];
    let b = tuple_list![1.0, 2.0, 3.0];
    assert_eq!(a.dominates(&b), Ordering::Equal);
}

#[test]
fn test_fast_non_dominated_sorting() {
    // Define the fitness values of the population
    let population_fitness = vec![
        tuple_list![1.0, 2.0], // Individual 0
        tuple_list![2.0, 1.0], // Individual 1
        tuple_list![1.5, 1.5], // Individual 2
        tuple_list![3.0, 4.0], // Individual 3 (dominated by everyone)
        tuple_list![4.0, 3.0], // Individual 4 (dominated by everyone)
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
