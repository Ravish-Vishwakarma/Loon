import "./App.css";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { LauncherPage } from "./pages/launcher";
import { SettingPage } from "./pages/setting";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<LauncherPage />} />
        <Route path="/setting" element={<SettingPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
