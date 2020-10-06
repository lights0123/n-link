/* eslint camelcase: 0, require-await: 0 */

import { RpcProvider } from 'worker-rpc';
// eslint-disable-next-line import/no-absolute-path
import type { Calculator } from 'web-libnspire';

console.log('worker!');
const ctx: Worker = self as any;
const module = import('web-libnspire');
let calc: Calculator | undefined;
const rpcProvider = new RpcProvider((message, transfer: any) =>
  ctx.postMessage(message, transfer)
);
ctx.onmessage = (e) => rpcProvider.dispatch(e.data);

type Path = { path: string };
type Data = { data: Uint8Array };
type SrcDest = { src: string; dest: string };

rpcProvider.registerRpcHandler<{id: number, sab: SharedArrayBuffer, vid: number, pid: number}>('init', async ({ id, sab, vid, pid }) => {
  if (calc) calc.free();
  calc = new (await module).Calculator(id, vid, pid, new Int32Array(sab));
});

rpcProvider.registerRpcHandler('updateDevice', async () => {
  return calc?.update();
});

rpcProvider.registerRpcHandler<{path: [string, number]}, Uint8Array | undefined>('downloadFile', async ({ path }) => {
  return calc?.download_file(path[0], path[1]);
});

rpcProvider.registerRpcHandler<Path & Data>(
  'uploadFile',
  async ({ path, data }) => {
    calc?.upload_file(path, data);
  }
);

rpcProvider.registerRpcHandler<Data>('uploadOs', async ({ data }) => {
  calc?.upload_os(data);
});

rpcProvider.registerRpcHandler<Path>('deleteFile', async ({ path }) => {
  calc?.delete_file(path);
});

rpcProvider.registerRpcHandler<Path>('deleteDir', async ({ path }) => {
  calc?.delete_dir(path);
});

rpcProvider.registerRpcHandler<Path>('createDir', async ({ path }) => {
  calc?.create_dir(path);
});

rpcProvider.registerRpcHandler<SrcDest>('move', async ({ src, dest }) => {
  calc?.move_file(src, dest);
});

rpcProvider.registerRpcHandler<SrcDest>('copy', async ({ src, dest }) => {
  calc?.copy_file(src, dest);
});

rpcProvider.registerRpcHandler<Path>('listDir', async ({ path }) => {
  return calc?.list_dir(path);
});
