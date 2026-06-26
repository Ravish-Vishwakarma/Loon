import { Mic } from 'lucide-react';

function LauncherPage() {
    return (
        <>
            <div data-tauri-drag-region className='flex flex-col'>
                <Mic />
                Hello
            </div>
        </>
    )
}

export { LauncherPage }