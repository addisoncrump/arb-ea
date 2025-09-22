use arb_ea::fast_non_dominated_sort;
use arb_ea::tuples::{Dom, DominationOrd};
use std::cmp::{Ordering, Reverse};
use std::collections::BTreeMap;
use tuple_list::tuple_list;

#[test]
fn membership_dom() {
    let population_fitness = [
        BTreeMap::new(),                  // weakest
        BTreeMap::from([(1, 1)]),         // dominates 0
        BTreeMap::from([(0, 0), (1, 1)]), // dominates 0, 1
        BTreeMap::from([(2, 1)]),         // dominates 0, but not 1 or 2
        BTreeMap::from([(1, 0), (2, 0)]), // dominates 0
        BTreeMap::from([(1, 0), (2, 1)]), // dominates 0, 3, 4
    ]
    .map(|f| Reverse(f));

    let dominations = vec![vec![], vec![0], vec![0, 1], vec![0], vec![0], vec![0, 3, 4]];
    for (candidate, dominates) in dominations.into_iter().enumerate() {
        for i in 0..population_fitness.len() {
            if dominates.contains(&i) {
                assert!(
                    population_fitness[candidate]
                        .dominates(&population_fitness[i])
                        .map_or(false, Ordering::is_lt),
                    "{candidate} should dominate {i}"
                );
            } else {
                assert!(
                    population_fitness[candidate]
                        .dominates(&population_fitness[i])
                        .map_or(true, Ordering::is_ge),
                    "{candidate} should not dominate {i}"
                );
            }
        }
    }

    let (_ranks, fronts) = fast_non_dominated_sort(&population_fitness);

    let expected = vec![
        vec![1, 2, 3, 4, 5].into_boxed_slice(),
        vec![0].into_boxed_slice(),
    ];
    assert_eq!(fronts, expected);
}

#[test]
fn membership_and_others() {
    let population_fitness = [
        BTreeMap::new(),                  // weakest
        BTreeMap::from([(1, 1)]),         // dominates 0
        BTreeMap::from([(0, 0), (1, 1)]), // dominates 0, 1
        BTreeMap::from([(2, 1)]),         // dominates 0, but not 1 or 2
        BTreeMap::from([(1, 0), (2, 0)]), // dominates 0
        BTreeMap::from([(1, 0), (2, 1)]), // dominates 0, 3, 4
    ]
    .map(|f| tuple_list!(0usize, 1.0, DominationOrd(Reverse(f))));

    let dominations = vec![vec![], vec![0], vec![0, 1], vec![0], vec![0], vec![0, 3, 4]];
    for (candidate, dominates) in dominations.into_iter().enumerate() {
        for i in 0..population_fitness.len() {
            if dominates.contains(&i) {
                assert!(
                    population_fitness[candidate]
                        .dominates(&population_fitness[i])
                        .map_or(false, Ordering::is_lt),
                    "{candidate} should dominate {i}"
                );
            } else {
                assert!(
                    population_fitness[candidate]
                        .dominates(&population_fitness[i])
                        .map_or(true, Ordering::is_ge),
                    "{candidate} should not dominate {i}"
                );
            }
        }
    }

    let (_ranks, fronts) = fast_non_dominated_sort(&population_fitness);

    let expected = vec![
        vec![1, 2, 3, 4, 5].into_boxed_slice(),
        vec![0].into_boxed_slice(),
    ];
    assert_eq!(fronts, expected);
}
