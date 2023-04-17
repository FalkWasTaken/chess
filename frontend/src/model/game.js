import { createSlice } from "@reduxjs/toolkit"
import _ from "lodash"

import { 
    Pos, 
    validIndex, 
    posBoard, 
    validPos, 
    canCapture,
    canControl,
    canMove,
    eqPos,
    isWhite,
    backRank  
} from "./utils"

const [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING] = [1, 2, 3, 4, 5, 6]

const initialState = {
    board: [
        [ 4,  2,  3,  5,  6,  3,  2,  4],
        [ 1,  1,  1,  1,  1,  1,  1,  1],
        [ 0,  0,  0,  0,  0,  0,  0,  0],
        [ 0,  0,  0,  0,  0,  0,  0,  0],
        [ 0,  0,  0,  0,  0,  0,  0,  0],
        [ 0,  0,  0,  0,  0,  0,  0,  0],
        [-1, -1, -1, -1, -1, -1, -1, -1],
        [-4, -2, -3, -5, -6, -3, -2, -4],
    ],
    player: 1,
    castleK: true,
    castleQ: true,
    enPassant: null,
    castleOther: {
        k: true,
        q: true
    },
    validMoves: [],
    from: null,
    prevMove: null,
    halfmove: 0,
    fullmove: 1,
    computerScore: null
}

const gameSlice = createSlice({
    name: "game",
    initialState,
    reducers: {
        TRY_MOVE: (state, action) => {
            if (action.payload.computerScore != null) {
                state.computerScore = action.payload.computerScore
            }
            const from = action.payload.from ? action.payload.from : state.from
            if (!from) return
            const to = action.payload.to ? action.payload.to : action.payload
            const board = state.board
            const player = state.player
            const piece = Math.abs(board[from.y][from.x])
            if (!canControl(state, from) || !canMove(state, to) || !checkMove(state, {from, to})) {
                state.refresh = !state.refresh
                return
            }

            state.board[to.y][to.x] = state.board[from.y][from.x]
            state.board[from.y][from.x] = 0

            switch (Math.abs(piece)) {
                case PAWN:
                    if (Math.abs(from.y - to.y) == 2) {
                        // Handle double forward
                        state.enPassant = Pos(from.x, from.y + player)
                    } else if (to.y == (isWhite(player) ? 7 : 0)) {
                        // Handle queen promotion
                        state.board[to.y][to.x] = QUEEN * player
                    } else if (state.enPassant && eqPos(to, state.enPassant)) {
                        // Handle en passant
                        state.board[from.y][to.x] = 0
                    }
                    // Reset halfmove timer on pawn move
                    state.halfmove = -1
                    break
                case ROOK:
                    if (from.y != (isWhite(player) ? 0 : 7))
                        break

                    if (state.castleK && from.x == 7)
                        state.castleK = false
                    
                    if (state.castleQ && from.x == 0) 
                        state.castleQ = false
                    
                    break
                case KING:
                    state.castleK = false
                    state.castleQ = false
                    for (const side of [1, -1]) {
                        if (to.x == from.x + 2*side) {
                            state.board[to.y][from.x + side] = ROOK * player
                            state.board[to.y][side == 1 ? 7 : 0] = 0
                        }
                    }
            }

            // Don't reset en passant square if a double pawn move was made
            if (!(Math.abs(piece) == PAWN && Math.abs(from.y - to.y) == 2)) {
                state.enPassant = null
            }

            // Reset halfmove timer on capture
            if (canCapture(state, to)) {
                state.halfmove = -1
            }

            state.player *= -1
            const castleOther = {k: state.castleK, q: state.castleQ}
            state.castleK = state.castleOther.k
            state.castleQ = state.castleOther.q
            state.castleOther = castleOther
            state.validMoves = []
            state.prevMove = {from, to}
            state.halfmove += 1
            if (!isWhite(player))
                state.fullmove += 1
        },
        SET_FROM: (state, action) => {
            const from = action.payload
            state.from = from
            if (!from || !canControl(state, from)) {
                state.validMoves = []
                return
            }
            state.validMoves = getMoves(state, from)
        }
    }
})

function checkMove(state, {from, to}) {
    let moves = getMoves(state, from)
    //console.log("Move: ", {from, to})
    //console.log("Valid moves: ", moves)
    if (!moves.find(pos => eqPos(pos, to))) {
        return false
    }

    const board = posBoard(state.board)
    const capture = board(to)
    state.board[to.y][to.x] = board(from)
    state.board[from.y][from.x] = 0
    const valid = !isKingChecked(state)
    state.board[from.y][from.x] = state.board[to.y][to.x]
    state.board[to.y][to.x] = capture

    return valid
}

function getMoves(state, from) {
    switch (Math.abs(state.board[from.y][from.x])) {
        case PAWN:
            return getPawnMoves(state, from)
        case KNIGHT:
            return getKnightMoves(state, from)
        case BISHOP: 
            return getBishopMoves(state, from)
        case ROOK:
            return getRookMoves(state, from)
        case QUEEN:
            return getQueenMoves(state, from)
        case KING: 
            return getKingMoves(state, from)
    }
}

function getPawnMoves(state, from) {
    const board = posBoard(state.board)
    const player = state.player
    const {x, y} = from

    let moves = []
    // Forward
    const forw = Pos(x, y + player)
    if (validPos(forw) && board(forw) == 0) {
        moves.push(forw)
    }
    // Double forward
    const forw2 = Pos(x, y + 2*player)
    if (from.y == (isWhite(state.player) ? 1 : 6) && validPos(forw2) && board(forw) == 0 && board(forw2) == 0) {
        moves.push(forw2)
    }
    // Capture + en passant
    [1, -1].forEach(k => {
        const to = Pos(x+k, y + player)
        if (validPos(to) && (canCapture(state, to) || (state.enPassant && eqPos(to, state.enPassant)))) {
            moves.push(to)
        }
    })

    return moves
}

function getKnightMoves(state, {x, y}) {
    let moves = []

    const kValues = [-2, -1, 1, 2]

    kValues.forEach(k1 => {
        kValues.forEach(k2 => {
            const to = Pos(x+k1, y+k2)
            if (Math.abs(k1) != Math.abs(k2) && validPos(to) && canMove(state, to))
                moves.push(to)
        })
    })

    return moves
}

function getBishopMoves(state, from) {
    let moves = []
    const {x, y} = from
    const arrays = [
        _.range(1, Math.min(x, y) + 1).map(k => Pos(x-k, y-k)),     // Left down
        _.range(1, Math.min(x, 7-y) + 1).map(k => Pos(x-k, y+k)),   // Left up
        _.range(1, Math.min(7-x, y) + 1).map(k => Pos(x+k, y-k)),   // Right down
        _.range(1, Math.min(7-x, 7-y) + 1).map(k => Pos(x+k, y+k)), // Left down
    ]
    for (const array of arrays) {
        for (const pos of array) {
            if (!canMove(state, pos)) 
                break
            moves.push(pos)
            if (canCapture(state, pos))
                break
        }
    }
    return moves
}

function getRookMoves(state, from) {
    const {x, y} = from
    let moves = []
    const arrays = [
        _.range(0, x).map(x => Pos(x, y)),    // Move left
        _.range(x+1, 8).map(x => Pos(x, y)),  // Move right
        _.range(0, y).map(y => Pos(x, y)),    // Move down
        _.range(y+1, 8).map(y => Pos(x, y)),  // Move up
    ]
    arrays[0].reverse()
    arrays[2].reverse()
    for (const array of arrays) {
        for (const pos of array) {
            if (!canMove(state, pos)) 
                break
            moves.push(pos)
            if (canCapture(state, pos))
                break
        }
    }
    return moves
}

function getQueenMoves(state, from) {
    return [...getBishopMoves(state, from), ...getRookMoves(state, from)]
}

function getKingMoves(state, from, disableCastle = false) {
    const board = posBoard(state.board)
    let moves = []
    for (const y of _.range(from.y-1, from.y+2)) {
        for (const x of _.range(from.x-1, from.x+2)) {
            const to = Pos(x, y)
            if (validPos(to) && canMove(state, to)) {
                moves.push(to)
            }
        }
    }

    if (disableCastle) return moves
    
    // Castle
    for (const side of [1, -1]) {
        const step1 = Pos(from.x + side, from.y)
        const step2 = Pos(from.x + 2*side, from.y)
        if ((side == 1 ? state.castleK : state.castleQ) && board(step1) == 0 && board(step2) == 0 && !isChecked(state, step1) && !isChecked(state, step2)) {
            moves.push(step2)
        }
    }
    

    return moves
}

function isKingChecked(state) {
    return isChecked(state, getKingPos(state))
}

function getKingPos(state) {
    for (const y of _.range(0, 8)) {
        for (const x of _.range(0, 8)) {
            if (state.board[y][x] == KING * state.player) {
                return Pos(x, y)
            }
        }
    }
}

function isChecked(state, position) {
    const board = posBoard(state.board)
    if (board(position) == 0) {
        return false
    }
    
    const {x, y} = position
    const player = state.player
    const other = player * -1
    const filter = (piece, second=null) => pos => board(pos) == piece * other || (second ? board(pos) == second * other : false)

    // Test pawn
    if ([x-1, x+1].map(xNew => Pos(xNew, y + player)).find(pos => validPos(pos) && board(pos) == PAWN * other)) {
        return true
    }

    // Test knight
    if (getKnightMoves(state, position).find(filter(KNIGHT))) {
        return true
    }

    // Test bishop + queen
    if (getBishopMoves(state, position).find(filter(BISHOP, QUEEN))) {
        return true
    }

    // Test rook + queen
    if (getRookMoves(state, position).find(filter(ROOK, QUEEN))) {
        return true
    }

    // Test king
    if (getKingMoves(state, position, true).find(filter(KING))) {
        return true
    }

    return false
}

export default gameSlice.reducer

function gameSelector(field = null) {
    return state => field ? state.game.present[field] : state.game.present
}

export {Pos, eqPos, gameSelector}