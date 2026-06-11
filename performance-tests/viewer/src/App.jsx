import React from "react";
import { Routes, Route, NavLink } from "react-router-dom";
import SingleRun from "./routes/SingleRun.jsx";
import Compare from "./routes/Compare.jsx";
import RunTests from "./routes/RunTests.jsx";
import Queue from "./routes/Queue.jsx";

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
            <NavLink to="/run" className={({ isActive }) => isActive ? "nav-link active" : "nav-link"}>
              Run Tests
            </NavLink>
            <NavLink to="/queue" className={({ isActive }) => isActive ? "nav-link active" : "nav-link"}>
              Queue
            </NavLink>
          </nav>
        </div>
      </header>
      <main className="app-main">
        <Routes>
          <Route path="/" element={<SingleRun />} />
          <Route path="/compare" element={<Compare />} />
          <Route path="/run" element={<RunTests />} />
          <Route path="/queue" element={<Queue />} />
        </Routes>
      </main>
    </div>
  );
}
