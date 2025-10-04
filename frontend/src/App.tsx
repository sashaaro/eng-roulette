import { useState } from 'react'
import './app.css'
import { BrowserRouter, Routes, Route } from "react-router";
import Home from "./routes/Home.tsx";
import Login from "./component/Login.tsx";
import Register from "./routes/Register.tsx";
import GoogleAuthCallback from "./routes/GoogleAuthCallback.tsx";
import {AuthProvider} from "./context/session.tsx";

function App() {
    return (
    <AuthProvider>
      <BrowserRouter>
          <Routes>
              <Route path="/" element={<Home />} />
              <Route path="/login" element={<Login />} />
              <Route path="/register" element={<Register />} />
              <Route path="/auth/google/callback" element={<GoogleAuthCallback />} />
          </Routes>
      </BrowserRouter>
    </AuthProvider>
  )
}

export default App
