use crate::types::*;
use crate::utils::*;
use super::MoveStatus;

use itertools::Itertools;
use rayon::prelude::*;
//use dashmap::DashMap;
//use chashmap::CHashMap;

use crate::break_block;
use crate::score_functions::*;
use std::collections::HashMap;
use rand::prelude::SliceRandom;

#[derive(Clone, Debug)]
pub struct FirstPar {
    board: Board,
    player: Player,
    can_castle: CastleStatus,
    en_passant: Option<Position>,
    depth: usize,
    pub num_leafs: usize,
    pub num_skips: usize
}

impl Engine for FirstPar {
    fn new(fen: &str, depth: usize) -> Result<Self, BadRequest> {
        let fen_data = parse_fen(fen)?;
    
        Ok(FirstPar {
            board: fen_data.board,
            player: fen_data.player,
            can_castle: fen_data.can_castle,
            en_passant: fen_data.en_passant,
            depth,
            num_leafs: 0,
            num_skips: 0
        })
    }

    fn choose_move(&self, best_moves: Vec<Move>) -> Move {
        if let Some(mv) = best_moves.iter().find(|&&(from, to)| self.board[from].is(KING) && from.0.abs_diff(to.0) == 2) {
            return *mv;
        }
        let king_pos = self.board.get_king_pos(-self.player);
        let dist_to_king = |pos: Position| pos.0.abs_diff(king_pos.0).max(pos.1.abs_diff(king_pos.1));
        let closer: Vec<Move> = best_moves.clone().into_iter().filter(|&(from, to)| dist_to_king(from) > dist_to_king(to)).collect();
        if !closer.is_empty() {
            *closer.choose(&mut rand::thread_rng()).unwrap()
        } else {
            *best_moves.choose(&mut rand::thread_rng()).unwrap()
        }
    }

    fn get_best_moves(&mut self) -> (Vec<Move>, Score) {
        let mut valid_moves = self.board.get_valid(self.player, &self.can_castle, self.en_passant);
        valid_moves.par_sort_unstable_by_key(|&(_, to)| -self.board[to].value_unsigned() as isize);
        let moves: Vec<(Move, Score, usize, usize)> = valid_moves.into_par_iter().map(|mv| {
            let mut clone = self.clone();
            clone.do_move(mv);
            let score = clone.recursive_best(Score::NEG_INFINITY, Score::INFINITY, &mut HashMap::new());
            (mv, score, clone.num_leafs, clone.num_skips)
        }).collect();
        let mut best = Vec::new();
        let mut best_score = Score::NEG_INFINITY * self.player as Score;
        for (mv, score, num_leafs, num_skips) in moves {
            self.num_leafs += num_leafs;
            self.num_skips += num_skips;
            if (self.is_white() && score >= best_score) || (!self.is_white() && score <= best_score) {
                if (self.is_white() && score > best_score) || (!self.is_white() && score < best_score) {
                    best_score = score;
                    best = Vec::new();
                }
                best.push(mv);
            }
        }
        println!("Num skips: {}", self.num_skips);
        (best, best_score)
    }
}

impl FirstPar {
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
            PAWN => break_block!({
                // Update en passant square
                if from.1.abs_diff(to.1) == 2 {
                    self.en_passant = Some((from.0, player.pawn_move(from.1)));
                    break;
                }

                // Handle promotion
                if to.1 == 0 || to.1 == 7 {
                    self.board[to] = QUEEN * player;
                }
            }),
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

    fn recursive_best(&mut self, mut alpha: Score, mut beta: Score, transpositions: &mut HashMap<Board, Score>) -> Score {
        if self.depth == 0 {
            self.num_leafs += 1;
            return self.score_function();
        }
        let valid_moves = self.board.get_valid(self.player, &self.can_castle, self.en_passant);
        let mut best = Score::NEG_INFINITY * self.player as Score;
        let king_pos = self.board.get_king_pos(self.player);
        if valid_moves.is_empty() {
            if self.board.is_checked(king_pos, self.player) {
                return best;
            }
            return 0.0;
        }  
        for mv in valid_moves.into_iter().sorted_unstable_by_key(|&(_, to)| -self.board[to].value_unsigned() as isize) {
            let move_status = self.do_move(mv);
            let score = match transpositions.get(&self.board) {
                Some(score) => {
                    self.num_skips += 1;
                    *score
                },
                None => {
                    let score = self.recursive_best(alpha, beta, transpositions);
                    transpositions.insert(self.board.clone(), score);
                    score
                }
            };
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
        best
    }
}

