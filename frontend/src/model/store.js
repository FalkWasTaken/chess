import { configureStore } from "@reduxjs/toolkit"
import undoable from "redux-undo"
import gameReducer from "./game"

export const store = configureStore({
  reducer: {
    game: undoable(gameReducer)
  },
  middleware: getDefaultMiddleware => getDefaultMiddleware()
});
