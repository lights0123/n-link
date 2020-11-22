import {Component, Vue} from 'vue-property-decorator';
import {RpcProvider} from 'worker-rpc';
import {saveAs} from 'file-saver';
import UsbWorker from 'worker-loader!@/components/usb.worker.ts';
import UsbCompat from '@/components/impl';
import {Cmd, FileInfo, GenericDevices, Info, PartialCmd, Progress,} from 'n-link-core/components/devices';
/// The USB vendor ID used by all Nspire calculators.
const VID = 0x0451;
/// The USB vendor ID used by all non-CX and original CX calculators.
const PID = 0xe012;
/// The USB vendor ID used by all CX II calculators.
const PID_CX2 = 0xe022;

async function promisified(...a: any[]): Promise<any> {
}

type WorkerExt = Worker & { rpc: RpcProvider };
export type Device = {
  device: USBDevice;
  name: string;
  isCxIi: boolean;
  needsDrivers: boolean;
  worker?: WorkerExt;
  info?: Info;
  progress?: Progress;
  queue?: Cmd[];
  running?: boolean;
};

async function downloadFile(
  dev: RpcProvider,
  path: [string, number]
) {
  const data: Uint8Array = await dev.rpc('downloadFile', {path});
  saveAs(new Blob([data]), path[0].split('/').pop());
}

async function uploadFile(dev: RpcProvider, path: string, data: Uint8Array) {
  await dev.rpc('uploadFile', {path, data});
}

async function uploadOs(dev: RpcProvider, data: Uint8Array) {
  await dev.rpc('uploadOs', {data});
}

async function deleteFile(dev: RpcProvider, path: string) {
  await dev.rpc('deleteFile', {path});
}

async function deleteDir(dev: RpcProvider, path: string) {
  await dev.rpc('deleteDir', {path});
}

async function createDir(dev: RpcProvider, path: string) {
  await dev.rpc('createDir', {path});
}

async function move(dev: RpcProvider, src: string, dest: string) {
  await dev.rpc('move', {src, dest});
}

async function copy(dev: RpcProvider, src: string, dest: string) {
  await dev.rpc('copy', {src, dest});
}

async function listDir(dev: RpcProvider, path: string) {
  return (await dev.rpc('listDir', {path})) as FileInfo[];
}

async function listAll(dev: RpcProvider, path: FileInfo): Promise<FileInfo[]> {
  if (!path.isDir) return [path];
  try {
    const contents = await listDir(dev, path.path);
    const parts: FileInfo[] = [];
    for (const file of contents) {
      parts.push(
        ...(await listAll(dev, {...file, path: `${path.path}/${file.path}`}))
      );
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
class Devices extends Vue implements GenericDevices {
  enumerating = false;
  hasEnumerated = false;
  devices: Record<string, Device> = {};

  created() {
  }

  async runQueue(dev: string) {
    const device = this.devices[dev];
    if (!device?.queue || !device.worker || device.running) return;
    const {rpc} = device.worker;
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
          await downloadFile(rpc, cmd.path);
        } else if (cmd.action === 'upload') {
          if (!('file' in cmd)) return;
          await uploadFile(rpc, `${cmd.path}/${cmd.file.name}`, new Uint8Array(await cmd.file.arrayBuffer()));
        } else if (cmd.action === 'uploadOs') {
          if (!('file' in cmd)) return;
          await uploadOs(rpc, new Uint8Array(await cmd.file.arrayBuffer()));
        } else if (cmd.action === 'deleteFile') {
          await deleteFile(rpc, cmd.path);
        } else if (cmd.action === 'deleteDir') {
          await deleteDir(rpc, cmd.path);
        } else if (cmd.action === 'createDir') {
          await createDir(rpc, cmd.path);
        } else if (cmd.action === 'move') {
          await move(rpc, cmd.src, cmd.dest);
        } else if (cmd.action === 'copy') {
          await copy(rpc, cmd.src, cmd.dest);
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
    device.queue?.push(
      ...cmds.map((cmd) => ({...cmd, id: queueId++} as Cmd))
    );
    this.runQueue(dev);
  }

  async enumerate() {
    try {
      if (!navigator.usb) return;
      const device = await navigator.usb.requestDevice({
        filters: [
          {vendorId: VID, productId: PID},
          {
            vendorId: VID,
            productId: PID_CX2,
          },
        ],
      });
      navigator.usb.ondisconnect = (e) => {
        const [key] =
        Object.entries(this.devices).find(
          ([_, {device}]) => device === e.device
        ) || [];
        if (key) {
          this.$delete(this.devices, key);
        }
      };
      this.$set(this.devices, queueId++, {
        device,
        name: device.productName,
        isCxIi: device.productId === PID_CX2,
        needsDrivers: false,
      } as Device);
    } catch (e) {
      console.error(e);
    }
  }

  async open(dev: string) {
    const device = this.devices[dev].device;
    await device.open();
    const worker: Worker & Partial<WorkerExt> = new UsbWorker();
    const sab = new SharedArrayBuffer(10000);
    const compat = new UsbCompat(sab);
    const id = compat.addDevice(device);
    const rpc = new RpcProvider((message, transfer: any) =>
      worker.postMessage(message, transfer)
    );
    worker.rpc = rpc;
    worker.onmessage = ({data}) => {
      if ('usbCmd' in data) return compat.processCmd(data);
      if('total' in data) {
        this.$set(this.devices[dev], 'progress', data);
        return;
      }
      rpc.dispatch(data);
    };
    this.$set(this.devices[dev], 'worker', worker as WorkerExt);

    await rpc.rpc('init', {id, sab, vid: device.vendorId, pid: device.productId});
    await this.update(dev);
  }

  async close(dev: string) {
    const device = this.devices[dev];
    device.worker?.terminate();
    device.device.close();
    this.$delete(this.devices, dev);
  }

  async update(dev: string) {
    console.log('up');
    const info = await this.devices[dev].worker?.rpc.rpc('updateDevice');
    console.log(info);
    this.$set(this.devices[dev], 'info', info);
  }

  async listDir(dev: string, path: string) {
    const worker = this.devices[dev].worker;
    if (!worker) return [];
    return await listDir(worker.rpc, path);
  }

  async uploadFiles(dev: string, path: string, files: File[]) {
    for (const file of files) {
      this.addToQueue(dev, {action: 'upload', path, file});
    }
  }

  async promptUploadFiles(dev: string, path: string) {
    throw new Error('Unimplemented');
  }

  async uploadOs(dev: string, filter: string) {
    throw new Error('Unimplemented');
  }

  async uploadOsFile(dev: string, file: File): Promise<void> {
    this.addToQueue(dev, {action: 'uploadOs', file});
  }

  async downloadFiles(dev: string, files: [string, number][]) {
    for (const path of files) {
      this.addToQueue(dev, {action: 'download', path});
    }
  }

  async delete(dev: string, files: FileInfo[]) {
    const rpc = this.devices[dev].worker?.rpc;
    if (!rpc) return;
    const toDelete: FileInfo[] = [];
    for (const file of files) {
      toDelete.push(...(await listAll(rpc, file)));
    }
    for (const file of toDelete) {
      this.addToQueue(dev, {
        action: file.isDir ? 'deleteDir' : 'deleteFile',
        path: file.path,
      });
    }
  }

  async createDir(dev: string, path: string) {
    this.addToQueue(dev, {action: 'createDir', path});
  }

  async copy(dev: string, src: string, dest: string) {
    this.addToQueue(dev, {action: 'copy', src, dest});
  }

  async move(dev: string, src: string, dest: string) {
    this.addToQueue(dev, {action: 'move', src, dest});
  }
}

const devices = new Devices();
export default devices;
Vue.prototype.$devices = devices;
