use crate::types::*;
use crate::utils::*;
use super::MoveStatus;

use itertools::Itertools;
use crate::break_block;
use crate::score_functions::*;

#[derive(Clone, Debug)]
pub struct First {
    board: Board,
    player: Player,
    can_castle: CastleStatus,
    en_passant: Option<Position>,
    depth: usize,
    pub num_leafs: usize
}

impl Engine for First {
    fn new(fen: &str, depth: usize) -> Result<Self, BadRequest> {
        let fen_data = parse_fen(fen)?;
    
        Ok(First {
            board: fen_data.board,
            player: fen_data.player,
            can_castle: fen_data.can_castle,
            en_passant: fen_data.en_passant,
            depth,
            num_leafs: 0
        })
    }

    fn get_best_moves(&mut self) -> (Vec<Move>, Score) {
        let mut best_moves = Vec::new();
        let mut best = Score::NEG_INFINITY * self.player as Score;
        let valid_moves = self.board.get_valid(self.player, &self.can_castle, self.en_passant);
        println!("Number of initial moves: {}", valid_moves.len());
        println!("Progress: [{}]", (0..valid_moves.len()).map(|_| "-").collect::<String>());
        print!("           ");
        for mv in valid_moves.into_iter().sorted_unstable_by_key(|&(_, to)| -self.board[to].value_unsigned() as isize) {
            print!("#");
            let move_status = self.do_move(mv);
            let score = self.recursive_best(Score::NEG_INFINITY, Score::INFINITY);
            self.undo_move(mv, move_status);
            if score.is_none() {
                println!("Previous error occured after move: {:?} -> {:?}", mv.0, mv.1);
                panic!("Panicing due to previous error!");
            }
            let score = score.unwrap();
            if self.is_white() && score >= best {
                if score > best {
                    best_moves = Vec::new();
                    best = score
                }
                best_moves.push(mv);
            } else if !self.is_white() && score <= best {
                if score < best {
                    best_moves = Vec::new();
                    best = score
                }
                best_moves.push(mv);
            }
        }
        println!();
        (best_moves, best)
    }
}



impl First {
    fn is_white(&self) -> bool {
        self.player == 1
    }

    fn do_move(&mut self, (from, to): Move) -> MoveStatus {
        let mut status = MoveStatus::default();
        status.capture = self.board[to];
        let piece = self.board[from];
        self.board[to] = piece;
        self.board[from] = 0;
        status.piece = piece;
        let player = self.player;

        // Reset en passant square and store previous value in status
        if let Some(en_passant) = self.en_passant {
            status.en_passant = Some(en_passant);
            self.en_passant = None;
        }

        // Handle special cases
        match piece.abs() {
            PAWN => 'pawn: {
                // Update en passant square
                if from.1.abs_diff(to.1) == 2 {
                    self.en_passant = Some((from.0, player.pawn_move(from.1)));
                    break 'pawn;
                }

                // Handle promotion
                if to.1 == 0 || to.1 == 7 {
                    self.board[to] = QUEEN * player;
                }
            },
            ROOK => {
                // Check castle rights at rook move
                if self.can_castle[player].k && from == (7, player.back_rank()) {
                    status.castle_status = Some(self.can_castle.clone());
                    self.can_castle[player].k = false; 
                } else if self.can_castle[player].q && from == (0, player.back_rank()) {
                    status.castle_status = Some(self.can_castle.clone());
                    self.can_castle[player].q = false;
                }
            },
            KING => {
                // If a king with castle rights moves, store castle status in status
                if self.can_castle[player].any() {
                    status.castle_status = Some(self.can_castle.clone());
                    self.can_castle[player].disable();
                }
                // Castle
                if from.0 + 2 == to.0 {
                    // King side
                    self.board[from.1][from.0+1] = ROOK * player;
                    self.board[from.1][7] = 0;
                } else if from.0 == to.0 + 2 {
                    // Queen side
                    self.board[from.1][from.0-1] = ROOK * player;
                    self.board[from.1][0] = 0;
                }
            },
            _ => {}
        }

        self.depth -= 1;
        self.player *= -1;
        status

    }

    fn undo_move(&mut self, (from, to): Move, status: MoveStatus) {
        self.board[from] = status.piece;
        self.board[to] = status.capture;
        self.en_passant = status.en_passant;
        if let Some(castle_status) = status.castle_status {
            self.can_castle = castle_status;
        }

        self.player *= -1;
        self.depth += 1;

        // Handle castle
        if status.piece.is(KING) && from.0.abs_diff(to.0) == 2 {
            if from.0 < to.0 {
                // King side castle
                self.board[from.1][7] = ROOK * self.player;
                self.board[from.1][5] = 0;
            } else {
                // Queen side castle
                self.board[from.1][0] = ROOK * self.player;
                self.board[from.1][3] = 0;
            }
        }

        
    }

    fn score_function(&self) -> Score {
        score1(&self.board, -self.player)
    }

    fn recursive_best(&mut self, mut alpha: Score, mut beta: Score) -> Option<Score> {
        if self.depth == 0 {
            self.num_leafs += 1;
            return Some(self.score_function());
        }
        let valid_moves = self.board.get_valid(self.player, &self.can_castle, self.en_passant);
        let mut best = -100.0 * self.player as Score;
        let king_pos = self.board.get_king_pos(self.player);
        if valid_moves.is_empty() {
            if self.board.is_checked(king_pos, self.player) {
                return Some(best - self.depth as Score * self.player as Score);
            }
            return Some(0.0);
        }  
        for mv in valid_moves.into_iter().sorted_unstable_by_key(|&(_, to)| -self.board[to].value_unsigned() as isize) {
            if self.board[mv.1].is(KING) {
                println!("King capture!");
                println!("Engine: {:?}", self);
                println!("Move: {:?} -> {:?}", mv.0, mv.1);
                return None;
            }
            let move_status = self.do_move(mv);
            let score = self.recursive_best(alpha, beta);
            if score.is_none() {
                println!("Previous error occured after move: {:?} -> {:?}", mv.0, mv.1);
                return None;
            }
            let score = score.unwrap();
            self.undo_move(mv, move_status);
            if self.is_white() {
                best = max_score(best, score);
                if best >= beta {
                    break;
                }
                alpha = max_score(best, alpha);
            } else {
                best = min_score(best, score);
                if best <= alpha {
                    break;
                }
                beta = min_score(best, beta);
            }
        }
        Some(best)
    }
}

