import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sparkles, LoaderCircle } from "lucide-react";
import { Button } from "@/components/ui/button";

type Status = "idle" | "recording" | "transcribing" | "cancelling" | "copied" | "polishing" | "cancelling-polish" | "polished" | "error";

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
            setTimeout(() => {
                setStatus("idle");
                invoke("hide_launcher_cmd").catch(console.error);
            }, 10000);
        });

        const unlistenPolishing = listen("polishing", () => {
            setStatus("polishing");
            setPendingPolish(null);
        });

        const unlistenPolishDone = listen("polish-done", () => {
            setStatus("polished");
            setPendingPolish(null);
            setTimeout(() => {
                setStatus("idle");
                invoke("hide_launcher_cmd").catch(console.error);
            }, 5000);
        });

        const unlistenCancelled = listen("transcription-cancelled", () => {
            setStatus("idle");
            invoke("hide_launcher_cmd").catch(console.error);
        });

        const unlistenCancelRequested = listen("cancel-requested", () => {
            setStatus((prev) => {
                if (prev === "transcribing") return "cancelling";
                if (prev === "polishing") return "cancelling-polish";
                return prev;
            });
            setTimeout(() => {
                setStatus((prev) => {
                    if (prev === "cancelling") return "transcribing";
                    if (prev === "cancelling-polish") return "polishing";
                    return prev;
                });
            }, 3000);
        });

        const unlistenError = listen("transcription-error", () => {
            setStatus("error");
            setTimeout(() => {
                setStatus("idle");
                invoke("hide_launcher_cmd").catch(console.error);
            }, 2000);
        });

        return () => {
            unlistenStart.then((f) => f());
            unlistenTranscribing.then((f) => f());
            unlistenDone.then((f) => f());
            unlistenPolishing.then((f) => f());
            unlistenPolishDone.then((f) => f());
            unlistenCancelled.then((f) => f());
            unlistenCancelRequested.then((f) => f());
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
            setTimeout(() => {
                setStatus("idle");
                invoke("hide_launcher_cmd").catch(console.error);
            }, 2000);
        }
    };

    const isRecording = status === "recording";

    return (
        <div
            className="flex items-center justify-center h-full w-full select-none"
        >
            <style>{`
                @keyframes wave {
                    0%, 100% { height: 4px; }
                    50% { height: 12px; }
                }
            `}</style>

            <div
                data-tauri-drag-region
                className="flex items-center justify-center h-full transition-all duration-300 ease-in-out rounded-full overflow-hidden border border-border"
                style={{
                    width: isRecording ? "48px" : "100%",
                    background: "#0a0a0a",
                }}
            >
                <div
                    className="flex items-center justify-center gap-1.5 h-full transition-all duration-300 ease-in-out"
                    style={{
                        padding: isRecording ? "0 8px" : "0 10px",
                        width: "100%",
                    }}
                >
                    {status === "recording" && <WaveBars />}
                    {status === "transcribing" && (
                        <>
                            <Spinner className="text-blue-500" />
                            <span className="text-[12px] text-blue-500 font-medium">Transcribing</span>
                        </>
                    )}
                    {status === "cancelling" && (
                        <>
                            <Spinner className="text-red-500" />
                            <span className="text-[12px] text-red-500 font-bold">Press Again to Cancel</span>
                        </>
                    )}
                    {status === "copied" && (
                        <>
                            <span className="text-[12px] text-green-500 font-medium">Copied</span>
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
                            <span className="text-[12px] text-blue-500 font-medium">Polishing</span>
                        </>
                    )}
                    {status === "cancelling-polish" && (
                        <>
                            <Spinner className="text-red-500" />
                            <span className="text-[12px] text-red-500 font-bold">Press Again to Cancel</span>
                        </>
                    )}
                    {status === "polished" && (
                        <span className="text-[10px] text-green-500 font-medium">Copied</span>
                    )}
                    {status === "error" && (
                        <span className="text-[10px] text-red-500 font-medium">Failed</span>
                    )}
                    {status === "idle" && (
                        <span className="text-[12px] text-muted-foreground">Ready</span>
                    )}
                </div>
            </div>
        </div>
    );
}

export { LauncherPage }
