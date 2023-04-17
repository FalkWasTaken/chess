import React, { useEffect, useState } from "react"
import { useDispatch, useSelector } from "react-redux"
import Draggable from "react-draggable"

import { Pos, eqPos, gameSelector } from "../model/game"

import "../styles/board.css"
import _ from "lodash"

const assetArray = ["", "pawn", "knight", "bishop", "rook", "queen", "king"]

function pieceToAsset(piece) {
    if (piece === 0) return null
    const name = assetArray[Math.abs(piece)]
    const color = piece > 0 ? "_w" : "_b"
    return `src/assets/${name}${color}.png`
}

function colorClass(x, y) {
    return x % 2 !== y % 2 ? "bg-black" : "bg-white"
}

function posFromId(id) {
    return { x: parseInt(id.charAt(6)), y: 7 - parseInt(id.charAt(7)) }
}

function flipPos({ x, y }) {
    return { x: x, y: 7 - y }
}

function Board() {
    const dispatch = useDispatch()
    const board = useSelector(gameSelector("board"))
    const validMoves = useSelector(gameSelector("validMoves"))
    const prevMove = useSelector(gameSelector("prevMove"))
    const [from, setFrom] = useState()

    useEffect(() => setFrom(), [prevMove])

    function dragStart(e) {
        e.target.style["z-index"] = "10"
        const square = e.target.parentNode
        const from = posFromId(square.id)
        dispatch({ type: "game/SET_FROM", payload: from })
        setFrom(from)
    }

    function dragStop(e) {
        e.target.style["z-index"] = "1"
        const elements = document.elementsFromPoint(e.clientX, e.clientY)
        const square = elements.find(elem => elem.classList.contains("boardSquare"))
        if (!square) return
        const to = posFromId(square.id)
        dispatch({ type: "game/TRY_MOVE", payload: to })
    }

    function renderPiece(piece, x, y) {
        const asset = pieceToAsset(piece)
        const key = `square${x}${y}`
        const pos = flipPos(Pos(x, y))

        function renderHighlight() {
            if (prevMove && (eqPos(prevMove.from, pos) || eqPos(prevMove.to, pos)))
                return "squareHighlight"

            if (!from || !eqPos(from, pos)) return ""
            return "squareHighlight"
        }

        function clickSquare() {
            if (piece === 0) {
                setFrom()
            }
            if (from) {
                dispatch({ type: "game/TRY_MOVE", payload: pos })
            }
        }

        return <div className={`boardSquare ${colorClass(x, y)} ${renderHighlight()} ${from ? "hoverSquare" : ""}`} key={key} id={key} onClick={clickSquare}>
            {asset ?
                <Draggable bounds="#board" onStart={dragStart} onStop={dragStop} position={Pos(0, 0)} positionOffset={Pos(0, 0)}>
                    <div>
                        <img className={"boardPiece"} src={asset} draggable="false" alt="" />
                    </div>
                </Draggable> : null
            }
            {from && validMoves.find(valid => eqPos(valid, pos)) ?
                <div className={board[7 - y][x] === 0 ? "squareMarker" : "captureMarker"} /> : null
            }

        </div>
    }

    function renderRow(row, y) {
        return <div className="boardRow" key={y}>
            {row.map((piece, x) => renderPiece(piece, x, y))}
        </div>
    }

    function renderBoard() {
        const corrected = [...board]
        corrected.reverse()
        return corrected.map(renderRow)
    }

    return <div className="board" id="board">
        {renderBoard()}
        <div className="boardNumbers">{_.rangeRight(1, 9).map(i => <span className={i % 2 !== 0 ? "text-white" : "text-black"} key={i}>{i}</span>)}</div>
        <div className="boardLetters">{[..."abcdefgh"].map((n, i) => <span className={i % 2 === 0 ? "text-white" : "text-black"} key={n}>{n}</span>)}</div>
    </div>
}

export default Board