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
  acceptConnection(): Promise<void>;
  acceptBidiStream(): Promise<void>;
  openBidiStream(): Promise<void>;
  asyncSend(data: Uint8Array): Promise<void>;
  asyncRecv(): Promise<Uint8Array>;
  connectionType(): Promise<string | undefined>;
  roundTripTime(): Promise<number | undefined>;
  currentPacketLoss(): Promise<number | undefined>;
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
  readonly ermiscall_spawn: (a: number, b: number, c: number, d: number) => number;
  readonly ermiscall_getLocalEndpointAddr: (a: number) => number;
  readonly ermiscall_connect: (a: number, b: number, c: number) => number;
  readonly ermiscall_acceptConnection: (a: number) => number;
  readonly ermiscall_acceptBidiStream: (a: number) => number;
  readonly ermiscall_openBidiStream: (a: number) => number;
  readonly ermiscall_asyncSend: (a: number, b: number, c: number) => number;
  readonly ermiscall_asyncRecv: (a: number) => number;
  readonly ermiscall_connectionType: (a: number) => number;
  readonly ermiscall_roundTripTime: (a: number) => number;
  readonly ermiscall_currentPacketLoss: (a: number) => number;
  readonly __wbg_connectionstats_free: (a: number, b: number) => void;
  readonly __wbg_get_connectionstats_rtt_ms: (a: number, b: number) => void;
  readonly __wbg_set_connectionstats_rtt_ms: (a: number, b: number, c: number) => void;
  readonly __wbg_get_connectionstats_packet_loss: (a: number, b: number) => void;
  readonly __wbg_set_connectionstats_packet_loss: (a: number, b: number, c: number) => void;
  readonly connectionstats_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly connectionstats_connection_type: (a: number, b: number) => void;
  readonly connectionstats_set_connection_type: (a: number, b: number, c: number) => void;
  readonly ermiscall_getStats: (a: number) => number;
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
  readonly __wasm_bindgen_func_elem_5585: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_5579: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_12782: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_12765: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_6164: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_6155: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_11619: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_11595: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_2210: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_2004: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_12836: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_12820: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_5674: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_5659: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_14156: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_export: (a: number, b: number) => number;
  readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export3: (a: number) => void;
  readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
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
