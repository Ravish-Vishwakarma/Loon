import "./App.css";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { LauncherPage } from "./pages/launcher";
import { HomePage } from "./pages/homepage";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<LauncherPage />} />
        <Route path="/home" element={<HomePage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
