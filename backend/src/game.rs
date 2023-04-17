use crate::{types::*, break_block};

use std::cmp::min;
use itertools::Itertools;

const KNIGHT_COMBINATIONS: [(i8, i8); 8] = [(2, 1), (2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2), (-2, 1), (-2, -1)];

impl Board {
    pub fn get_king_pos(&self, player: Player) -> Position {
        all_coords().find(|&pos| self[pos] == KING * player).unwrap()
    }

    // Check whether a square is checked by a piece 
    pub fn is_checked(&self, pos: Position, player: Player) -> bool {
        let (x, y) = pos;
        let other = -player;
        
        // Test pawn (Also checks for bishops, queens and king)
        for pos in [-1, 1].into_iter().filter_map(|k| (x as i8 + k, y as i8 + player).to_valid()) {
            if matches!(other * self[pos], PAWN | BISHOP | QUEEN | KING) {
                return true;
            }
        }

        // Test knight
        if KNIGHT_COMBINATIONS
            .iter()
            .filter_map(|&(k1, k2)| (x as i8 + k1, y as i8 + k2).to_valid())
            .any(|pos| self[pos] == KNIGHT * other) {
                return true;
        }

        // Test bishop + queen
        for iterator in get_all_bishop_moves(pos) {
            for to in iterator {
                if matches!(other * self[to], BISHOP | QUEEN ) {
                    return true;
                } else if self[to] != 0 {
                    break;
                }
            }
        }
        
        // Test rook + queen
        for iterator in get_all_rook_moves(pos) {
            for to in iterator {
                if matches!(other * self[to], ROOK | QUEEN) {
                    return true;
                } else if self[to] != 0 {
                    break;
                }
            }
        }
        
        // Test king
        if get_all_king_moves(pos).any(|pos| self[pos] == KING * other) {
            return true;
        }

        false
    }

    // Determine the lowest valued piece that are attacking / defending a certain square
    pub fn check_pos(&self, pos: Position, mut player: Player, defender: bool) -> Score {
        let (x, y) = pos;
        if defender {
            player *= -1;
        }
        let other = -player;
        // Test pawn
        for pos in [-1, 1].into_iter().filter_map(|k| (x as i8 + k, y as i8 + player).to_valid()) {
            if self[pos] == PAWN * other {
                return PAWN.value();
            }
        }
        // Test knight
        if KNIGHT_COMBINATIONS
            .iter()
            .filter_map(|&(k1, k2)| (x as i8 + k1, y as i8 + k2).to_valid())
            .any(|pos| self[pos] == KNIGHT * other) {
                return KNIGHT.value();
        }

        let mut queen = false;

        // Test bishop + rook
        for target in [BISHOP, ROOK] {
            let all_moves = match target {
                BISHOP => get_all_bishop_moves(pos),
                _ => get_all_rook_moves(pos)
            };
            for iterator in all_moves.into_iter() {
                for to in iterator {
                    let piece = self[to] * other;
                    if piece == target {
                        return target.value();
                    } else if piece != 0 {
                        if piece == QUEEN {
                            queen = true;
                        }
                        break;
                    }
                }
            }
        }

        if queen {
            return QUEEN.value();
        }

        // Test king
        if get_all_king_moves(pos).any(|pos| self[pos] == KING * other) {
            return KING.value();
        }
        0.0
    }

    pub fn get_valid(&mut self, player: Player, castle_status: &CastleStatus, en_passant: Option<Position>) -> Vec<Move> {
        let mut moves = Vec::new();
        let king_pos = self.get_king_pos(player);
        let in_check = self.is_checked(king_pos, player);
        for from in all_coords().filter(|&pos| player.can_control(self[pos])) {
            let (x, y) = from;
            let piece = self[from];
            match piece.abs() {
                PAWN => {
                    let y1 = player.pawn_move(y);
                    let y2 = player.pawn_move(y1);
                    // Move forward
                    if self[y1][x] == 0 {
                        moves.push((from, (x, y1)));
                        if y == player.pawn_base() && self[y2][x] == 0 {
                            moves.push((from, (x, y2)));
                        }
                    }
                    // Capture right
                    if x < 7 && player.can_capture(self[y1][x+1]) {
                        moves.push((from, (x+1, y1)));
                    }
                    // Capture left
                    if x > 0 && player.can_capture(self[y1][x-1]) {
                        moves.push((from, (x-1, y1)));
                    }
                    // En passant
                    if let Some(pos) = en_passant {
                        if pos.1 == y1 && (pos.0 == x+1 || pos.0+1 == x) {
                            moves.push((from, pos))
                        }
                    }
                },
                KNIGHT => {
                    let knight_moves = 
                        KNIGHT_COMBINATIONS.iter()
                        .filter_map(|&(k1, k2)| (x as i8 + k1, y as i8 + k2).to_valid())
                        .filter(|&to| player.can_move(self[to]))
                        .map(|to| (from, to));

                    moves.extend(knight_moves);
                },
                BISHOP => {
                    self.get_bishop_moves(&mut moves, player, from);
                },
                ROOK => {
                    self.get_rook_moves(&mut moves, player, from);
                },
                QUEEN => {
                    self.get_bishop_moves(&mut moves, player, from);
                    self.get_rook_moves(&mut moves, player, from);
                },
                KING => break_block!({
                    for to in get_all_king_moves(from) {
                        // Normal king move
                        if player.can_move(self[to]) {
                            moves.push((from, to));
                        }
                    }

                    if self.is_checked(from, player) {
                        break;
                    }

                    // Castle king side
                    if castle_status[player].k && self[y][x+1] == 0 && self[y][x+2] == 0 && !self.is_checked((x+1, y), player) {
                        moves.push((from, (x+2, y)));
                    }
                    // Castle queen side
                    if castle_status[player].q && self[y][x-1] == 0 && self[y][x-2] == 0 && self[y][x-3] == 0 && !self.is_checked((x-1, y), player) {
                        moves.push((from, (x-2, y)));
                    }
                }),
                _ => {}
            }
        }

        moves.into_iter().filter(|&(from, to)| {
            if in_check || self[from].is(KING) {
                let capture = self[to];
                self[to] = self[from];
                self[from] = 0;
                let valid = if self[to].is(KING) {
                    !self.is_checked(to, player)
                } else {
                    !self.is_checked(king_pos, player)
                };
                self[from] = self[to];
                self[to] = capture;
                valid
            } else {
                !self.smart_checked(player, king_pos, (from, to)) // Doesn't work if king is in check!!
            }
        }).collect()
    }

    fn get_bishop_moves(&self, moves: &mut Vec<Move>, player: Player, from: Position) {
        let (x, y) = from;
        let iterators: [Vec<Position>; 4] = [
            (1..=min(x, y)).map(|k| (x-k, y-k)).collect(),      // Left down
            (1..=min(x, 7-y)).map(|k| (x-k, y+k)).collect(),    // Left up
            (1..=min(7-x, y)).map(|k| (x+k, y-k)).collect(),    // Right down
            (1..=min(7-x, 7-y)).map(|k| (x+k, y+k)).collect()   // Right up
        ];
        for iterator in iterators {
            for to in iterator {
                if !player.can_move(self[to]) {
                    break;
                } 
                moves.push((from, to));
                if player.can_capture(self[to]) {
                    break;
                }
            }
        }
    }

    fn get_rook_moves(&self, moves: &mut Vec<Move>, player: Player, from: Position) {
        let (x, y) = from; 
        let iterators: [Vec<Position>; 4] = [
            (0..x).map(|x_new| (x_new, y)).rev().collect(), // Move left
            (x+1..8).map(|x_new| (x_new, y)).collect(),     // Move right
            (0..y).map(|y_new| (x, y_new)).rev().collect(), // Move down
            (y+1..8).map(|y_new| (x, y_new)).collect()      // Move up
        ];
        for iterator in iterators {
            for to in iterator {
                if !player.can_move(self[to]) {
                    break;
                } 
                moves.push((from, to));
                if player.can_capture(self[to]) {
                    break;
                }
            }
        }        
    }

    pub fn count_material(&self) -> Score {
        self.0.iter().flat_map(|row| row.iter().map(IsPiece::value)).sum()
    }

    fn smart_checked(&self, player: Player, pos: Position, (from, to): Move) -> bool {
        let other = -player;
        let eps = 0.001;
        let dif1 = sub_pos(from, pos);
        let q1 = dif1.quote();
        if !(dif1.is_axis() || (q1.abs() - 1.0).abs() < eps) {
            return false
        }
        let dif2 = sub_pos(to, pos);
        let q2 = dif2.quote();
        if (q1 - q2).abs() < eps {
            return false;
        }
        let from_i = (from.0 as i8, from.1 as i8);
        let dif = dif1.map(num::signum);
        //println!("Checking move: {from:?} -> {to:?}");
        for p in (1..7).filter_map(|k| from_i.add(dif.mul(k)).to_valid()) {
            let target = match dif.is_axis() {
                true => ROOK,
                false => BISHOP
            };
            let piece = self[p] * other;
            if piece == target || piece == QUEEN {
                return true;
            } else if piece != 0 {
                break;
            }
        }

        false
    }
}

fn get_all_king_moves((x, y): Position) -> impl Iterator<Item = Position> {
    let get_range = |i: usize| i.saturating_sub(1)..=min(7, i+1);
    get_range(x).cartesian_product(get_range(y))
}

fn get_all_bishop_moves((x, y): Position) -> [Vec<Position>; 4] {
    [
        (1..=min(x, y)).map(|k| (x-k, y-k)).collect(),      // Left down
        (1..=min(x, 7-y)).map(|k| (x-k, y+k)).collect(),    // Left up
        (1..=min(7-x, y)).map(|k| (x+k, y-k)).collect(),    // Right down
        (1..=min(7-x, 7-y)).map(|k| (x+k, y+k)).collect()   // Right up
    ]
}

fn get_all_rook_moves((x, y): Position) -> [Vec<Position>; 4] {
    [
        (0..x).map(|x_new| (x_new, y)).rev().collect(), // Move left
        (x+1..8).map(|x_new| (x_new, y)).collect(),     // Move right
        (0..y).map(|y_new| (x, y_new)).rev().collect(), // Move down
        (y+1..8).map(|y_new| (x, y_new)).collect()      // Move up
    ]
}

pub fn all_coords() -> impl Iterator<Item = Position> {
    (0..8).cartesian_product(0..8)
}