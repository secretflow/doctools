// @ts-no-check

/**
 * Main module of the runtime.
 *
 * Initializes the globalThis object with the necessary Web APIs.
 *
 * Heavily-adapted from [deno_runtime].
 *
 * [deno_runtime]: https://github.com/denoland/deno/blob/v1.39.4/runtime
 *
 * @see https://docs.rs/deno_core/0.245.0/deno_core/struct.JsRuntime.html#method.load_main_module
 * @see https://github.com/denoland/deno/blob/v1.39.4/runtime/js/98_global_scope_shared.js
 * @see https://github.com/denoland/deno/blob/v1.39.4/runtime/js/99_main.js
 */

import { core } from "ext:core/mod.js";
import * as console from "ext:deno_console/01_console.js";
import * as crypto from "ext:deno_crypto/00_crypto.js";
import * as url from "ext:deno_url/00_url.js";
import * as urlPattern from "ext:deno_url/01_urlpattern.js";
import { DOMException } from "ext:deno_web/01_dom_exception.js";
import * as event from "ext:deno_web/02_event.js";
import * as timers from "ext:deno_web/02_timers.js";
import * as abortSignal from "ext:deno_web/03_abort_signal.js";
import * as base64 from "ext:deno_web/05_base64.js";
import * as streams from "ext:deno_web/06_streams.js";
import * as encoding from "ext:deno_web/08_text_encoding.js";
import * as file from "ext:deno_web/09_file.js";
import * as fileReader from "ext:deno_web/10_filereader.js";
import * as compression from "ext:deno_web/14_compression.js";
import * as performance from "ext:deno_web/15_performance.js";
import * as imageData from "ext:deno_web/16_image_data.js";
import * as webidl from "ext:deno_webidl/00_webidl.js";

// not including these in globalThis, but
// all JS files must be evaluated during v8 snapshotting
import "ext:deno_web/04_global_interfaces.js";
import "ext:deno_web/12_location.js";
import "ext:deno_web/13_message_port.js";

((globalThis) => {
  delete globalThis.__bootstrap;
  // @ts-expect-error https://github.com/denoland/deno/blob/be4d0b7ad306312b0c01e1abc345e54b4e589ab6/runtime/js/99_main.js#L548
  delete Object.prototype.__proto__;

  /**
   * @param {unknown} value
   * @returns {PropertyDescriptor}
   */
  const frozen = (value) => ({
    enumerable: false,
    configurable: false,
    writable: false,
    value,
  });

  /**
   * @param {() => unknown} fn
   * @returns {PropertyDescriptor}
   */
  const getter = (fn) => ({
    enumerable: false,
    configurable: false,
    get: fn,
  });

  Object.defineProperties(globalThis, {
    console: frozen(
      new console.Console(
        /**
         * @param {string} msg
         * @param {number} level
         * @returns {void}
         * @see https://choubey.gitbook.io/internals-of-deno/bridge/4.2-print#registration
         */
        (msg, level) => core.print(msg, level > 1),
      ),
    ),
  });

  Object.defineProperties(globalThis, {
    crypto: frozen(crypto.crypto),
    Crypto: frozen(crypto.Crypto),
    CryptoKey: frozen(crypto.CryptoKey),
    SubtleCrypto: frozen(crypto.SubtleCrypto),
  });

  Object.defineProperties(globalThis, {
    URL: frozen(url.URL),
    URLPattern: frozen(urlPattern.URLPattern),
    URLSearchParams: frozen(url.URLSearchParams),
  });

  Object.defineProperties(globalThis, {
    Blob: frozen(file.Blob),
    File: frozen(file.File),
    FileReader: frozen(fileReader.FileReader),
  });

  Object.defineProperties(globalThis, {
    AbortController: frozen(abortSignal.AbortController),
    AbortSignal: frozen(abortSignal.AbortSignal),
  });

  Object.defineProperties(globalThis, {
    CloseEvent: frozen(event.CloseEvent),
    CustomEvent: frozen(event.CustomEvent),
    ErrorEvent: frozen(event.ErrorEvent),
    Event: frozen(event.Event),
    EventTarget: frozen(event.EventTarget),
    MessageEvent: frozen(event.MessageEvent),
    ProgressEvent: frozen(event.ProgressEvent),
    PromiseRejectionEvent: frozen(event.PromiseRejectionEvent),
  });

  Object.defineProperties(globalThis, {
    ByteLengthQueuingStrategy: frozen(streams.ByteLengthQueuingStrategy),
    CountQueuingStrategy: frozen(streams.CountQueuingStrategy),
    ReadableStream: frozen(streams.ReadableStream),
    ReadableStreamDefaultReader: frozen(streams.ReadableStreamDefaultReader),
    TransformStream: frozen(streams.TransformStream),
    WritableStream: frozen(streams.WritableStream),
    WritableStreamDefaultWriter: frozen(streams.WritableStreamDefaultWriter),
    WritableStreamDefaultController: frozen(streams.WritableStreamDefaultController),
    ReadableByteStreamController: frozen(streams.ReadableByteStreamController),
    ReadableStreamBYOBReader: frozen(streams.ReadableStreamBYOBReader),
    ReadableStreamBYOBRequest: frozen(streams.ReadableStreamBYOBRequest),
    ReadableStreamDefaultController: frozen(streams.ReadableStreamDefaultController),
    TransformStreamDefaultController: frozen(streams.TransformStreamDefaultController),
  });

  Object.defineProperties(globalThis, {
    CompressionStream: frozen(compression.CompressionStream),
    DecompressionStream: frozen(compression.DecompressionStream),
  });

  Object.defineProperties(globalThis, {
    Performance: frozen(performance.Performance),
    PerformanceEntry: frozen(performance.PerformanceEntry),
    PerformanceMark: frozen(performance.PerformanceMark),
    PerformanceMeasure: frozen(performance.PerformanceMeasure),
  });

  Object.defineProperties(globalThis, {
    TextDecoder: frozen(encoding.TextDecoder),
    TextEncoder: frozen(encoding.TextEncoder),
    TextDecoderStream: frozen(encoding.TextDecoderStream),
    TextEncoderStream: frozen(encoding.TextEncoderStream),
  });

  Object.defineProperties(globalThis, {
    DOMException: frozen(DOMException),
    ImageData: frozen(imageData.ImageData),
    atob: frozen(base64.atob),
    btoa: frozen(base64.btoa),
    clearInterval: frozen(timers.clearInterval),
    clearTimeout: frozen(timers.clearTimeout),
    performance: frozen(performance.performance),
    reportError: frozen(event.reportError),
    setInterval: frozen(timers.setInterval),
    setTimeout: frozen(timers.setTimeout),
    [webidl.brand]: frozen(webidl.brand),
  });

  Object.defineProperties(globalThis, {
    self: getter(() => globalThis),
  });

  event.setEventTargetData(globalThis);
  event.saveGlobalThisReference(globalThis);

  event.defineEventHandler(globalThis, "error");
  event.defineEventHandler(globalThis, "load");
  event.defineEventHandler(globalThis, "beforeunload");
  event.defineEventHandler(globalThis, "unload");
  event.defineEventHandler(globalThis, "unhandledrejection");

  core.setMacrotaskCallback(timers.handleTimerMacrotask);
  core.setReportExceptionCallback(event.reportException);

  const versions = core.ops.op_snapshot_versions();

  core.setBuildInfo(versions.target);
  Object.defineProperties(globalThis, {
    __version__: frozen(versions),
  });

  performance.setTimeOrigin(Date.now());
})(globalThis);
