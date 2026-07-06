import { Heart, Settings } from "lucide-react"
import { Button } from "./ui/button"

function Header() {
    return <>
        <div className="flex h-8 justify-between">
            <p className="m-1 font-jetbrains text-lg">
                LOON
            </p>
            <div>
                <Button variant={"ghost"} size={"icon"}>
                    <Heart></Heart>
                </Button>
                <Button variant={"ghost"} size={"icon"}>
                    <Settings></Settings>
                </Button>
            </div>
        </div>
    </>
}

export { Header }