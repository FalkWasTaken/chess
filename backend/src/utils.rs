use rocket::response::status;

use crate::types::*;

use std::collections::{HashMap, HashSet};

lazy_static! {
    static ref CHAR_TO_INDEX: HashMap<char, usize> = {
        HashMap::from([
            ('a', 0),
            ('b', 1),
            ('c', 2),
            ('d', 3),
            ('e', 4),
            ('f', 5),
            ('g', 6),
            ('h', 7)
        ])
    };
}

pub const INDEX_TO_CHAR: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];


pub fn _move_to_string((from, to): Move) -> String {
    let from = (INDEX_TO_CHAR[from.0], from.1);
    let to = (INDEX_TO_CHAR[to.0], to.1);
    format!("{}{}, {}{}", from.0, from.1+1, to.0, to.1+1)
}

pub fn pos_to_string((x, y): Position) -> String {
    format!("{}{}", INDEX_TO_CHAR[x], y + 1)
}


pub fn char_to_piece(from: char) -> Result<Piece, &'static str> {    
    let mut piece = match from.to_lowercase().next().unwrap() {
        'p' => 1,
        'n' => 2,
        'b' => 3,
        'r' => 4,
        'q' => 5,
        'k' => 6,
        _ => 0
    };

    if piece == 0 {
        return Err("Error whilst parsing piece");
    }

    if from.is_lowercase() {
        piece *= -1;
    }

    Ok(piece)
}

/*
*   Parse a chess board from a FEN code.
*/
fn board_from_fen(fen: &str) -> Result<Board, BadRequest> {
    let mut board = Board([[0; 8]; 8]);
    for (y, line) in fen.split_terminator('/').enumerate() {
        let mut x = 0;
        for ch in line.chars() {
            match char_to_piece(ch) {
                Ok(piece) => board[7-y][x] = piece,
                Err(error) => {
                    match ch.to_digit(10) {
                        Some(skip) => x += skip as usize - 1,
                        _ => return Err(status::BadRequest(Some(error)))
                    }
                }
            }
            x += 1;
        }
    }
    Ok(board)
}

fn player_from_fen(fen: &str) -> Result<Player, BadRequest> {
    match fen {
        "w" => Ok(1),
        "b" => Ok(-1),
        _ => Err(status::BadRequest(Some("Invalid player field")))
    }
}

fn castle_from_fen(fen: &str) -> Result<CastleStatus, BadRequest> {
    let error = status::BadRequest(Some("Invalid castle information"));
    let mut status = CastleStatus::default(false);
    if fen == "-" {
        return Ok(status);
    }

    let mut log = HashSet::new();
    for ch in fen.chars() {
        if log.contains(&ch) {
            return Err(error);
        }
        log.insert(ch);
        status[ch] = true;
    }
    Ok(status)
}

fn en_passant_from_fen(fen: &str) -> Result<Option<Position>, BadRequest> {
    if fen == "-" {
        return Ok(None);
    }
    let chars: Vec<char> = fen.chars().collect();
    let x = CHAR_TO_INDEX.get(&chars[0]);
    let y = chars[1].to_digit(10);
    if x.is_none() || y.is_none() {
        return Err(status::BadRequest(Some("Invalid en passant information!")));
    }
    Ok(Some((*x.unwrap(), y.unwrap() as usize)))
}

pub fn parse_fen(fen: &str) -> Result<FenData, BadRequest> {
    // Split parts of FEN
    let fen: Vec<&str> = fen.split_whitespace().collect();
        
    let board = board_from_fen(fen[0])?;
    let player = player_from_fen(fen[1])?;
    let can_castle = castle_from_fen(fen[2])?;
    let en_passant = en_passant_from_fen(fen[3])?;
    let half_moves = fen[4].parse();
    let move_number = fen[5].parse();

    if half_moves.is_err() || move_number.is_err() {
        return Err(status::BadRequest(Some("Error in fen parsing!")));
    }

    let half_moves = half_moves.unwrap();
    let move_number = move_number.unwrap();

    Ok(FenData {
        board,
        player,
        can_castle,
        en_passant,
        half_moves,
        move_number
    })
}

pub fn cmp_scores(a: &Score, b: &Score) -> std::cmp::Ordering {
    a.partial_cmp(b).unwrap()
}

pub fn max_score(a: Score, b: Score) -> Score {
    std::cmp::max_by(a, b, cmp_scores)
}

pub fn min_score(a: Score, b: Score) -> Score {
    std::cmp::min_by(a, b, cmp_scores)
}

pub fn _valid_pos((x, y): Position) -> bool {
    x.is_valid() && y.is_valid()
}

#[macro_export]
macro_rules! break_block {
    ($xs: block) => {
        loop {
            $xs
            break;
        }
    };
}