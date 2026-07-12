import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Header } from "@/components/header"
import { toast } from "sonner"

type TranscriptionRow = {
    id: number
    transcription: string
    ai: string
    created_at: string
}

function HomePage() {
    const [rows, setRows] = useState<TranscriptionRow[]>([])

    const formatTimestamp = (ts: string) => {
        const d = new Date(ts.replace(" ", "T") + "Z")
        const pad = (n: number) => n.toString().padStart(2, "0")
        return `${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${pad(d.getFullYear()).slice(2)} ${pad(d.getHours())}:${pad(d.getMinutes())}`
    }

    const load = async () => {
        try {
            const data = await invoke<TranscriptionRow[]>("read_transcriptions")
            setRows(data)
        } catch (e) {
            console.error("failed to load transcriptions:", e)
        }
    }

    useEffect(() => {
        load()
    }, [])

    const copyToClipboard = async (text: string, label: string) => {
        await navigator.clipboard.writeText(text)
        toast.success(`${label} copied to clipboard`)
    }

    return (
        <div className="flex flex-col gap-4 p-4 min-h-screen bg-background">
            <Header />
            <div className="rounded-lg border overflow-hidden">
                <table className="w-full text-sm table-fixed">
                    <thead>
                        <tr className="border-b text-left text-muted-foreground">
                            <th className="px-3 py-2 font-medium w-2/5">Transcription</th>
                            <th className="px-3 py-2 font-medium w-2/5">AI Polish</th>
                            <th className="px-3 py-2 font-medium w-1/5">Date</th>
                        </tr>
                    </thead>
                    <tbody>
                        {rows.length === 0 ? (
                            <tr>
                                <td colSpan={3} className="px-3 py-4 text-center text-muted-foreground">
                                    No transcriptions yet.
                                </td>
                            </tr>
                        ) : (
                            rows.map((row) => (
                                <tr key={row.id} className="border-b last:border-b-0 hover:bg-muted/50">
                                    <td
                                        className="px-3 py-2 truncate cursor-pointer hover:text-foreground transition-colors"
                                        title={row.transcription}
                                        onClick={() => copyToClipboard(row.transcription, "Transcription")}
                                    >
                                        {row.transcription}
                                    </td>
                                    <td
                                        className={`px-3 py-2 truncate transition-colors ${row.ai
                                            ? "cursor-pointer hover:text-foreground text-muted-foreground"
                                            : "text-muted-foreground/50"
                                            }`}
                                        title={row.ai || undefined}
                                        onClick={() => row.ai && copyToClipboard(row.ai, "AI polish")}
                                    >
                                        {row.ai || "—"}
                                    </td>
                                    <td className="px-3 py-2 text-muted-foreground whitespace-nowrap">{formatTimestamp(row.created_at)}</td>
                                </tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    )
}

export { HomePage }
