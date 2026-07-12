import { Heart, Home, Settings } from "lucide-react"
import { Button } from "./ui/button"
import { useNavigate } from "react-router-dom"
import { openUrl } from "@tauri-apps/plugin-opener"

function Header() {
    const navigate = useNavigate()

    return <>
        <div className="flex h-8 justify-between">
            <p className="m-1 font-jetbrains text-lg">
                LOON
            </p>
            <div>
                <Button variant={"ghost"} size={"icon"} onClick={() => navigate("/home")}>
                    <Home></Home>
                </Button>
                <Button variant={"ghost"} size={"icon"} onClick={() => openUrl("https://ravishvish.gumroad.com/coffee")}>
                    <Heart></Heart>
                </Button>
                <Button variant={"ghost"} size={"icon"} onClick={() => navigate("/settings")}>
                    <Settings></Settings>
                </Button>
            </div>
        </div>
    </>
}

export { Header }