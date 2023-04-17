use std::ops::Index;

use crate::{types::Position, byte_board::{Piece::*, PieceType::*}};

enum Piece {
    White(PieceType),
    Black(PieceType),
    Empty
}

enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

impl Piece {
    fn to_bits(&self) -> u16 {
        match self {
            White(piece) => match piece {
                Pawn => 0x1,
                Knight => 0x2,
                Bishop => 0x3,
                Rook => 0x4,
                Queen => 0x5,
                King => 0x6
            },
            Black(piece) => match piece {
                Pawn => 0x7,
                Knight => 0x8,
                Bishop => 0x9,
                Rook => 0xa,
                Queen => 0xb,
                King => 0xc
            },
            Empty => 0
        } 
    }
}

enum Piece2 {
    PawnW,
    KnightW,
    BishopW,
    RookW,
    QueenW,
    KingW,
    PawnB,
    KnightB,
    BishopB,
    RookB,
    QueenB,
    KingB
}

struct BitBoard(u16);

impl BitBoard {
    fn get(&self, pos: Position) -> u16 {
        (self.0 & pos_to_mask(pos)) >> (16*pos.1 + pos.0)
    }

    fn set(&mut self, pos: Position, piece: Piece) {
        self.0 |= pos_to_mask(pos);
        self.0 &= piece.to_bits() << (16*pos.1 + pos.0);
    }
}

fn pos_to_mask((x, y): Position) -> u16 {
    0xf << (16*y + x)
}