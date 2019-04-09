#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::correctness)]

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

thread_local! {
    static TEST_DIR: PathBuf = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/data/");
        path
    };
}

fn create_path(name: &str) -> PathBuf {
    let mut path: PathBuf = TEST_DIR.with(|x| x.clone());
    path.push(name);
    path
}

fn test_file(name: &str) {
    let mut asm_file = create_path(name);
    let mut hack_file = asm_file.clone();
    asm_file.set_extension("asm");
    hack_file.set_extension("hack");

    let my = hasm::assemble_file(&asm_file).unwrap();
    let mut cmp = String::new();
    let _ = File::open(&hack_file).unwrap().read_to_string(&mut cmp);

    println!("is:\n{}", my);
    println!("should:\n{}", cmp);

    assert_eq!(my, cmp);
}

#[test]
fn add() {
    test_file("Add");
}

#[test]
fn max_l() {
    test_file("MaxL");
}

#[test]
fn max() {
    test_file("Max");
}

#[test]
fn rect() {
    test_file("Rect");
}

#[test]
fn rect_l() {
    test_file("RectL");
}

#[test]
fn pong() {
    test_file("Pong");
}

#[test]
fn pong_l() {
    test_file("PongL");
}
