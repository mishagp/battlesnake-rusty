use log::info;
use serde_json::{json, Value};
use std::collections::HashMap;
use rand::prelude::IndexedRandom;

use crate::{Battlesnake, Board, Coord, Game, Move};

pub fn info() -> Value {
    json!({
        "apiversion": "1",
        "author": "mishagp",
        "color": "#a72145",
        "head": "silly",
        "tail": "sharp",
    })
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

pub fn get_move(_game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Value {
    let mut is_move_safe: HashMap<_, _> = vec![
        (Move::Up, true),
        (Move::Down, true),
        (Move::Left, true),
        (Move::Right, true),
    ]
    .into_iter()
    .collect();

    let mut is_move_desirable: HashMap<_, _> = vec![
        (Move::Up, true),
        (Move::Down, true),
        (Move::Left, true),
        (Move::Right, true),
    ]
    .into_iter()
    .collect();

    let my_head = &you.body[0];
    let my_body = &you.body;
    let board_width = &board.width;
    let board_height = &board.height;

    //Get count of my body parts in each quadrant
    let mut my_body_quadrant_count = vec![0, 0, 0, 0];
    for body_part_coord in &you.body {
        let body_part_quadrant = match (
            body_part_coord.x < board_width / 2,
            body_part_coord.y < board_height / 2,
        ) {
            (true, true) => 1,
            (true, false) => 2,
            (false, true) => 3,
            (false, false) => 4,
        };
        my_body_quadrant_count[body_part_quadrant - 1] += 1;
    }

    //Get count of my food in each quadrant
    let mut my_food_quadrant_count = vec![0, 0, 0, 0];
    for food in &board.food {
        let food_quadrant = match (food.x < board_width / 2, food.y < board_height / 2) {
            (true, true) => 1,
            (true, false) => 2,
            (false, true) => 3,
            (false, false) => 4,
        };
        my_food_quadrant_count[food_quadrant - 1] += 1;
    }

    let sorted_food: &mut Vec<Coord> = &mut board.food.clone();
    sorted_food.sort_by(|a, b| {
        let a_dist = (a.x - my_head.x).abs() + (a.y - my_head.y).abs();
        let b_dist = (b.x - my_head.x).abs() + (b.y - my_head.y).abs();
        a_dist.cmp(&b_dist)
    });

    let chosen: &Move;
    let mut shout: &str = "";

    if my_head.x <= 1 {
        if my_head.x == 0 {
            is_move_safe.insert(Move::Left, false);
        } else {
            is_move_desirable.insert(Move::Left, false);
        }
    } else if my_head.x >= board_width - 2 {
        if my_head.x >= board_width - 1 {
            is_move_safe.insert(Move::Right, false);
        } else {
            is_move_desirable.insert(Move::Right, false);
        }
    }

    if my_head.y <= 1 {
        if my_head.y == 0 {
            is_move_safe.insert(Move::Down, false);
        } else {
            is_move_desirable.insert(Move::Down, false);
        }
    } else if my_head.y >= board_height - 2 {
        if my_head.y == board_height - 1 {
            is_move_safe.insert(Move::Up, false);
        } else {
            is_move_desirable.insert(Move::Up, false);
        }
    }

    // Prevent your Battlesnake from colliding with itself
    for body_part_coord in my_body {
        if body_part_coord.x == my_head.x && body_part_coord.y == my_head.y {
            info!("self collision ruled out");
        } else {
            if body_part_coord.x == my_head.x - 1 && body_part_coord.y == my_head.y {
                info!("body collision left ruled out");
                is_move_safe.insert(Move::Left, false);
            } else if body_part_coord.x == my_head.x + 1 && body_part_coord.y == my_head.y {
                info!("body collision right ruled out");
                is_move_safe.insert(Move::Right, false);
            }

            if body_part_coord.y == my_head.y - 1 && body_part_coord.x == my_head.x {
                info!("body collision down ruled out");
                is_move_safe.insert(Move::Down, false);
            } else if body_part_coord.y == my_head.y + 1 && body_part_coord.x == my_head.x {
                info!("body collision up ruled out");
                is_move_safe.insert(Move::Up, false);
            }
        }
    }

    for opponent in &board.snakes {
        //Iterate over each opponent's body part coordinates and mark moves unsafe if they are in the way
        for body_part_coord in &opponent.body {
            if body_part_coord.x == my_head.x - 1 && body_part_coord.y == my_head.y {
                info!("opponent collision left ruled out");
                is_move_safe.insert(Move::Left, false);
            } else if body_part_coord.x == my_head.x + 1 && body_part_coord.y == my_head.y {
                info!("opponent collision right ruled out");
                is_move_safe.insert(Move::Right, false);
            }

            if body_part_coord.y == my_head.y - 1 && body_part_coord.x == my_head.x {
                info!("opponent collision down ruled out");
                is_move_safe.insert(Move::Down, false);
            } else if body_part_coord.y == my_head.y + 1 && body_part_coord.x == my_head.x {
                info!("opponent collision up ruled out");
                is_move_safe.insert(Move::Up, false);
            }
        }
    }

    // Are there any safe moves left?
    let safe_moves = is_move_safe
        .into_iter()
        .filter(|&(_, v)| v)
        .map(|(k, _)| k)
        .collect::<Vec<_>>();

    // Are there any desirable moves left?
    let desirable_moves = is_move_desirable
        .into_iter()
        .filter(|&(_, v)| v)
        .map(|(k, _)| k)
        .collect::<Vec<_>>();

    // If there is more than one safe move, choose a desirable move which is also a safe move
    if safe_moves.len() > 1 {
        let safe_desirable_moves: &[&Move] = &safe_moves
            .iter()
            .filter(|&m| desirable_moves.contains(&m))
            .collect::<Vec<_>>();

        if safe_desirable_moves.len() > 0 {
            // Determine which quadrant has the least body parts
            let mut min_quadrant = 0;
            let mut min_quadrant_count = my_body_quadrant_count[0];
            let mut max_food_count = 0;

            for i in 1..4 {
                if my_body_quadrant_count[i] < min_quadrant_count {
                    min_quadrant = i;
                    min_quadrant_count = my_body_quadrant_count[i];
                    max_food_count = 0;
                } else if my_body_quadrant_count[i] == min_quadrant_count {
                    if my_food_quadrant_count[i] > max_food_count {
                        min_quadrant = i;
                        max_food_count = my_food_quadrant_count[i];
                    }
                }
            }

            // Choose a move that moves towards min_quadrant
            let mut min_quadrant_moves = vec![];
            for &move_ in safe_desirable_moves {
                match move_ {
                    Move::Up => {
                        if min_quadrant == 1 || min_quadrant == 2 {
                            min_quadrant_moves.push(move_);
                        }
                    }
                    Move::Down => {
                        if min_quadrant == 3 || min_quadrant == 4 {
                            min_quadrant_moves.push(move_);
                        }
                    }
                    Move::Left => {
                        if min_quadrant == 1 || min_quadrant == 3 {
                            min_quadrant_moves.push(move_);
                        }
                    }
                    Move::Right => {
                        if min_quadrant == 2 || min_quadrant == 4 {
                            min_quadrant_moves.push(move_);
                        }
                    }
                }
            }
            if min_quadrant_moves.len() > 0 {
                // Filter sorted_food to only include food in min_quadrant
                let mut min_quadrant_food = vec![];
                for food in sorted_food {
                    match min_quadrant {
                        1 => {
                            if food.x < board_width / 2 && food.y < board_height / 2 {
                                min_quadrant_food.push(food);
                            }
                        }
                        2 => {
                            if food.x < board_width / 2 && food.y >= board_height / 2 {
                                min_quadrant_food.push(food);
                            }
                        }
                        3 => {
                            if food.x >= board_width / 2 && food.y < board_height / 2 {
                                min_quadrant_food.push(food);
                            }
                        }
                        4 => {
                            if food.x >= board_width / 2 && food.y >= board_height / 2 {
                                min_quadrant_food.push(food);
                            }
                        }
                        _ => {}
                    }
                }

                // Choose a move from min_quadrant_moves that moves towards the first food in min_quadrant_food
                let mut min_quadrant_food_moves = vec![];
                for &move_ in &min_quadrant_moves {
                    let (dx, dy) = match move_ {
                        Move::Up => (0, -1),
                        Move::Down => (0, 1),
                        Move::Left => (-1, 0),
                        Move::Right => (1, 0),
                    };
                    let new_head = Coord {
                        x: my_head.x + dx,
                        y: my_head.y + dy,
                    };
                    if min_quadrant_food.len() > 0 {
                        let food = &min_quadrant_food[0];
                        if new_head.x == food.x && new_head.y == food.y {
                            min_quadrant_food_moves.push(move_);
                        }
                    }
                }

                if min_quadrant_food_moves.len() > 0 {
                    chosen = min_quadrant_food_moves
                        .choose(&mut rand::rng())
                        .unwrap();
                } else {
                    chosen = &min_quadrant_moves.choose(&mut rand::rng()).unwrap();
                }
            } else {
                chosen = safe_desirable_moves
                    .choose(&mut rand::rng())
                    .unwrap();
            }
        } else {
            // Choose a random move from the safe ones
            chosen = safe_moves.choose(&mut rand::rng()).unwrap();
        }
    } else {
        if safe_moves.len() == 0 {
            shout = "The only winning move is not to play...";
            chosen = &Move::Up;
        } else {
            // Choose a random move from the safe ones
            chosen = safe_moves.choose(&mut rand::rng()).unwrap();
        }
    }

    info!("MOVE {}: {}", turn, chosen.as_str());
    let response = if shout.is_empty() {
        json!({
            "move": chosen.as_str(),
        })
    } else {
        json!({
            "move": chosen.as_str(),
            "shout": shout,
        })
    };
    response
}
