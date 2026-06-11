import React from "react";
import { Routes, Route, NavLink } from "react-router-dom";
import SingleRun from "./routes/SingleRun.jsx";
import Compare from "./routes/Compare.jsx";

export default function App() {
  return (
    <div className="app">
      <header className="app-header">
        <div className="header-inner">
          <span className="header-logo">📊 Petstore Benchmark Viewer</span>
          <nav className="header-nav">
            <NavLink to="/" end className={({ isActive }) => isActive ? "nav-link active" : "nav-link"}>
              Single Run
            </NavLink>
            <NavLink to="/compare" className={({ isActive }) => isActive ? "nav-link active" : "nav-link"}>
              Compare
            </NavLink>
          </nav>
        </div>
      </header>
      <main className="app-main">
        <Routes>
          <Route path="/" element={<SingleRun />} />
          <Route path="/compare" element={<Compare />} />
        </Routes>
      </main>
    </div>
  );
}
