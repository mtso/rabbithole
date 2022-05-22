import { configureStore } from "@reduxjs/toolkit";
import rabbitReducer from "./rabbitSlice";

export default configureStore({
  reducer: {
    rabbit: rabbitReducer,
  },
});

