#![allow(dead_code)]
#![feature(std_misc, slice_patterns)]

extern crate ecs;
extern crate rustbox;
extern crate rand;

mod entities;
mod generation;
mod game;
mod util;

fn main() {
    game::Game::new().play();
}
