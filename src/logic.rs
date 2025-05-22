use log::info;
use serde_json::{json, Value};
use std::collections::{HashMap};
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

fn coords_equal(a: &Coord, b: &Coord) -> bool {
    a.x == b.x && a.y == b.y
}

fn is_position_safe(pos: &Coord, board: &Board, you: &Battlesnake, look_ahead: bool) -> bool {
    // Check if the position is out of bounds
    if pos.x < 0 || pos.x >= board.width || pos.y < 0 || pos.y >= board.height {
        return false;
    }

    // Check if position collides with your body
    for (i, body_part) in you.body.iter().enumerate() {
        if body_part.x == pos.x && body_part.y == pos.y {
            // The tail will move, so it's safe unless the snake just ate
            if i == you.body.len() - 1 && you.body.len() > 1 && !look_ahead {
                // Check if we just ate (if health is 100, we just ate)
                if you.health < 100 {
                    return true;
                }
            }
            return false;
        }
    }

    // Check if position collides with other snakes
    for snake in &board.snakes {
        if snake.id == you.id {
            continue; // Skip your own snake, already checked
        }

        for (i, body_part) in snake.body.iter().enumerate() {
            if body_part.x == pos.x && body_part.y == pos.y {
                // The tail will move, so it's safe unless the snake just ate
                if i == snake.body.len() - 1 && snake.body.len() > 1 && !look_ahead {
                    // We don't know if other snakes just ate, so assume they didn't
                    return true;
                }
                return false;
            }
        }

        // Check for potential head-to-head collisions
        if look_ahead {
            // Calculate the distance between the snake's head and the position
            let distance = (snake.head.x - pos.x).abs() + (snake.head.y - pos.y).abs();

            // If the distance is 1, the snake could move to this position in its next turn
            if distance == 1 {
                // If the other snake is longer or equal in length, avoid this position
                if snake.length >= you.length {
                    return false;
                }
            }

            // If the position is the same as the snake's head, it's a direct collision
            if snake.head.x == pos.x && snake.head.y == pos.y {
                // If the other snake is longer or equal in length, avoid this position
                if snake.length >= you.length {
                    return false;
                }
            }
        }
    }

    true
}

// Simulate a move and return the new position
fn simulate_move(head: &Coord, move_dir: &Move) -> Coord {
    match move_dir {
        Move::Up => Coord { x: head.x, y: head.y + 1 },
        Move::Down => Coord { x: head.x, y: head.y - 1 },
        Move::Left => Coord { x: head.x - 1, y: head.y },
        Move::Right => Coord { x: head.x + 1, y: head.y },
    }
}

// Look ahead multiple moves and evaluate safety
fn evaluate_move_safety(board: &Board, you: &Battlesnake, move_dir: &Move, depth: i32) -> i32 {
    if depth == 0 {
        return 1; // Base case: move is safe at this depth
    }

    let new_head = simulate_move(&you.head, move_dir);

    // If the immediate move is unsafe, return 0
    if !is_position_safe(&new_head, board, you, true) {
        return 0;
    }

    // Create a simulated board for the next move
    let mut simulated_board = Board {
        height: board.height,
        width: board.width,
        food: board.food.clone(),
        snakes: Vec::new(),
        hazards: board.hazards.clone(),
    };

    // Create a simulated you for the next move
    let mut simulated_you = Battlesnake {
        id: you.id.clone(),
        name: you.name.clone(),
        health: you.health - 1, // Decrease health by 1 each turn
        body: you.body.clone(),
        head: new_head.clone(),
        length: you.length,
        latency: you.latency.clone(),
        shout: you.shout.clone(),
    };

    // Update the head to the new position
    simulated_you.head = new_head.clone();
    simulated_you.body.insert(0, new_head.clone());

    // Check if the snake ate food
    let mut ate_food = false;
    for (i, food) in board.food.iter().enumerate() {
        if coords_equal(&new_head, food) {
            ate_food = true;
            // Remove food from the simulated board
            simulated_board.food = board.food.clone();
            simulated_board.food.remove(i);
            // Increase health to 100 when eating food
            simulated_you.health = 100;
            // Increase length by 1
            simulated_you.length += 1;
            break;
        }
    }

    // If the snake didn't eat food, remove the tail
    if !ate_food && simulated_you.body.len() > simulated_you.length as usize {
        simulated_you.body.pop();
    }

    // Add the simulated you to the simulated board
    simulated_board.snakes.push(simulated_you.clone());

    // Simulate other snakes' movements
    for snake in &board.snakes {
        if snake.id == you.id {
            continue; // Skip your own snake, already handled
        }

        // Simple AI for other snakes: try to move towards food or away from walls
        let possible_moves = [Move::Up, Move::Down, Move::Left, Move::Right];
        let mut safe_moves = Vec::new();

        for &move_dir in &possible_moves {
            let new_head = simulate_move(&snake.head, &move_dir);

            // Skip if the move is out of bounds
            if new_head.x < 0 || new_head.x >= board.width || new_head.y < 0 || new_head.y >= board.height {
                continue;
            }

            // Skip if the move collides with any snake's body (except the tail which will move)
            let mut collision = false;
            for other_snake in &board.snakes {
                for (i, body_part) in other_snake.body.iter().enumerate() {
                    if i == other_snake.body.len() - 1 && other_snake.body.len() > 1 {
                        // Skip the tail
                        continue;
                    }
                    if body_part.x == new_head.x && body_part.y == new_head.y {
                        collision = true;
                        break;
                    }
                }
                if collision {
                    break;
                }
            }

            if !collision {
                safe_moves.push(move_dir);
            }
        }

        // If there are no safe moves, the snake is trapped
        if safe_moves.is_empty() {
            continue;
        }

        // Choose a move (for simplicity, just take the first safe move)
        let chosen_move = safe_moves[0];
        let new_head = simulate_move(&snake.head, &chosen_move);

        // Create a simulated snake for the next move
        let mut simulated_snake = Battlesnake {
            id: snake.id.clone(),
            name: snake.name.clone(),
            health: snake.health - 1, // Decrease health by 1 each turn
            body: snake.body.clone(),
            head: new_head.clone(),
            length: snake.length,
            latency: snake.latency.clone(),
            shout: snake.shout.clone(),
        };

        // Update the head to the new position
        simulated_snake.head = new_head.clone();
        simulated_snake.body.insert(0, new_head.clone());

        // Check if the snake ate food
        let mut ate_food = false;
        for (i, food) in simulated_board.food.iter().enumerate() {
            if coords_equal(&new_head, food) {
                ate_food = true;
                // Remove food from the simulated board
                simulated_board.food.remove(i);
                // Increase health to 100 when eating food
                simulated_snake.health = 100;
                // Increase length by 1
                simulated_snake.length += 1;
                break;
            }
        }

        // If the snake didn't eat food, remove the tail
        if !ate_food && simulated_snake.body.len() > simulated_snake.length as usize {
            simulated_snake.body.pop();
        }

        // Add the simulated snake to the simulated board
        simulated_board.snakes.push(simulated_snake);
    }

    // Check for head-to-head collisions
    let mut head_positions = HashMap::new();
    for snake in &simulated_board.snakes {
        let entry = head_positions.entry((snake.head.x, snake.head.y)).or_insert_with(Vec::new);
        entry.push(snake.clone());
    }

    // Remove snakes that lost head-to-head collisions
    let mut snakes_to_remove = Vec::new();
    for (_, colliding_snakes) in &head_positions {
        if colliding_snakes.len() > 1 {
            // Find the longest snake
            let mut max_length = 0;
            for snake in colliding_snakes {
                if snake.length > max_length {
                    max_length = snake.length;
                }
            }

            // Mark shorter snakes for removal
            for snake in colliding_snakes {
                if snake.length < max_length {
                    snakes_to_remove.push(snake.id.clone());
                }
            }
        }
    }

    // Remove the marked snakes
    simulated_board.snakes.retain(|snake| !snakes_to_remove.contains(&snake.id));

    // Check if our snake is still alive
    let our_snake = simulated_board.snakes.iter().find(|snake| snake.id == you.id);
    if our_snake.is_none() {
        return 0; // Our snake died
    }

    // For each possible next move, recursively evaluate
    let possible_moves = [Move::Up, Move::Down, Move::Left, Move::Right];
    let mut safe_next_moves = 0;
    let mut total_safety = 0;

    for &next_move in &possible_moves {
        let next_pos = simulate_move(&our_snake.unwrap().head, &next_move);

        // Skip if the move is out of bounds
        if next_pos.x < 0 || next_pos.x >= board.width || next_pos.y < 0 || next_pos.y >= board.height {
            continue;
        }

        // Skip if the move collides with any snake's body (except the tail which will move)
        let mut collision = false;
        for snake in &simulated_board.snakes {
            for (i, body_part) in snake.body.iter().enumerate() {
                if i == snake.body.len() - 1 && snake.body.len() > 1 {
                    // Skip the tail if it will move
                    let mut tail_will_move = true;
                    for food in &simulated_board.food {
                        if coords_equal(&snake.head, food) {
                            tail_will_move = false;
                            break;
                        }
                    }
                    if tail_will_move {
                        continue;
                    }
                }
                if body_part.x == next_pos.x && body_part.y == next_pos.y {
                    collision = true;
                    break;
                }
            }
            if collision {
                break;
            }
        }

        if !collision {
            safe_next_moves += 1;
            total_safety += evaluate_move_safety(&simulated_board, our_snake.unwrap(), &next_move, depth - 1);
        }
    }

    // If there are no safe next moves, this is a dead end
    if safe_next_moves == 0 {
        return 0;
    }

    total_safety
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

    // Evaluate moves with look-ahead
    let possible_moves = [Move::Up, Move::Down, Move::Left, Move::Right];
    let mut move_safety_scores = HashMap::new();

    for &move_dir in &possible_moves {
        // Skip moves that are already marked as unsafe
        if !is_move_safe.get(&move_dir).unwrap_or(&false) {
            continue;
        }

        // Evaluate the safety of this move looking ahead 3 moves
        let safety_score = evaluate_move_safety(board, you, &move_dir, 3);
        move_safety_scores.insert(move_dir, safety_score);
    }

    // Filter out moves with a safety score of 0
    for (move_dir, score) in &move_safety_scores {
        if *score == 0 {
            is_move_safe.insert(*move_dir, false);
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

    // Sort safe moves by their safety score (higher is better)
    let mut scored_safe_moves = safe_moves.clone();
    scored_safe_moves.sort_by(|a, b| {
        let a_score = move_safety_scores.get(a).unwrap_or(&0);
        let b_score = move_safety_scores.get(b).unwrap_or(&0);
        b_score.cmp(a_score) // Descending order
    });

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
