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
        (IterMovement::DownFirst(0), &1usize),
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
        (IterMovement::DownFirst(0), &0usize),
        (IterMovement::Right, &1usize),
        (IterMovement::Right, &2usize),
    ];
    let actual_output: Vec<(_, _)> = forest.iter().collect::<Vec<_>>();
    assert_eq!(expected_output.as_slice(), actual_output.as_slice());
}

#[test]
fn basic_forest_004() {
    let mut forest: Forest<usize> = Forest::new();
    let node1 = forest.create_node(1usize);
    let node2 = forest.create_node(2usize);
    let node3 = forest.create_node(3usize);
    let node4 = forest.create_node(4usize);
    let _node5 = forest.create_node(5usize);
    forest.append_node_child(node2, node1);
    forest.append_node_child(node2, node3);
    forest.prepend_node_child(node3, node4);

    let expected_output: Vec<(_, _)> = vec![
        (IterMovement::DownFirst(0), &2usize),
        (IterMovement::DownFirst(1), &1usize),
        (IterMovement::Right, &3usize),
        (IterMovement::DownFirst(1), &4usize),
        (IterMovement::UpNRight(2), &5usize),
    ];
    let actual_output: Vec<(_, _)> = forest.iter().collect::<Vec<_>>();
    assert_eq!(expected_output.as_slice(), actual_output.as_slice());

    let expected_output: Vec<(_, _)> = vec![
        (IterMovement::DownFirst(1), &1usize),
        (IterMovement::RightDownFirstN(1), &4usize),
        (IterMovement::Up(1), &3usize),
        (IterMovement::Up(1), &2usize),
        (IterMovement::Right, &5usize),
    ];
    let actual_output: Vec<(_, _)> = forest.iter().mode(IterMode::PostOrder).collect::<Vec<_>>();
    assert_eq!(expected_output.as_slice(), actual_output.as_slice());
}

#[test]
fn basic_forest_005() {
    let mut forest: Forest<usize> = Forest::new();
    let node1 = forest.create_node(1usize);
    let node2 = forest.create_node(2usize);
    let node3 = forest.create_node(3usize);
    let node4 = forest.create_node(4usize);
    let _node5 = forest.create_node(5usize);
    forest.append_node_child(node2, node1);
    forest.insert_node_child_before(node2, node3, node1);
    forest.insert_node_child_after(node2, node4, node1);

    let expected_output: Vec<(_, _)> = vec![
        (IterMovement::DownFirst(0), &2usize),
        (IterMovement::DownFirst(1), &3usize),
        (IterMovement::Right, &1usize),
        (IterMovement::Right, &4usize),
        (IterMovement::UpNRight(1), &5usize),
    ];
    let actual_output: Vec<(_, _)> = forest.iter().collect::<Vec<_>>();
    assert_eq!(expected_output.as_slice(), actual_output.as_slice());

    let expected_output: Vec<(_, _)> = vec![
        (IterMovement::DownFirst(1), &3usize),
        (IterMovement::Right, &1usize),
        (IterMovement::Right, &4usize),
        (IterMovement::Up(1), &2usize),
        (IterMovement::Right, &5usize),
    ];
    let actual_output: Vec<(_, _)> = forest.iter().mode(IterMode::PostOrder).collect::<Vec<_>>();
    assert_eq!(expected_output.as_slice(), actual_output.as_slice());
}
