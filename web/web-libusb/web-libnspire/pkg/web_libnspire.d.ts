/* tslint:disable */
/* eslint-disable */
/**
*/
export function set_panic_hook(): void;
/**
*/
export class Calculator {
  free(): void;
/**
* @param {number} id
* @param {number} vid
* @param {number} pid
* @param {Int32Array} comm
*/
  constructor(id: number, vid: number, pid: number, comm: Int32Array);
/**
* @returns {Info}
*/
  update(): Info;
/**
* @param {string} path
* @returns {FileInfos}
*/
  list_dir(path: string): FileInfos;
/**
* @param {string} path
* @param {number} size
* @returns {Uint8Array}
*/
  download_file(path: string, size: number): Uint8Array;
/**
* @param {string} path
* @param {Uint8Array} bytes
*/
  upload_file(path: string, bytes: Uint8Array): void;
/**
* @param {Uint8Array} bytes
*/
  upload_os(bytes: Uint8Array): void;
/**
* @param {string} path
*/
  delete_file(path: string): void;
/**
* @param {string} path
*/
  delete_dir(path: string): void;
/**
* @param {string} path
*/
  create_dir(path: string): void;
/**
* @param {string} src
* @param {string} dest
*/
  copy_file(src: string, dest: string): void;
/**
* @param {string} src
* @param {string} dest
*/
  move_file(src: string, dest: string): void;
}
