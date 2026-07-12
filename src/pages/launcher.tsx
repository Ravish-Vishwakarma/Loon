import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { X, Sparkles, LoaderCircle } from "lucide-react";
import { Button } from "@/components/ui/button";

type Status = "idle" | "recording" | "transcribing" | "copied" | "polishing" | "polished" | "error";

function WaveBars() {
    return (
        <div className="flex items-center gap-[2px] h-3">
            {[0, 1, 2, 3, 4].map((i) => (
                <span
                    key={i}
                    className="w-[2px] bg-red-500 rounded-full animate-[wave_0.8s_ease-in-out_infinite]"
                    style={{
                        animationDelay: `${i * 0.12}s`,
                        height: "4px",
                    }}
                />
            ))}
        </div>
    );
}

function Spinner({ className }: { className?: string }) {
    return <LoaderCircle className={`animate-spin ${className ?? ""}`} size={12} />;
}

function LauncherPage() {
    const [status, setStatus] = useState<Status>("idle");
    const [pendingPolish, setPendingPolish] = useState<{ id: number; text: string } | null>(null);

    useEffect(() => {
        const unlistenStart = listen("start-recording", () => {
            setStatus("recording");
            setPendingPolish(null);
        });

        const unlistenTranscribing = listen("transcribing", () => {
            setStatus("transcribing");
        });

        const unlistenDone = listen<{ id: number; text: string }>("transcription-done", (event) => {
            setStatus("copied");
            setPendingPolish(event.payload);
            setTimeout(() => setStatus("idle"), 3000);
        });

        const unlistenPolishDone = listen("polish-done", () => {
            setStatus("polished");
            setPendingPolish(null);
            setTimeout(() => setStatus("idle"), 2000);
        });

        const unlistenError = listen("transcription-error", () => {
            setStatus("error");
            setTimeout(() => setStatus("idle"), 2000);
        });

        return () => {
            unlistenStart.then((f) => f());
            unlistenTranscribing.then((f) => f());
            unlistenDone.then((f) => f());
            unlistenPolishDone.then((f) => f());
            unlistenError.then((f) => f());
        };
    }, []);

    const handlePolish = async () => {
        if (!pendingPolish) return;
        setStatus("polishing");
        try {
            await invoke("polish_cmd", { id: pendingPolish.id, text: pendingPolish.text });
        } catch (e) {
            console.error("polish failed:", e);
            setStatus("error");
            setTimeout(() => setStatus("idle"), 2000);
        }
    };

    const handleClose = async () => {
        if (status === "recording") {
            try {
                await invoke("stop_recording_cmd");
            } catch (e) {
                console.error("stop failed:", e);
            }
        }
        setStatus("idle");
        setPendingPolish(null);
        await getCurrentWindow().hide();
    };

    return (
        <div data-tauri-drag-region className="flex items-center justify-between h-full w-full select-none px-2.5">
            <style>{`
                @keyframes wave {
                    0%, 100% { height: 4px; }
                    50% { height: 12px; }
                }
            `}</style>

            <div className="flex items-center gap-1.5 min-w-0">
                {status === "recording" && (
                    <>
                        <WaveBars />
                        <span className="text-[10px] text-red-500 font-medium">REC</span>
                    </>
                )}
                {status === "transcribing" && (
                    <>
                        <Spinner className="text-blue-500" />
                        <span className="text-[10px] text-blue-500 font-medium">Transcribing</span>
                    </>
                )}
                {status === "copied" && (
                    <>
                        <span className="text-[10px] text-green-500 font-medium">Copied</span>
                        <Button
                            variant="ghost"
                            size="icon-xs"
                            onClick={handlePolish}
                            className="text-blue-500 hover:text-blue-600 hover:bg-blue-500/10"
                        >
                            <Sparkles size={10} />
                        </Button>
                    </>
                )}
                {status === "polishing" && (
                    <>
                        <Spinner className="text-blue-500" />
                        <span className="text-[10px] text-blue-500 font-medium">Polishing</span>
                    </>
                )}
                {status === "polished" && (
                    <span className="text-[10px] text-green-500 font-medium">Copied</span>
                )}
                {status === "error" && (
                    <span className="text-[10px] text-red-500 font-medium">Failed</span>
                )}
                {status === "idle" && (
                    <span className="text-[10px] text-muted-foreground">Ready</span>
                )}
            </div>

            <Button
                variant="ghost"
                size="icon-xs"
                onClick={handleClose}
                className="text-muted-foreground hover:text-foreground shrink-0"
            >
                <X size={10} />
            </Button>
        </div>
    );
}

export { LauncherPage };
