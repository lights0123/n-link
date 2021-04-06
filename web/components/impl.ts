import { Encoder } from '@msgpack/msgpack';

export enum UsbError {
  NotFound = 'NotFound',
  Security = 'Security',
  Network = 'Network',
  Abort = 'Abort',
  InvalidState = 'InvalidState',
  InvalidAccess = 'InvalidAccess',
  Unknown = 'Unknown',
}

const exceptionMap: Record<string, string> = globalThis.DOMException
  ? {
      NotFoundError: UsbError.NotFound,
      SecurityError: UsbError.Security,
      NetworkError: UsbError.Network,
      AbortError: UsbError.Abort,
      InvalidStateError: UsbError.InvalidState,
      InvalidAccessError: UsbError.InvalidAccess,
    }
  : {};

export type Cmd =
  | ({ usbCmd: 'bulkTransferOut' } & BulkTransferOut)
  | ({ usbCmd: 'bulkTransferIn' } & BulkTransferIn)
  | ({ usbCmd: 'selectConfiguration' } & SelectConfiguration)
  | ({ usbCmd: 'claimInterface' } & ClaimInterface)
  | ({ usbCmd: 'releaseInterface' } & ReleaseInterface)
  | ({ usbCmd: 'resetDevice' } & ResetDevice)
  | ({ usbCmd: 'activeConfigDescriptor' } & ActiveConfigDescriptor);

export type NullReply = { Ok: null } | { Err: UsbError };

export type BulkTransferOut = {
  device: number;
  endpoint: number;
  data: Uint8Array;
};

export type BulkTransferOutReply = { Ok: number } | { Err: UsbError };

export type BulkTransferIn = {
  device: number;
  endpoint: number;
  length: number;
};

export type Data = Uint8Array;

export type BulkTransferInReply = { Ok: Data } | { Err: UsbError };

export type SelectConfiguration = { device: number; config: number };

export type ClaimInterface = { device: number; number: number };

export type ReleaseInterface = { device: number; number: number };

export type ResetDevice = { device: number };

export type ActiveConfigDescriptor = { device: number };

export type USBEndpoint = { address: number; packetSize: number };

export type USBAlternateInterface = {
  alternateSetting: number;
  interfaceClass: number;
  interfaceSubclass: number;
  interfaceProtocol: number;
  endpoints: USBEndpoint[];
};

export type USBConfiguration = {
  configurationValue: number;
  interfaces: USBAlternateInterface[][];
};

export type ActiveConfigDescriptorReply =
  | { Ok: USBConfiguration }
  | { Err: UsbError };

const encoder = new Encoder();

let count = 0;
export default class UsbCompat {
  devices: Record<number, USBDevice> = {};
  arr: SharedArrayBuffer;
  lastError?: DOMException;

  constructor(arr: SharedArrayBuffer) {
    this.arr = arr;
  }

  addDevice(dev: USBDevice) {
    const i = count++;
    this.devices[i] = dev;
    return i;
  }

  private async _processCmd(cmd: Cmd) {
    try {
      if (cmd.usbCmd === 'bulkTransferOut') {
        const res = await this.devices[cmd.device].transferOut(
          cmd.endpoint & ~0x80,
          cmd.data
        );
        const reply: BulkTransferOutReply = { Ok: res.bytesWritten };
        return reply;
      } else if (cmd.usbCmd === 'bulkTransferIn') {
        const res = await this.devices[cmd.device].transferIn(
          cmd.endpoint & ~0x80,
          cmd.length
        );
        const reply: BulkTransferInReply = {
          Ok: new Uint8Array(res.data!.buffer),
        };
        return reply;
      } else if (cmd.usbCmd === 'selectConfiguration') {
        await this.devices[cmd.device].selectConfiguration(cmd.config);
      } else if (cmd.usbCmd === 'claimInterface') {
        await this.devices[cmd.device].claimInterface(cmd.number);
      } else if (cmd.usbCmd === 'releaseInterface') {
        await this.devices[cmd.device].releaseInterface(cmd.number);
      } else if (cmd.usbCmd === 'resetDevice') {
        await this.devices[cmd.device].reset();
      } else if (cmd.usbCmd === 'activeConfigDescriptor') {
        const configuration = this.devices[cmd.device].configuration!;
        const reply: ActiveConfigDescriptorReply = {
          Ok: {
            configurationValue: configuration.configurationValue,
            interfaces: configuration.interfaces.map(({ alternates }) => {
              return alternates.map((alternate) => ({
                alternateSetting: alternate.alternateSetting,
                interfaceClass: alternate.interfaceClass,
                interfaceSubclass: alternate.interfaceSubclass,
                interfaceProtocol: alternate.interfaceProtocol,
                endpoints: alternate.endpoints.map((endpoint) => ({
                  address:
                    endpoint.endpointNumber |
                    (endpoint.direction === 'in' ? 0x80 : 0),
                  packetSize: endpoint.packetSize,
                })),
              }));
            }),
          },
        };
        return reply;
      }
      const reply: NullReply = { Ok: null };
      return reply;
    } catch (e) {
      console.error(e);
      this.lastError = e;
      return {
        Err: { [exceptionMap[e.name as string] || UsbError.Unknown]: null },
      };
    }
  }

  async processCmd(cmd: Cmd) {
    const msg = await this._processCmd(cmd);
    const encoded = encoder.encode(msg);

    if (encoded.length > this.arr.length - 4) {
      throw new Error('too long');
    }
    new Uint8Array(this.arr).set(encoded, 4);
    const notify = new Int32Array(this.arr);
    Atomics.store(notify, 0, encoded.length);
    Atomics.notify(notify, 0, Infinity);
  }
}
