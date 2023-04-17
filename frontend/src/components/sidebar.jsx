import React from "react"
import { useDispatch, useSelector } from "react-redux"
import { ActionCreators } from "redux-undo"

import { gameSelector } from "../model/game"
import { requestAnalysisThunk } from "../model/utils"

import "../styles/sidebar.css"

function Sidebar() {
    const dispatch = useDispatch()
    const score = useSelector(gameSelector("computerScore"))


    return <div className="sidebar">
        <button onClick={() => dispatch(ActionCreators.undo())}>Undo</button>
        <button onClick={() => dispatch(requestAnalysisThunk())}>Get computer move</button>
        {score != null ? <div>
            Computer score: <span>{score}</span>
        </div>: ""}
    </div>
}

export default Sidebar