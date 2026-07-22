import { reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import type {
    GenericDevices,
    Device,
    DevId,
    FileInfo,
    Info,
    Progress,
    PartialCmd,
    Cmd,
} from 'n-link-core/components/devices';

function devToString(dev: DevId) {
    return `${dev.busNumber}:${dev.address}`;
}

function stringToDev(dev: string): DevId {
    const parts = dev.split(':');
    // eslint-disable-next-line
    return {busNumber: Number.parseInt(parts[0]), address: Number.parseInt(parts[1])};
}

async function downloadFile(dev: DevId | string, path: [string, number], dest: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await invoke('download_file', {...dev, path, dest});
}

async function uploadFile(dev: DevId | string, path: string, src: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await invoke('upload_file', {...dev, path, src});
}

async function uploadOs(dev: DevId | string, src: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await invoke('upload_os', {...dev, src});
}

async function deleteFile(dev: DevId | string, path: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await invoke('delete_file', {...dev, path});
}

async function deleteDir(dev: DevId | string, path: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await invoke('delete_dir', {...dev, path});
}

async function createDir(dev: DevId | string, path: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await invoke('create_nspire_dir', {...dev, path});
}

async function move(dev: DevId | string, src: string, dest: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await invoke('move_file', {...dev, src, dest});
}

async function copy(dev: DevId | string, src: string, dest: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await invoke('copy', {...dev, src, dest});
}

async function listDir(dev: DevId | string, path: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    return await invoke('list_dir', {...dev, path}) as FileInfo[];
}

async function listAll(dev: DevId | string, path: FileInfo): Promise<FileInfo[]> {
    if (!path.isDir) return [path];
    try {
        const contents = await listDir(dev, path.path);
        const parts: FileInfo[] = [];
        for (const file of contents) {
            parts.push(...(await listAll(dev, {...file, path: `${path.path}/${file.path}`})));
        }
        parts.push(path);
        return parts;
    } catch (e) {
        console.error(path, e);
        return [];
    }
}

let queueId = 0;

const state = reactive<GenericDevices>({
    devices: {},
    enumerating: false,
    hasEnumerated: false,

    async enumerate() {
        state.enumerating = true;
        try {
            for (const dev of await invoke('enumerate') as (Device & DevId)[]) {
                state.devices[devToString(dev as DevId)] = dev;
            }
        } finally {
            state.enumerating = false;
        }
    },

    async open(dev: DevId | string) {
        if (typeof dev === 'string') dev = stringToDev(dev);
        const info = await invoke<Info>('open_device', {...dev});
        state.devices[devToString(dev)].info = info;
    },

    async close(dev: DevId | string) {
        if (typeof dev === 'string') dev = stringToDev(dev);
        await invoke('close_device', {...dev});
        delete state.devices[devToString(dev)].info;
    },

    async update(dev: DevId | string) {
        if (typeof dev === 'string') dev = stringToDev(dev);
        const info = await invoke<Info>('update_device', {...dev});
        state.devices[devToString(dev)].info = info;
    },

    async listDir(dev: DevId | string, path: string) {
        return await listDir(dev, path);
    },

    async promptUploadFiles(dev: DevId | string, path: string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        const files = await openDialog({filters: [{extensions: ['tns'], name: 'TNS files'}], multiple: true});
        // The user can cancel the dialog, in which case `files` is null.
        // Iterating a null value here would throw at runtime.
        if (!files) return;
        for (const src of files) {
            addToQueue(dev, {action: 'upload', path, src});
        }
    },

    async uploadOs(dev: DevId | string, filter: string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        const src = await openDialog({filters: [{extensions: [filter], name: 'TI Nspire OS upgrade files'}]}) as string | null;
        if (!src) return;
        addToQueue(dev, {action: 'uploadOs', src});
    },

    async downloadFiles(dev: DevId | string, files: [string, number][]) {
        if (typeof dev !== 'string') dev = devToString(dev);
        const dest = await openDialog({directory: true}) as string | null;
        if (!dest) return;
        for (const path of files) {
            addToQueue(dev, {action: 'download', path, dest});
        }
    },

    async delete(dev: DevId | string, files: FileInfo[]) {
        if (typeof dev !== 'string') dev = devToString(dev);
        const toDelete: FileInfo[] = [];
        for (const file of files) {
            toDelete.push(...await listAll(dev, file));
        }
        for (const file of toDelete) {
            addToQueue(dev, {action: file.isDir ? 'deleteDir' : 'deleteFile', path: file.path});
        }
    },

    async createDir(dev: DevId | string, path: string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        addToQueue(dev, {action: 'createDir', path});
    },

    async copy(dev: DevId | string, src: string, dest: string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        addToQueue(dev, {action: 'copy', src, dest});
    },

    async move(dev: DevId | string, src: string, dest: string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        addToQueue(dev, {action: 'move', src, dest});
    },

    // The desktop app resolves uploads through a native OS file-picker dialog
    // (see promptUploadFiles/uploadOs above), which yields a filesystem path
    // that the Rust side reads directly. It has no way to turn a browser
    // File object back into such a path, so — exactly as in the original
    // Vue 2 desktop store — these two interface members are intentionally
    // left unimplemented here. The web host (n-link/web) provides the real
    // File-based implementation.
    async uploadFiles(): Promise<void> {
    },

    async uploadOsFile(): Promise<void> {
    },
});

async function runQueue(dev: DevId | string) {
    if (typeof dev !== 'string') dev = devToString(dev);
    const device = state.devices[dev];
    const queue = device?.queue;
    if (!queue || device.running) return;
    device.running = true;
    // eslint-disable-next-line no-constant-condition
    while (true) {
        // The device has been removed
        if (!state.devices[dev]) return;

        const cmd = queue[0];
        if (!cmd) {
            device.running = false;
            return;
        }
        try {
            if (cmd.action === 'download') {
                if ('dest' in cmd && cmd.dest) {
                    await downloadFile(dev, cmd.path, cmd.dest);
                } else {
                    console.error('Missing destination for download command', cmd);
                }
            } else if (cmd.action === 'upload') {
                if ('src' in cmd) {
                    await uploadFile(dev, cmd.path, cmd.src);
                } else {
                    console.error('File uploads are not supported by the desktop app', cmd);
                }
            } else if (cmd.action === 'uploadOs') {
                if ('src' in cmd) {
                    await uploadOs(dev, cmd.src);
                } else {
                    console.error('File uploads are not supported by the desktop app', cmd);
                }
            } else if (cmd.action === 'deleteFile') {
                await deleteFile(dev, cmd.path);
            } else if (cmd.action === 'deleteDir') {
                await deleteDir(dev, cmd.path);
            } else if (cmd.action === 'createDir') {
                await createDir(dev, cmd.path);
            } else if (cmd.action === 'move') {
                await move(dev, cmd.src, cmd.dest);
            } else if (cmd.action === 'copy') {
                await copy(dev, cmd.src, cmd.dest);
            }
        } catch (e) {
            console.error(e);
        }
        if ('progress' in device) delete device.progress;
        queue.shift();
        await state.update(dev);
    }
}

function addToQueue(dev: string, ...cmds: PartialCmd[]) {
    const device = state.devices[dev];
    if (!device) return;
    if (!device.queue) {
        device.queue = [];
    }
    device.queue.push(...cmds.map(cmd => ({...cmd, id: queueId++} as Cmd)));
    runQueue(dev);
}

state.enumerate().then(() => {
    state.hasEnumerated = true;
}, console.error);

listen('addDevice', dev => {
    const payload = dev.payload as Device & DevId;
    const str = devToString(payload);
    const existing = state.devices[str] || {};
    state.devices[str] = {...existing, ...payload};
});

listen('removeDevice', dev => {
    delete state.devices[devToString(dev.payload as DevId)];
});

listen('progress', dev => {
    const payload = dev.payload as Progress & DevId;
    const str = devToString(payload);
    state.devices[str].progress = payload;
});

export default state;
