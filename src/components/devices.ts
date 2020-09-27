import {promisified} from 'tauri/api/tauri';
import {listen} from 'tauri/api/event';
import {Component, Vue} from 'vue-property-decorator';


export type DevId = { address: number; busNumber: number };

function devToString(dev: DevId) {
    return `${dev.busNumber}:${dev.address}`;
}

function stringToDev(dev: string): DevId {
    const parts = dev.split(':');
    // eslint-disable-next-line
    return {busNumber: Number.parseInt(parts[0]), address: Number.parseInt(parts[1])};
}

export type Version = { major: number; minor: number; patch: number; build: number };
export type Lcd = { width: number; height: number; bpp: number; sample_mode: number };
export type HardwareType =
    | "Cas"
    | "NonCas"
    | "CasCx"
    | "NonCasCx"
    | { Unknown: number };
// The current state of the calculator.
export type RunLevel =
    | "Recovery"
    | "Os"
    | { Unknown: number };
export type Battery =
    | "Powered"
    | "Low"
    | "Ok"
    | { Unknown: number };
export type Info = { free_storage: number; total_storage: number; free_ram: number; total_ram: number; version: Version; boot1_version: Version; boot2_version: Version; hw_type: HardwareType; clock_speed: number; lcd: Lcd; os_extension: string; file_extension: string; name: string; id: string; run_level: RunLevel; battery: Battery; is_charging: boolean };

export type FileInfo = { path: string; isDir: boolean; date: number; size: number };

export type Progress = { remaining: number; total: number };

export type PartialCmd = { action: 'download'; path: [string, number]; dest: string }
    | { action: 'upload'; path: string; src: string }
    | { action: 'uploadOs'; src: string }
    | { action: 'deleteFile'; path: string }
    | { action: 'deleteDir'; path: string }
    | { action: 'createDir'; path: string }
    | { action: 'move'; src: string; dest: string }
    | { action: 'copy'; src: string; dest: string };

export type Cmd = { id: number } & PartialCmd;

export type Device = { name: string; isCxIi: boolean; needsDrivers: boolean; info?: Info; progress?: Progress; queue?: Cmd[]; running?: boolean };

async function downloadFile(dev: DevId | string, path: [string, number], dest: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await promisified({...dev, cmd: 'downloadFile', path, dest});
}

async function uploadFile(dev: DevId | string, path: string, src: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await promisified({...dev, cmd: 'uploadFile', path, src});
}

async function uploadOs(dev: DevId | string, src: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await promisified({...dev, cmd: 'uploadOs', src});
}

async function deleteFile(dev: DevId | string, path: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await promisified({...dev, cmd: 'deleteFile', path});
}

async function deleteDir(dev: DevId | string, path: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await promisified({...dev, cmd: 'deleteDir', path});
}

async function createDir(dev: DevId | string, path: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await promisified({...dev, cmd: 'createNspireDir', path});
}

async function move(dev: DevId | string, src: string, dest: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await promisified({...dev, cmd: 'move', src, dest});
}

async function copy(dev: DevId | string, src: string, dest: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    await promisified({...dev, cmd: 'copy', src, dest});
}

async function listDir(dev: DevId | string, path: string) {
    if (typeof dev === 'string') dev = stringToDev(dev);
    return await promisified({...dev, cmd: 'listDir', path}) as FileInfo[];
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

@Component
class Devices extends Vue {
    devices: Record<string, Device> = {};
    enumerating = false;

    created() {
        this.enumerate().catch(console.error);
        listen('addDevice', dev => {
            const payload = dev.payload as Device & DevId;
            const str = devToString(payload);
            const existing = this.devices[str] || {};
            this.$set(this.devices, str, {...existing, ...payload});
        });
        listen('removeDevice', dev => {
            this.$delete(this.devices, devToString(dev.payload as DevId));
        });
        listen('progress', dev => {
            const payload = dev.payload as Progress & DevId;
            const str = devToString(payload);
            this.$set(this.devices[str], 'progress', payload);
        });
    }

    async runQueue(dev: DevId | string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        const device = this.devices[dev];
        if (!device?.queue || device.running) return;
        this.$set(device, 'running', true);
        // eslint-disable-next-line no-constant-condition
        while (true) {
            // The device has been removed
            if (!this.devices[dev]) return;

            const cmd = device.queue[0];
            if (!cmd) {
                device.running = false;
                return;
            }
            try {
                if (cmd.action === 'download') {
                    await downloadFile(dev, cmd.path, cmd.dest);
                } else if (cmd.action === 'upload') {
                    await uploadFile(dev, cmd.path, cmd.src);
                } else if (cmd.action === 'uploadOs') {
                    await uploadOs(dev, cmd.src);
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
            if ('progress' in device) this.$delete(device, 'progress');
            device.queue.shift();
            await this.update(dev);
        }
    }

    private addToQueue(dev: string, ...cmds: PartialCmd[]) {
        const device = this.devices[dev];
        if (!device) return;
        if (!device.queue) {
            this.$set(device, 'queue', []);
        }
        device.queue?.push(...cmds.map(cmd => ({...cmd, id: queueId++} as Cmd)));
        this.runQueue(dev);
    }

    async enumerate() {
        this.enumerating = true;
        try {
            for (const dev of await promisified({cmd: 'enumerate'}) as (Device & DevId)[]) {
                this.$set(this.devices, devToString(dev as DevId), dev);
            }
        } finally {
            this.enumerating = false;
        }
    }

    async open(dev: DevId | string) {
        if (typeof dev === 'string') dev = stringToDev(dev);
        const info = await promisified({...dev, cmd: 'openDevice'});
        this.$set(this.devices[devToString(dev)], 'info', info);
    }

    async close(dev: DevId | string) {
        if (typeof dev === 'string') dev = stringToDev(dev);
        await promisified({...dev, cmd: 'closeDevice'});
        this.$delete(this.devices[devToString(dev)], 'info');
    }

    async update(dev: DevId | string) {
        if (typeof dev === 'string') dev = stringToDev(dev);
        const info = await promisified({...dev, cmd: 'updateDevice'});
        this.$set(this.devices[devToString(dev)], 'info', info);
    }

    async listDir(dev: DevId | string, path: string) {
        return await listDir(dev, path);
    }

    async promptUploadFiles(dev: DevId | string, path: string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        const files = await promisified({cmd: 'selectFiles', filter: ['tns']}) as string[];
        for (const src of files) {
            this.addToQueue(dev, {action: 'upload', path, src});
        }
    }

    async uploadOs(dev: DevId | string, filter: string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        const src = await promisified({cmd: 'selectFile', filter: [filter]}) as string | null;
        if (!src) return;
        this.addToQueue(dev, {action: 'uploadOs', src});
    }

    async downloadFiles(dev: DevId | string, files: [string, number][]) {
        if (typeof dev !== 'string') dev = devToString(dev);
        const dest = await promisified({cmd: 'selectFolder'}) as string | null;
        if (!dest) return;
        for (const path of files) {
            this.addToQueue(dev, {action: 'download', path, dest});
        }
    }

    async delete(dev: DevId | string, files: FileInfo[]) {
        if (typeof dev !== 'string') dev = devToString(dev);
        const toDelete: FileInfo[] = [];
        for (const file of files) {
            toDelete.push(...await listAll(dev, file));
        }
        for (const file of toDelete) {
            this.addToQueue(dev, {action: file.isDir ? 'deleteDir' : 'deleteFile', path: file.path});
        }
    }

    async createDir(dev: DevId | string, path: string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        this.addToQueue(dev, {action: 'createDir', path});
    }

    async copy(dev: DevId | string, src: string, dest: string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        this.addToQueue(dev, {action: 'copy', src, dest});
    }

    async move(dev: DevId | string, src: string, dest: string) {
        if (typeof dev !== 'string') dev = devToString(dev);
        this.addToQueue(dev, {action: 'move', src, dest});
    }
}


const devices = new Devices();
export default devices;
Vue.prototype.$devices = devices;
declare module 'vue/types/vue' {
    // 3. Declare augmentation for Vue
    interface Vue {
        $devices: Devices;
    }
}
