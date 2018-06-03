extern crate forests;
use forests::*;

#[test]
fn basic_forest_001() {
    let /*mut */ forest: Forest<usize> = Forest::new();

    let expected_output: Vec<(_, _)> = vec![];
    let actual_output: Vec<(_, _)> = forest.iter().collect::<Vec<_>>();
    assert_eq!(expected_output.as_slice(), actual_output.as_slice());
}

#[test]
fn basic_forest_002() {
    let mut forest: Forest<usize> = Forest::new();
    forest.create_node(1usize);
    forest.create_node(2usize);
    forest.create_node(3usize);

    let expected_output: Vec<(_, _)> = vec![
        (IterMovement::DownFirst(1), &1usize),
        (IterMovement::Right, &2usize),
        (IterMovement::Right, &3usize),
    ];
    let actual_output: Vec<(_, _)> = forest.iter().collect::<Vec<_>>();
    assert_eq!(expected_output.as_slice(), actual_output.as_slice());
}

#[test]
fn basic_forest_003() {
    let mut forest: Forest<usize> = Forest::new();
    forest.create_node(7usize);
    forest.create_node(8usize);
    forest.create_node(9usize);

    for (idx, r) in forest.iter_mut().values().enumerate() {
        *r = idx;
    }

    let expected_output: Vec<(_, _)> = vec![
        (IterMovement::DownFirst(1), &0usize),
        (IterMovement::Right, &1usize),
        (IterMovement::Right, &2usize),
    ];
    let actual_output: Vec<(_, _)> = forest.iter().collect::<Vec<_>>();
    assert_eq!(expected_output.as_slice(), actual_output.as_slice());
}


