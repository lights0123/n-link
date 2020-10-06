export type DevId = { address: number; busNumber: number };

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

export type PartialCmd = { action: 'download'; path: [string, number]; dest?: string }
    | { action: 'upload'; path: string } & ({ src: string } | { file: File })
    | { action: 'uploadOs' } & ({ src: string } | { file: File })
    | { action: 'deleteFile'; path: string }
    | { action: 'deleteDir'; path: string }
    | { action: 'createDir'; path: string }
    | { action: 'move'; src: string; dest: string }
    | { action: 'copy'; src: string; dest: string };

export type Cmd = { id: number } & PartialCmd;

export type Device = { name: string; isCxIi: boolean; needsDrivers: boolean; info?: Info; progress?: Progress; queue?: Cmd[]; running?: boolean };

export interface GenericDevices {
    devices: Record<string, Device>;
    enumerating: boolean;
    hasEnumerated: boolean;

    enumerate(): Promise<void>;
    open(dev: string): Promise<void>;
    close(dev: string): Promise<void>;
    update(dev: string): Promise<void>;
    listDir(dev: DevId | string, path: string): Promise<FileInfo[]>;
    uploadFiles(dev: DevId | string, path: string, files: File[]): Promise<void>;
    promptUploadFiles(dev: DevId | string, path: string): Promise<void>;
    uploadOs(dev: DevId | string, filter: string): Promise<void>;
    uploadOsFile(dev: DevId | string, file: File): Promise<void>;
    downloadFiles(dev: DevId | string, files: [string, number][]): Promise<void>;
    delete(dev: DevId | string, files: FileInfo[]): Promise<void>;
    createDir(dev: DevId | string, path: string): Promise<void>;
    copy(dev: DevId | string, src: string, dest: string): Promise<void>;
    move(dev: DevId | string, src: string, dest: string): Promise<void>;
}

declare module 'vue/types/vue' {
    // 3. Declare augmentation for Vue
    interface Vue {
        $devices: GenericDevices;
    }
}
