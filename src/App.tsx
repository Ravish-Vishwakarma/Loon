import "./App.css";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { LauncherPage } from "./pages/launcher";
import { HomePage } from "./pages/homepage";
import { SettingsPage } from "./pages/settings";
import { Toaster } from "sonner";

function App() {
  return (
    <BrowserRouter>
      <Toaster position="bottom-center" richColors />
      <Routes>
        <Route path="/" element={<LauncherPage />} />
        <Route path="/home" element={<HomePage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
