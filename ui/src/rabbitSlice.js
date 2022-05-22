import { createSlice } from "@reduxjs/toolkit";

export const rabbitSlice = createSlice({
  name: 'rabbit',
  initialState: {
    id: null,
    createdAt: null,
    status: null,
    bodyColor: null,
    patchColor: null,
    eyeColor: null,
  },
  reducers: {
    setId: (state, action) => {
      console.log("setId", action);
      state.id = action.payload.rabbitId;
    },
    setStatus: (state, action) => {
      console.log("setStatus", action);
      state.status = action.payload.status;
    },
    setRabbit: (state, action) => {
      const { status, bodyColor, patchColor, eyeColor } = action.payload;
      state.status = status;
      state.bodyColor = bodyColor;
      state.patchColor = patchColor;
      state.eyeColor = eyeColor;
    },
  }
});

export const {
  setId,
  setStatus,
  setRabbit,
} = rabbitSlice.actions;

export default rabbitSlice.reducer;

