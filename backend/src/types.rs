use rocket::response::status;
use rand::seq::SliceRandom;
use std::ops::{Index, IndexMut};
pub type BadRequest = status::BadRequest<&'static str>;

pub type Position = (usize, usize);
pub type IPos = (i8, i8);
pub type Move = (Position, Position);
pub type Score = f64;
pub type Player = i8;
pub type Piece = i8;

pub const PAWN: Piece = 1;
pub const KNIGHT: Piece = 2;
pub const BISHOP: Piece = 3;
pub const ROOK: Piece = 4;
pub const QUEEN: Piece = 5;
pub const KING: Piece = 6;
//pub const UPPER_RIGHT: Position = (7, 7);
//pub const LOWER_RIGHT: Position = (7, 0);
//pub const UPPER_LEFT: Position = (0, 7);
//pub const LOWER_LEFT: Position = (0, 0);
pub const SCORE_ERR: Score = 0.001;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Board(pub [[Piece; 8]; 8]);

#[derive(Clone, Debug)]
pub struct CastleStatus {
    pub white: PlayerCastleStatus,
    pub black: PlayerCastleStatus
}

#[derive(Clone, Debug)]
pub struct PlayerCastleStatus {
    pub k: bool,
    pub q: bool
}

impl CastleStatus {
    pub fn default(status: bool) -> CastleStatus {
        CastleStatus {white: PlayerCastleStatus {k: status, q: status}, black: PlayerCastleStatus {k: status, q: status}}
    }
}

impl PlayerCastleStatus {
    pub fn disable(&mut self) {
        self.k = false;
        self.q = false;
    }
    pub fn any(&self) -> bool {
        self.k || self.q
    }
}

impl Index<Player> for CastleStatus {
    type Output = PlayerCastleStatus;

    fn index(&self, player: Player) -> &Self::Output {
        if player.is_white() {
            &self.white
        } else {
            &self.black
        }
    }
}

impl IndexMut<Player> for CastleStatus {
    fn index_mut(&mut self, player: Player) -> &mut Self::Output {
        if player.is_white() {
            &mut self.white
        } else {
            &mut self.black
        }
    }
}

impl Index<char> for CastleStatus {
    type Output = bool;

    fn index(&self, ch: char) -> &Self::Output {
        match ch {
            'K' => {
                &self.white.k
            }, 
            'Q' => {
                &self.white.q
            }, 
            'k' => {
                &self.black.k
            }, 
            'q' => {
                &self.black.q
            },
            _ => {
                panic!("Castle index must be either 'K', 'Q', 'k' or 'q'. Got {}.", ch)
            }
        }
    }
}

impl IndexMut<char> for CastleStatus {
    fn index_mut(&mut self, ch: char) -> &mut Self::Output {
        match ch {
            'K' => {
                &mut self.white.k
            }, 
            'Q' => {
                &mut self.white.q
            }, 
            'k' => {
                &mut self.black.k
            }, 
            'q' => {
                &mut self.black.q
            },
            _ => {
                panic!("Castle index must be either 'K', 'Q', 'k' or 'q'. Got {}.", ch)
            }
        }
    }
}

pub struct FenData {
    pub board: Board,
    pub player: Player,
    pub can_castle: CastleStatus,
    pub en_passant: Option<Position>,
    pub half_moves: usize,
    pub move_number: usize
}


//
//  Traits
//
pub trait Engine {
    fn new(fen: &str, depth: usize) -> Result<Self, BadRequest> where Self: Sized;

    fn get_best_moves(&mut self) -> (Vec<Move>, Score);

    fn choose_move(&self, best_moves: Vec<Move>) -> Move {
        *best_moves.choose(&mut rand::thread_rng()).unwrap()
    }

    fn make_move(&mut self) -> (Option<Move>, Score) {
        let (best_moves, score) = self.get_best_moves();
        if best_moves.is_empty() {
            return (None, score);
        }
        (Some(self.choose_move(best_moves)), score)
    }
}

impl IntoIterator for Board {
    type Item = [Piece; 8];

    type IntoIter = std::array::IntoIter<Self::Item, 8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Index<Position> for Board {
    type Output = Piece;

    fn index(&self, (x, y): Position) -> &Self::Output {
        &self.0[y][x]
    }
}

impl Index<usize> for Board {
    type Output = [Piece; 8];

    fn index(&self, y: usize) -> &Self::Output {
        &self.0[y]
    }
}

impl IndexMut<Position> for Board {
    fn index_mut(&mut self, (x, y): Position) -> &mut Self::Output {
        &mut self.0[y][x]
    }
}

impl IndexMut<usize> for Board {
    fn index_mut(&mut self, y: usize) -> &mut Self::Output {
        &mut self.0[y]
    }
}

pub trait IsPiece {
    fn value(&self) -> Score;
    fn value_unsigned(&self) -> Score {
        self.value().abs()
    }
    fn is(&self, piece_type: Piece) -> bool;
    fn is_white(&self) -> bool;
    fn is_empty(&self) -> bool;
}

impl IsPiece for Piece {
    fn is(&self, piece_type: Piece) -> bool {
        self.abs() == piece_type
    }

    fn is_white(&self) -> bool {
        self.is_positive()
    }

    fn is_empty(&self) -> bool {
        *self == 0
    }
    fn value(&self) -> Score {
        let value = match self.abs() {
            1 => 1.0,
            2 | 3 => 3.0,
            4 => 5.0,
            5 => 9.0,
            6 => 1000.0,
            _ => 0.0
        };
        value * self.signum() as f64
    }
}

pub trait PlayerTrait {
    fn can_capture(&self, piece: Piece) -> bool;
    fn can_move(&self, piece: Piece) -> bool;
    fn can_control(&self, piece: Piece) -> bool;
    fn back_rank(&self) -> usize;
    fn pawn_base(&self) -> usize;
    fn pawn_move(&self, y: usize) -> usize;
}

impl PlayerTrait for Player {
    fn can_capture(&self, piece: Piece) -> bool {
        piece.signum() == *self * -1
    }
    fn can_move(&self, piece: Piece) -> bool {
        self.can_capture(piece) || piece == 0
    }
    fn can_control(&self, piece: Piece) -> bool {
        *self == piece.signum()
    }
    fn back_rank(&self) -> usize {
        if *self == 1 {
            0
        } else {
            7
        }
    }
    fn pawn_base(&self) -> usize {
        if *self == 1 {
            1
        } else {
            6
        }
    }
    fn pawn_move(&self, y: usize) -> usize {
        (y as Player + *self) as usize
    }
}

pub trait BoardIndex {
    fn is_valid(&self) -> bool;
}

impl BoardIndex for usize {
    fn is_valid(&self) -> bool {
        *self < 8 
    }
}

impl BoardIndex for i8 {
    fn is_valid(&self) -> bool {
        *self >= 0 && *self < 8
    }
}

impl BoardIndex for isize {
    fn is_valid(&self) -> bool {
        *self >= 0 && *self < 8
    }
}

pub fn _to_index(i: i8) -> Option<usize> {
    if i.is_valid() {
        return Some(i as usize)
    }
    None
}

pub fn sub_pos(p1: Position, p2: Position) -> IPos {
    (p1.0 as i8 - p2.0 as i8, p1.1 as i8 - p2.1 as i8)
}

pub trait PosTrait {
    type Content;
    fn add(&self, other: Self) -> Self;
    fn mul(&self, k: Self::Content) -> Self;
    fn quote(&self) -> f32;
    fn is_axis(&self) -> bool;
    fn map(&self, f: fn(Self::Content) -> Self::Content) -> (Self::Content, Self::Content);
    fn to_valid(&self) -> Option<Position>;
}

impl PosTrait for Position {
    type Content = usize;
    fn add(&self, other: Self) -> Self {
        (self.0 + other.0, self.1 + other.1)
    }
    fn quote(&self) -> f32 {
        self.0 as f32 / self.1 as f32
    }
    fn is_axis(&self) -> bool {
        self.0 * self.1 == 0
    }
    fn map(&self, f: fn(Self::Content) -> Self::Content) -> (Self::Content, Self::Content) {
        (f(self.0), f(self.1))
    }
    fn to_valid(&self) -> Option<Position> {
        if self.0.is_valid() && self.1.is_valid() {
            return Some((self.0, self.1))
        }
        None
    }
    fn mul(&self, k: Self::Content) -> Self {
        (self.0 * k, self.1 * k)
    }
}

impl PosTrait for IPos {
    type Content = i8;
    fn add(&self, other: Self) -> Self {
        (self.0 + other.0, self.1 + other.1)
    }
    fn quote(&self) -> f32 {
        self.0 as f32 / self.1 as f32
    }
    fn is_axis(&self) -> bool {
        self.0 * self.1 == 0
    }
    fn map(&self, f: fn(Self::Content) -> Self::Content) -> (Self::Content, Self::Content) {
        (f(self.0), f(self.1))
    }
    fn to_valid(&self) -> Option<Position> {
        if self.0.is_valid() && self.1.is_valid() {
            return Some((self.0 as usize, self.1 as usize))
        }
        None
    }
    fn mul(&self, k: Self::Content) -> Self {
        (self.0 * k, self.1 * k)
    }
}