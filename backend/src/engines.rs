pub mod first_par;
pub mod first;
pub mod first_trans;

pub use first::First;
pub use first_par::FirstPar;
pub use first_trans::FirstTrans;

use crate::types::{Piece, Position, CastleStatus};

pub struct MoveStatus {
    piece: Piece,
    capture: Piece,
    en_passant: Option<Position>,
    castle_status: Option<CastleStatus>
}

impl MoveStatus {
    fn default() -> MoveStatus {
        MoveStatus { piece: 0, capture: 0, en_passant: None, castle_status: None }
    }
}