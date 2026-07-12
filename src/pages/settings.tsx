import { useCallback, useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { revealItemInDir } from "@tauri-apps/plugin-opener"
import { Header } from "@/components/header"
import { Button } from "@/components/ui/button"
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from "@/components/ui/alert-dialog"
import { toast } from "sonner"
import { Download, Trash2, LoaderCircle, Check, ChevronDown, ChevronRight, FolderOpen } from "lucide-react"

type Config = {
    shortcut: string
    ai_model: string
    transcription_model: string
    ai_polish_prompt: string
    launch_on_startup: boolean
    auto_polish: boolean
    paste_mode: "copy" | "paste" | "both"
}

type AppPaths = {
    models_dir: string
    db_path: string
    config_path: string
    whisper_bin: string
}

type Model = {
    id: string
    name: string
    backend: string
    size: number
    filename: string
    download_url: string
    languages: string[]
}

type OllamaModel = {
    name: string
    model: string
    details: {
        parameter_size: string
        quantization_level: string
    }
}

const DEFAULT_POLISH_PROMPT = "Clean and polish the following transcription. Correct spelling, grammar, punctuation, and formatting. Remove filler words and accidental repetitions. Fix obvious transcription mistakes using context, but do not change the meaning or add new information. Keep the tone natural and preserve speaker labels and timestamps if they are present. Output only the polished transcript. \n Transcript: \n {{transcription}}"

function formatSize(bytes: number): string {
    if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(1)} GB`
    if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(0)} MB`
    return `${(bytes / 1e3).toFixed(0)} KB`
}

function Section({ title, open, onToggle, children }: {
    title: string
    open: boolean
    onToggle: () => void
    children: React.ReactNode
}) {
    return (
        <div className="flex flex-col gap-3">
            <button
                onClick={onToggle}
                className="flex items-center gap-2 text-sm font-semibold text-foreground hover:text-foreground/80 transition-colors cursor-pointer"
            >
                {open ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                {title}
            </button>
            {open && <div className="ml-5 flex flex-col gap-4">{children}</div>}
        </div>
    )
}

function SettingsPage() {
    const [config, setConfig] = useState<Config | null>(null)
    const [original, setOriginal] = useState<Config | null>(null)
    const [appPaths, setAppPaths] = useState<AppPaths | null>(null)
    const [downloadedModels, setDownloadedModels] = useState<Model[]>([])
    const [availableModels, setAvailableModels] = useState<Model[]>([])
    const [ollamaModels, setOllamaModels] = useState<OllamaModel[]>([])
    const [downloading, setDownloading] = useState<string | null>(null)

    const [openGeneral, setOpenGeneral] = useState(true)
    const [openTranscription, setOpenTranscription] = useState(true)
    const [openPolish, setOpenPolish] = useState(true)

    const fetchDownloaded = useCallback(() => {
        fetch("http://localhost:15000/v1/models/downloaded")
            .then((r) => r.json())
            .then((data: Model[]) => setDownloadedModels(data))
            .catch(console.error)
    }, [])

    const fetchAvailable = useCallback(() => {
        fetch("http://localhost:15000/v1/models/available")
            .then((r) => r.json())
            .then((data: Model[]) => setAvailableModels(data))
            .catch(console.error)
    }, [])

    useEffect(() => {
        invoke<Config>("load_config_cmd").then((c) => {
            setConfig(c)
            setOriginal(c)
        }).catch(console.error)
        invoke<AppPaths>("get_app_paths_cmd").then(setAppPaths).catch(console.error)
        fetchDownloaded()
        fetchAvailable()
        fetch("http://localhost:11434/api/tags")
            .then((r) => r.json())
            .then((data: { models: OllamaModel[] }) => setOllamaModels(data.models))
            .catch(console.error)
    }, [fetchDownloaded, fetchAvailable])

    const update = (key: keyof Config, value: string | boolean) => {
        if (!config) return
        setConfig({ ...config, [key]: value })
    }

    const handleSave = async () => {
        if (!config) return
        if (!config.ai_polish_prompt.includes("{{transcription}}")) {
            toast.warning("Polish prompt is missing {{transcription}} placeholder")
            return
        }
        try {
            await invoke("save_config_cmd", { config })
            setOriginal(config)
            toast.success("Settings saved")
        } catch (e) {
            toast.error("Failed to save settings")
            console.error(e)
        }
    }

    const handleDownload = async (modelId: string) => {
        setDownloading(modelId)
        try {
            const resp = await fetch("http://localhost:15000/v1/models/download", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ model_id: modelId }),
            })
            const data = await resp.json()
            if (data.status === "downloaded") {
                toast.success(`${modelId} downloaded`)
                fetchDownloaded()
            } else {
                toast.error(`Download failed: ${data.status}`)
            }
        } catch (e) {
            toast.error(`Download failed: ${e}`)
        } finally {
            setDownloading(null)
        }
    }

    const handleDelete = async (modelId: string) => {
        if (!window.confirm(`Delete ${modelId}?`)) return
        try {
            const resp = await fetch(`http://localhost:15000/v1/models/${modelId}`, {
                method: "DELETE",
            })
            const data = await resp.json()
            if (data.status === "deleted") {
                toast.success(`${modelId} deleted`)
                fetchDownloaded()
            } else {
                toast.error(`Delete failed: ${data.status}`)
            }
        } catch (e) {
            toast.error(`Delete failed: ${e}`)
        }
    }

    const isDownloaded = (id: string) => downloadedModels.some((m) => m.id === id)
    const hasChanges = config !== null && original !== null && JSON.stringify(config) !== JSON.stringify(original)

    if (!config) return null

    return (
        <div className="flex flex-col gap-6 p-4 w-full min-h-screen bg-background">
            <div className="flex items-center justify-between">
                <Header />
                <Button onClick={handleSave} disabled={!hasChanges}>Save</Button>
            </div>

            {/* General */}
            <Section title="General" open={openGeneral} onToggle={() => setOpenGeneral(!openGeneral)}>
                <div className="flex flex-col gap-1.5">
                    <label className="text-sm font-medium">Keyboard Shortcut</label>
                    <input
                        type="text"
                        value={config.shortcut}
                        onChange={(e) => update("shortcut", e.target.value)}
                        placeholder="e.g. Ctrl+Shift+Space"
                        className="flex h-8 w-full max-w-sm rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    />
                    <p className="text-xs text-muted-foreground">
                        Global shortcut to start/stop recording. Set to "None" to disable. <b>(Require Restart)</b>
                    </p>
                </div>

                <div className="flex flex-col gap-1.5">
                    <label className="text-sm font-medium">Output Mode</label>
                    <select
                        value={config.paste_mode}
                        onChange={(e) => update("paste_mode", e.target.value as Config["paste_mode"])}
                        className="flex h-8 w-full max-w-sm rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    >
                        <option value="copy">Copy to clipboard</option>
                        <option value="paste">Paste at cursor</option>
                        <option value="both">Copy + Paste at cursor</option>
                    </select>
                    <p className="text-xs text-muted-foreground">
                        What happens after transcription. Paste simulates Ctrl+V at your cursor.
                    </p>
                </div>
            </Section>

            <div className="h-px bg-border" />

            {/* Transcription */}
            <Section title="Transcription" open={openTranscription} onToggle={() => setOpenTranscription(!openTranscription)}>
                <div className="flex flex-col gap-1.5">
                    <label className="text-sm font-medium">Active Model</label>
                    <select
                        value={config.transcription_model}
                        onChange={(e) => update("transcription_model", e.target.value)}
                        className="flex h-8 w-full max-w-sm rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    >
                        {downloadedModels.length === 0 && <option value="">No models downloaded</option>}
                        {downloadedModels.map((m) => (
                            <option key={m.id} value={m.id}>
                                {m.name} ({m.id})
                            </option>
                        ))}
                    </select>
                    <p className="text-xs text-muted-foreground">
                        Whisper model used for speech-to-text transcription.
                    </p>
                </div>

                <div className="flex flex-col gap-2">
                    <label className="text-sm font-medium">Downloaded Models</label>
                    {downloadedModels.length === 0 ? (
                        <p className="text-xs text-muted-foreground">No models downloaded yet.</p>
                    ) : (
                        <div className="rounded-lg border overflow-hidden">
                            <table className="w-full text-sm table-fixed">
                                <thead>
                                    <tr className="border-b text-left text-muted-foreground">
                                        <th className="px-3 py-2 font-medium w-2/5">Model</th>
                                        <th className="px-3 py-2 font-medium w-1/5">Size</th>
                                        <th className="px-3 py-2 font-medium w-1/5">Backend</th>
                                        <th className="px-3 py-2 font-medium w-1/5"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {downloadedModels.map((m) => (
                                        <tr key={m.id} className="border-b last:border-b-0">
                                            <td className="px-3 py-2 truncate" title={m.name}>{m.name}</td>
                                            <td className="px-3 py-2 text-muted-foreground">{formatSize(m.size)}</td>
                                            <td className="px-3 py-2 text-muted-foreground">{m.backend}</td>
                                            <td className="px-3 py-2 text-right">
                                                <Button
                                                    variant="ghost"
                                                    size="icon-xs"
                                                    onClick={() => handleDelete(m.id)}
                                                    className="text-destructive hover:text-destructive"
                                                >
                                                    <Trash2 size={12} />
                                                </Button>
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                    )}
                </div>

                <div className="flex flex-col gap-2">
                    <label className="text-sm font-medium">Available Models</label>
                    {availableModels.length === 0 ? (
                        <p className="text-xs text-muted-foreground">No models available.</p>
                    ) : (
                        <div className="flex flex-col gap-1">
                            {availableModels.map((m) => {
                                const downloaded = isDownloaded(m.id)
                                const isDownloading = downloading === m.id
                                return (
                                    <div
                                        key={m.id}
                                        className="flex items-center justify-between rounded-md border px-3 py-2"
                                    >
                                        <div className="flex items-center gap-3 min-w-0">
                                            {downloaded && <Check size={12} className="text-green-500 shrink-0" />}
                                            <div className="flex flex-col min-w-0">
                                                <span className="text-sm font-medium truncate">{m.name}</span>
                                                <span className="text-xs text-muted-foreground">
                                                    {formatSize(m.size)} · {m.languages.join(", ")}
                                                </span>
                                            </div>
                                        </div>
                                        <div className="shrink-0">
                                            {isDownloading ? (
                                                <LoaderCircle size={14} className="animate-spin text-muted-foreground" />
                                            ) : downloaded ? null : (
                                                <Button
                                                    variant="ghost"
                                                    size="icon-xs"
                                                    onClick={() => handleDownload(m.id)}
                                                >
                                                    <Download size={12} />
                                                </Button>
                                            )}
                                        </div>
                                    </div>
                                )
                            })}
                        </div>
                    )}
                </div>
            </Section>

            <div className="h-px bg-border" />

            {/* AI Polish */}
            <Section title="AI Polish" open={openPolish} onToggle={() => setOpenPolish(!openPolish)}>
                <div className="flex flex-col gap-1.5">
                    <label className="text-sm font-medium">AI Model (Ollama)</label>
                    <select
                        value={config.ai_model}
                        onChange={(e) => update("ai_model", e.target.value)}
                        className="flex h-8 w-full max-w-sm rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    >
                        {ollamaModels.length === 0 && <option value="">No models available</option>}
                        {ollamaModels.map((m) => (
                            <option key={m.model} value={m.model}>
                                {m.name} ({m.details.parameter_size})
                            </option>
                        ))}
                    </select>
                    <p className="text-xs text-muted-foreground">
                        Local Ollama model used to polish transcriptions.
                    </p>
                </div>

                <div className="flex items-center justify-between rounded-md border p-3 max-w-sm">
                    <div className="flex flex-col gap-0.5">
                        <label className="text-sm font-medium">Auto Polish</label>
                        <p className="text-xs text-muted-foreground">
                            Automatically polish after recording.
                        </p>
                    </div>
                    <button
                        onClick={() => update("auto_polish", !config.auto_polish)}
                        className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors ${config.auto_polish ? "bg-primary" : "bg-input"
                            }`}
                    >
                        <span
                            className={`pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0 transition-transform ${config.auto_polish ? "translate-x-4" : "translate-x-0"
                                }`}
                        />
                    </button>
                </div>

                <div className="flex flex-col gap-1.5">
                    <div className="flex items-center justify-between">
                        <label className="text-sm font-medium">Polish Prompt</label>
                        <Button
                            variant="ghost"
                            size="xs"
                            onClick={() => update("ai_polish_prompt", DEFAULT_POLISH_PROMPT)}
                        >
                            Reset
                        </Button>
                    </div>
                    <textarea
                        value={config.ai_polish_prompt}
                        onChange={(e) => update("ai_polish_prompt", e.target.value)}
                        rows={5}
                        className="flex rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring resize-none"
                    />
                    <p className="text-xs text-muted-foreground">
                        Prompt template sent to Ollama. Use {"{{transcription}}"} as placeholder for the raw text.
                    </p>
                </div>
            </Section>

            <div className="h-px bg-border" />

            {/* Data & Storage */}
            <div className="flex flex-col gap-3">
                <h3 className="text-sm font-semibold text-foreground">Data & Storage</h3>
                <div className="ml-5 flex flex-col gap-2">
                    {appPaths && (
                        <>
                            <div className="flex items-center justify-between rounded-md border px-3 py-2">
                                <div className="flex flex-col min-w-0">
                                    <span className="text-xs text-muted-foreground">Whisper Binary</span>
                                    <span className="text-sm truncate" title={appPaths.whisper_bin}>{appPaths.whisper_bin || "Not found"}</span>
                                </div>
                                {appPaths.whisper_bin && (
                                    <Button variant="ghost" size="icon-xs" onClick={() => revealItemInDir(appPaths.whisper_bin)}>
                                        <FolderOpen size={12} />
                                    </Button>
                                )}
                            </div>
                            <div className="flex items-center justify-between rounded-md border px-3 py-2">
                                <div className="flex flex-col min-w-0">
                                    <span className="text-xs text-muted-foreground">Models Directory</span>
                                    <span className="text-sm truncate" title={appPaths.models_dir}>{appPaths.models_dir}</span>
                                </div>
                                <Button variant="ghost" size="icon-xs" onClick={() => revealItemInDir(appPaths.models_dir)}>
                                    <FolderOpen size={12} />
                                </Button>
                            </div>
                            <div className="flex items-center justify-between rounded-md border px-3 py-2">
                                <div className="flex flex-col min-w-0">
                                    <span className="text-xs text-muted-foreground">Database</span>
                                    <span className="text-sm truncate" title={appPaths.db_path}>{appPaths.db_path}</span>
                                </div>
                                <Button variant="ghost" size="icon-xs" onClick={() => revealItemInDir(appPaths.db_path)}>
                                    <FolderOpen size={12} />
                                </Button>
                            </div>
                        </>
                    )}
                    <AlertDialog>
                        <AlertDialogTrigger asChild>
                            <Button variant="destructive" size="sm" className="mt-1">
                                <Trash2 size={12} className="mr-1" />
                                Clear All Transcriptions
                            </Button>
                        </AlertDialogTrigger>
                        <AlertDialogContent>
                            <AlertDialogHeader>
                                <AlertDialogTitle>Clear all transcriptions?</AlertDialogTitle>
                                <AlertDialogDescription>
                                    This will permanently delete all transcriptions and their AI polish. This action cannot be undone.
                                </AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                                <AlertDialogCancel>Cancel</AlertDialogCancel>
                                <AlertDialogAction onClick={async () => {
                                    try {
                                        await invoke("clear_transcriptions");
                                        toast.success("All transcriptions cleared");
                                    } catch (e) {
                                        console.error("clear failed:", e);
                                        toast.error("Failed to clear transcriptions");
                                    }
                                }}>
                                    Delete All
                                </AlertDialogAction>
                            </AlertDialogFooter>
                        </AlertDialogContent>
                    </AlertDialog>
                </div>
            </div>

            <p className="text-xs text-muted-foreground text-center mt-4">
                Made with ❤️ by{" "}
                <a
                    href="https://github.com/Ravish-Vishwakarma"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="underline hover:text-foreground transition-colors"
                >
                    ravish
                </a>
            </p>
        </div>
    )
}

export { SettingsPage }
