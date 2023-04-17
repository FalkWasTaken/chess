const API_URL = "http://localhost:8000/engine"

const pieceToChar = [..." pnbrqk"]
const indexToChar = [..."abcdefgh"]
const charToIndex = {a: 0, b:1, c: 2, d: 3, e: 4, f: 5, g: 6, h: 7}

function requestAnalysisThunk() {
    return async (dispatch, getState) => {
        const state = getState().game.present
        const fen = toFEN(state)
        const URL = API_URL + "/make_move?" + new URLSearchParams({fen})
        const res = await fetch(URL).then(res => res.json())
        console.log(res)
        if (!res.checkmate) {
            dispatch({type: "game/TRY_MOVE", payload: {from: algPosToNum(res.from), to: algPosToNum(res.to), computerScore: res.score}})
        }
    }
}

function algPosToNum(pos) {
    return {x: charToIndex[pos[0]], y: parseInt(pos[1]) - 1}
}

function toFEN(state) {
    let board = [...state.board]
    board.reverse()
    const player = state.player
    const white = isWhite(player)
    let fen = ""
    for (const row of board) {
        let countEmpty = 0
        for (const piece of row) {
            if (!piece) {
                countEmpty++
                continue
            } else if (countEmpty) {
                fen += countEmpty
                countEmpty = 0
            }
            let char = pieceToChar[Math.abs(piece)]
            if (isWhite(piece)) 
                char = char.toUpperCase()
            fen += char
        }
        if (countEmpty) {
            fen += countEmpty
            countEmpty = 0
        }
        fen += "/"
    }
    fen = fen.slice(0, -1)
    fen += ` ${white ? "w" : "b"} `
    
    let castleWhite = white ? {k: state.castleK, q: state.castleQ} : state.castleOther
    let castleBlack = !white ? {k: state.castleK, q: state.castleQ} : state.castleOther
    const castleStatus = `${castleWhite.k ? "K" : ""}${castleWhite.q ? "Q" : ""}${castleBlack.k ? "k" : ""}${castleBlack.q ? "q" : ""}`
    fen += castleStatus.length > 0 ? castleStatus : "-"

    fen += ` ${state.enPassant ? posToAlg(state.enPassant) : "-"} `
    fen += `${state.halfmove} ${state.fullmove}`
    return fen
}

function posToAlg(pos) {
    return `${indexToChar[pos.x]}${pos.y + 1}`
}

function Pos(x, y) {
    return {x: x, y: y}
}

function posBoard(board) {
    return pos => board[pos.y][pos.x]
}

function validPos(pos) {
    return validIndex(pos.x) && validIndex(pos.y)
}

function validIndex(i) {
    return i >= 0 && i < 8
}

function canCapture(state, to) {
    return Math.sign(state.board[to.y][to.x]) == state.player * -1
}

function canMove(state, to) {
    return Math.sign(state.board[to.y][to.x]) != state.player
}

function canControl(state, pos) {
    return Math.sign(state.board[pos.y][pos.x]) == state.player
}

function eqPos(a, b) {
    return a.x == b.x && a.y == b.y
}

function isWhite(x) {
    return Math.sign(x) == 1
}

function backRank(state) {
    return isWhite(state.player) ? 0 : 7
}

export { 
    toFEN, 
    Pos, 
    validIndex, 
    posBoard, 
    validPos, 
    canCapture,
    canControl,
    canMove,
    eqPos,
    isWhite,
    backRank,
    requestAnalysisThunk
}