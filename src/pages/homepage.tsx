import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Header } from "@/components/header"
import { Button } from "@/components/ui/button"

type TranscriptionRow = {
    id: number
    transcription: string
    ai: string
    audio: string
    created_at: string
}

function HomePage() {
    const [status, setStatus] = useState("No action yet.")
    const [rows, setRows] = useState<TranscriptionRow[]>([])

    const handleCreate = async () => {
        try {
            const insertedId = await invoke<number>("create_transcription", {
                transcription: "Hello from the settings window",
                ai: "local-test",
                audio: "test-audio.wav",
            })
            setStatus(`Inserted transcription with id ${insertedId}`)
        } catch (error) {
            setStatus(`Create failed: ${error}`)
        }
    }

    const handleRead = async () => {
        try {
            const data = await invoke<TranscriptionRow[]>("read_transcriptions")
            setRows(data)
            setStatus(`Loaded ${data.length} transcription(s)`)
        } catch (error) {
            setStatus(`Read failed: ${error}`)
        }
    }

    return (
        <div className="flex flex-col gap-4 p-4">
            <Header />
            <Button
                onClick={
                    async () => {
                        let data = await invoke("get_storage_used_size");
                        console.log(data);
                    }
                }
            >Get Size</Button>
            <div className="flex gap-2">
                <Button onClick={handleCreate}>Create test transcription</Button>
                <Button variant="outline" onClick={handleRead}>Read transcriptions</Button>
            </div>
            <p className="text-sm text-muted-foreground">{status}</p>
            <div className="rounded-lg border p-3">
                <h2 className="mb-2 text-sm font-semibold">Saved transcriptions</h2>
                {rows.length === 0 ? (
                    <p className="text-sm text-muted-foreground">No rows yet.</p>
                ) : (
                    <ul className="space-y-2 text-sm">
                        {rows.map((row) => (
                            <li key={row.id} className="rounded border p-2">
                                <div className="font-medium">{row.transcription}</div>
                                <div className="text-muted-foreground">
                                    AI: {row.ai} · Audio: {row.audio} · {row.created_at}
                                </div>
                            </li>
                        ))}
                    </ul>
                )}
            </div>
        </div>
    )
}

export { HomePage }