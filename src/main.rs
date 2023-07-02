extern crate sdl2;

use sdl2::event::Event;
// use sdl2::keyboard::Keycode;
use std::time::Duration;

use logic::game::{Game, Player};
use view::ui::{UI};

pub mod logic;
pub mod view;

const FPS: u32 = 60;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("sprouts-rs", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let player1 = Player::new(0, String::from("player 1"));
    let player2 = Player::new(1, String::from("player 2"));

    let mut game = Game::new(player1, player2, 3);

    let mut canvas = UI::new(window, game.get_nodes());

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running;
                },
                _ => canvas.process(event, &mut game),
            };
        } 

        canvas.render();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
    }
}

