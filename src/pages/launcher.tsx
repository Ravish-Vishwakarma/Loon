import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

type Status = "idle" | "recording" | "transcribing" | "copied" | "error";

function LauncherPage() {
    const [status, setStatus] = useState<Status>("idle");

    useEffect(() => {
        const unlistenStart = listen("start-recording", () => {
            setStatus("recording");
        });

        const unlistenTranscribing = listen("transcribing", () => {
            setStatus("transcribing");
        });

        const unlistenDone = listen<string>("transcription-done", () => {
            setStatus("copied");
            setTimeout(() => setStatus("idle"), 2000);
        });

        const unlistenError = listen<string>("transcription-error", () => {
            setStatus("error");
            setTimeout(() => setStatus("idle"), 2000);
        });

        return () => {
            unlistenStart.then((f) => f());
            unlistenTranscribing.then((f) => f());
            unlistenDone.then((f) => f());
            unlistenError.then((f) => f());
        };
    }, []);

    const handleClose = async () => {
        if (status === "recording") {
            try {
                await invoke("stop_recording_cmd");
            } catch (e) {
                console.error("stop failed:", e);
            }
        }
        setStatus("idle");
        await getCurrentWindow().hide();
    };

    return (
        <div data-tauri-drag-region className='flex items-center justify-between h-full w-full select-none px-2'>
            <div className='flex items-center gap-1.5'>
                {status === "recording" && (
                    <span className='flex items-center gap-1.5 text-[11px] text-red-500 font-medium'>
                        <span className='w-2 h-2 rounded-full bg-red-500 animate-pulse' />
                        Recording
                    </span>
                )}
                {status === "transcribing" && (
                    <span className='flex items-center gap-1.5 text-[11px] text-blue-500 font-medium'>
                        <span className='w-2 h-2 rounded-full bg-blue-500 animate-pulse' />
                        Transcribing
                    </span>
                )}
                {status === "copied" && (
                    <span className='text-[11px] text-green-500 font-medium'>Copied!</span>
                )}
                {status === "error" && (
                    <span className='text-[11px] text-red-500 font-medium'>Failed</span>
                )}
                {status === "idle" && (
                    <span className='text-[11px] text-muted-foreground'>Ready</span>
                )}
            </div>
            <button
                onClick={handleClose}
                className='text-[11px] text-muted-foreground hover:text-foreground cursor-pointer'
            >
                ✕
            </button>
        </div>
    )
}

export { LauncherPage }
