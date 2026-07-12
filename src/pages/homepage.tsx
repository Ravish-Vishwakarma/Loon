import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Header } from "@/components/header"

type TranscriptionRow = {
    id: number
    transcription: string
    ai: string
    created_at: string
}

function HomePage() {
    const [rows, setRows] = useState<TranscriptionRow[]>([])

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

    return (
        <div className="flex flex-col gap-4 p-4">
            <Header />
            <div className="rounded-lg border">
                <table className="w-full text-sm">
                    <thead>
                        <tr className="border-b text-left text-muted-foreground">
                            <th className="px-3 py-2 font-medium">Transcription</th>
                            <th className="px-3 py-2 font-medium">Model</th>
                            <th className="px-3 py-2 font-medium">Date</th>
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
                                    <td className="px-3 py-2">{row.transcription}</td>
                                    <td className="px-3 py-2 text-muted-foreground">{row.ai}</td>
                                    <td className="px-3 py-2 text-muted-foreground whitespace-nowrap">{row.created_at}</td>
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
