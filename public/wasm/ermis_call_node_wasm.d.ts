/* tslint:disable */
/* eslint-disable */
/**
 * The `ReadableStreamType` enum.
 *
 * *This API requires the following crate features to be activated: `ReadableStreamType`*
 */
type ReadableStreamType = "bytes";
export class ConnectionStats {
  free(): void;
  [Symbol.dispose](): void;
  constructor(connection_type?: string | null, rtt_ms?: number | null, packet_loss?: number | null);
  get rtt_ms(): number | undefined;
  set rtt_ms(value: number | null | undefined);
  get packet_loss(): number | undefined;
  set packet_loss(value: number | null | undefined);
  get connection_type(): string | undefined;
  set connection_type(value: string | null | undefined);
}
export class ErmisCall {
  free(): void;
  [Symbol.dispose](): void;
  constructor();
  spawn(relay_urls: any, secret_key?: Uint8Array | null): Promise<void>;
  getLocalEndpointAddr(): Promise<string>;
  connect(addr: string): Promise<void>;
  close(): void;
  acceptConnection(): Promise<void>;
  acceptBidiStream(): Promise<void>;
  openBidiStream(): Promise<void>;
  asyncSend(data: Uint8Array): Promise<void>;
  asyncRecv(): Promise<Uint8Array>;
  connectionType(): string | undefined;
  roundTripTime(): number | undefined;
  currentPacketLoss(): number | undefined;
  sendRaptorQ(data: Uint8Array): void;
  asyncSendRaptorQ(data: Uint8Array): Promise<void>;
  asyncRecvRaptorQ(): Promise<Uint8Array>;
  recvRaptorQ(): Uint8Array;
  getStats(): Promise<any>;
}
export class IntoUnderlyingByteSource {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  start(controller: ReadableByteStreamController): void;
  pull(controller: ReadableByteStreamController): Promise<any>;
  cancel(): void;
  readonly type: ReadableStreamType;
  readonly autoAllocateChunkSize: number;
}
export class IntoUnderlyingSink {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  write(chunk: any): Promise<any>;
  close(): Promise<any>;
  abort(reason: any): Promise<any>;
}
export class IntoUnderlyingSource {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  pull(controller: ReadableStreamDefaultController): Promise<any>;
  cancel(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_ermiscall_free: (a: number, b: number) => void;
  readonly ermiscall_new: () => number;
  readonly ermiscall_spawn: (a: number, b: any, c: number, d: number) => any;
  readonly ermiscall_getLocalEndpointAddr: (a: number) => any;
  readonly ermiscall_connect: (a: number, b: number, c: number) => any;
  readonly ermiscall_close: (a: number) => [number, number];
  readonly ermiscall_acceptConnection: (a: number) => any;
  readonly ermiscall_acceptBidiStream: (a: number) => any;
  readonly ermiscall_openBidiStream: (a: number) => any;
  readonly ermiscall_asyncSend: (a: number, b: number, c: number) => any;
  readonly ermiscall_asyncRecv: (a: number) => any;
  readonly ermiscall_connectionType: (a: number) => [number, number];
  readonly ermiscall_roundTripTime: (a: number) => [number, number];
  readonly ermiscall_currentPacketLoss: (a: number) => [number, number];
  readonly ermiscall_sendRaptorQ: (a: number, b: number, c: number) => [number, number];
  readonly ermiscall_asyncSendRaptorQ: (a: number, b: number, c: number) => any;
  readonly ermiscall_asyncRecvRaptorQ: (a: number) => any;
  readonly ermiscall_recvRaptorQ: (a: number) => [number, number, number, number];
  readonly __wbg_connectionstats_free: (a: number, b: number) => void;
  readonly __wbg_get_connectionstats_rtt_ms: (a: number) => [number, number];
  readonly __wbg_set_connectionstats_rtt_ms: (a: number, b: number, c: number) => void;
  readonly __wbg_get_connectionstats_packet_loss: (a: number) => [number, number];
  readonly __wbg_set_connectionstats_packet_loss: (a: number, b: number, c: number) => void;
  readonly connectionstats_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly connectionstats_connection_type: (a: number) => [number, number];
  readonly connectionstats_set_connection_type: (a: number, b: number, c: number) => void;
  readonly ermiscall_getStats: (a: number) => any;
  readonly __wbg_intounderlyingsink_free: (a: number, b: number) => void;
  readonly intounderlyingsink_write: (a: number, b: any) => any;
  readonly intounderlyingsink_close: (a: number) => any;
  readonly intounderlyingsink_abort: (a: number, b: any) => any;
  readonly __wbg_intounderlyingbytesource_free: (a: number, b: number) => void;
  readonly intounderlyingbytesource_type: (a: number) => number;
  readonly intounderlyingbytesource_autoAllocateChunkSize: (a: number) => number;
  readonly intounderlyingbytesource_start: (a: number, b: any) => void;
  readonly intounderlyingbytesource_pull: (a: number, b: any) => any;
  readonly intounderlyingbytesource_cancel: (a: number) => void;
  readonly __wbg_intounderlyingsource_free: (a: number, b: number) => void;
  readonly intounderlyingsource_pull: (a: number, b: any) => any;
  readonly intounderlyingsource_cancel: (a: number) => void;
  readonly ring_core_0_17_14__bn_mul_mont: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h71838f812387c6c5: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h351e5151eede5e72: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h9980edf9d8d47d64: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__h9c788f3e23312217: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h7cbb1ee88366865d: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h4402ffe5eeb05d40: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hce413bf567735f41: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__ha4bc4ea52f7ebfab: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__he279ee7597a37bfc: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__hc7df9f5060c1b308: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hf6c43e02bcdac30d: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hd682a2e286731212: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h68bb72b078dad8dc: (a: number, b: number) => number;
  readonly wasm_bindgen__convert__closures_____invoke__hbb82a021b1318aad: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
