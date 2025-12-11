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
  closeEndpoint(): Promise<void>;
  closeConnection(): void;
  acceptConnection(): Promise<void>;
  sendControlFrame(data: Uint8Array): void;
  sendAudioFrame(data: Uint8Array): void;
  sendFrame(data: Uint8Array): void;
  notifyNewGop(): void;
  recv(): Uint8Array;
  asyncRecv(): Promise<Uint8Array>;
  beginWithGop(data: Uint8Array): void;
  connectionType(): string | undefined;
  roundTripTime(): number | undefined;
  currentPacketLoss(): number | undefined;
  networkChange(): void;
  getStats(): any;
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
  readonly ermiscall_spawn: (a: number, b: number, c: number, d: number) => number;
  readonly ermiscall_getLocalEndpointAddr: (a: number) => number;
  readonly ermiscall_connect: (a: number, b: number, c: number) => number;
  readonly ermiscall_closeEndpoint: (a: number) => number;
  readonly ermiscall_closeConnection: (a: number, b: number) => void;
  readonly ermiscall_acceptConnection: (a: number) => number;
  readonly ermiscall_sendControlFrame: (a: number, b: number, c: number, d: number) => void;
  readonly ermiscall_sendAudioFrame: (a: number, b: number, c: number, d: number) => void;
  readonly ermiscall_sendFrame: (a: number, b: number, c: number, d: number) => void;
  readonly ermiscall_notifyNewGop: (a: number, b: number) => void;
  readonly ermiscall_recv: (a: number, b: number) => void;
  readonly ermiscall_asyncRecv: (a: number) => number;
  readonly ermiscall_beginWithGop: (a: number, b: number, c: number, d: number) => void;
  readonly ermiscall_connectionType: (a: number, b: number) => void;
  readonly ermiscall_roundTripTime: (a: number, b: number) => void;
  readonly ermiscall_currentPacketLoss: (a: number, b: number) => void;
  readonly ermiscall_networkChange: (a: number) => void;
  readonly __wbg_connectionstats_free: (a: number, b: number) => void;
  readonly __wbg_get_connectionstats_rtt_ms: (a: number, b: number) => void;
  readonly __wbg_set_connectionstats_rtt_ms: (a: number, b: number, c: number) => void;
  readonly __wbg_get_connectionstats_packet_loss: (a: number, b: number) => void;
  readonly __wbg_set_connectionstats_packet_loss: (a: number, b: number, c: number) => void;
  readonly connectionstats_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly connectionstats_connection_type: (a: number, b: number) => void;
  readonly connectionstats_set_connection_type: (a: number, b: number, c: number) => void;
  readonly ermiscall_getStats: (a: number, b: number) => void;
  readonly __wbg_intounderlyingbytesource_free: (a: number, b: number) => void;
  readonly intounderlyingbytesource_type: (a: number) => number;
  readonly intounderlyingbytesource_autoAllocateChunkSize: (a: number) => number;
  readonly intounderlyingbytesource_start: (a: number, b: number) => void;
  readonly intounderlyingbytesource_pull: (a: number, b: number) => number;
  readonly intounderlyingbytesource_cancel: (a: number) => void;
  readonly __wbg_intounderlyingsource_free: (a: number, b: number) => void;
  readonly intounderlyingsource_pull: (a: number, b: number) => number;
  readonly intounderlyingsource_cancel: (a: number) => void;
  readonly __wbg_intounderlyingsink_free: (a: number, b: number) => void;
  readonly intounderlyingsink_write: (a: number, b: number) => number;
  readonly intounderlyingsink_close: (a: number) => number;
  readonly intounderlyingsink_abort: (a: number, b: number) => number;
  readonly ring_core_0_17_14__bn_mul_mont: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly __wasm_bindgen_func_elem_12858: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_12841: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_11688: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_11664: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_6230: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_6219: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_2287: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_2083: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_12912: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_12896: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_5748: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_5735: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_14376: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_export: (a: number, b: number) => number;
  readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export3: (a: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
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
