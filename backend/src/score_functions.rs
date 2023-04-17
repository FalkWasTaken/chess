use crate::types::*;
use crate::game::*;
use crate::utils::cmp_scores;

pub fn score1(board: &Board, player: Player) -> Score {
    let mut score = board.count_material();
    let max_undefended = all_coords().filter(|&pos| player.can_control(board[pos])).filter_map(|pos| {
        let piece = board[pos].value_unsigned();
        let attacker = board.check_pos(pos, player, false);
        let defender = board.check_pos(pos, player, true);
        if is_unprotected(piece, attacker, defender) {
            let dif = if defender.abs() > SCORE_ERR {attacker} else {0.0};
            Some(piece - dif)
        } else {
            None
        }
    }).max_by(cmp_scores).unwrap_or(0.0);
    score += bad_placement(board);
    score - max_undefended * player as Score
}

fn is_unprotected(piece: Score, attacker: Score, defender: Score) -> bool {
    attacker.abs() > SCORE_ERR && (defender.abs() < SCORE_ERR || attacker < piece)
}

fn bad_placement(board: &Board) -> Score {
    let mut res = 0.0;
    for pos in all_coords().filter(|&pos| board[pos] != 0) {
        let (x, y) = pos;
        let piece = board[pos];
        let penalty = piece.signum() as Score * 0.1;
        match piece.abs() {
            KNIGHT if x == 0 || x == 7 => {
                res -= penalty;
            },
            BISHOP | KNIGHT if y == 0 || y == 7 => {
                res -= penalty;
            }
            ROOK => {
                for k in 0..8 {
                    if board[k][x].is(PAWN) {
                        res -= penalty;
                    }
                }
            },
            _ => {}
        }
    }
    res
}