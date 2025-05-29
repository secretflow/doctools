/** The global namespace where Deno specific, non-standard APIs are located. */
declare namespace Deno {
  /**
   * Options which can be set when using {@linkcode Deno.mkdir} and
   * {@linkcode Deno.mkdirSync}.
   *
   * @category File System */
  export interface MkdirOptions {
    /** If set to `true`, means that any intermediate directories will also be
     * created (as with the shell command `mkdir -p`).
     *
     * Intermediate directories are created with the same permissions.
     *
     * When recursive is set to `true`, succeeds silently (without changing any
     * permissions) if a directory already exists at the path, or if the path
     * is a symlink to an existing directory.
     *
     * @default {false} */
    recursive?: boolean;
    /** Permissions to use when creating the directory (defaults to `0o777`,
     * before the process's umask).
     *
     * Ignored on Windows. */
    mode?: number;
  }

  /** Creates a new directory with the specified path.
   *
   * ```ts
   * await Deno.mkdir("new_dir");
   * await Deno.mkdir("nested/directories", { recursive: true });
   * await Deno.mkdir("restricted_access_dir", { mode: 0o700 });
   * ```
   *
   * Defaults to throwing error if the directory already exists.
   *
   * Requires `allow-write` permission.
   *
   * @tags allow-write
   * @category File System
   */
  export function mkdir(path: string | URL, options?: MkdirOptions): Promise<void>;

  /** Removes the named file or directory.
   *
   * ```ts
   * await Deno.remove("/path/to/empty_dir/or/file");
   * await Deno.remove("/path/to/populated_dir/or/file", { recursive: true });
   * ```
   *
   * Throws error if permission denied, path not found, or path is a non-empty
   * directory and the `recursive` option isn't set to `true`.
   *
   * Requires `allow-write` permission.
   *
   * @tags allow-write
   * @category File System
   */
  export function remove(path: string | URL, options?: RemoveOptions): Promise<void>;

  /** Creates a new directory with the specified path.
   *
   * ```ts
   * await Deno.mkdir("new_dir");
   * await Deno.mkdir("nested/directories", { recursive: true });
   * await Deno.mkdir("restricted_access_dir", { mode: 0o700 });
   * ```
   *
   * Defaults to throwing error if the directory already exists.
   *
   * Requires `allow-write` permission.
   *
   * @tags allow-write
   * @category File System
   */
  export function mkdir(path: string | URL, options?: MkdirOptions): Promise<void>;

  /**
   * Options which can be set when using {@linkcode Deno.remove} and
   * {@linkcode Deno.removeSync}.
   *
   * @category File System */
  export interface RemoveOptions {
    /** If set to `true`, path will be removed even if it's a non-empty directory.
     *
     * @default {false} */
    recursive?: boolean;
  }

  /** Write string `data` to the given `path`, by default creating a new file if
   * needed, else overwriting.
   *
   * ```ts
   * await Deno.writeTextFile("hello1.txt", "Hello world\n");  // overwrite "hello1.txt" or create it
   * ```
   *
   * Requires `allow-write` permission, and `allow-read` if `options.create` is
   * `false`.
   *
   * @tags allow-read, allow-write
   * @category File System
   */
  export function writeTextFile(
    path: string | URL,
    data: string | ReadableStream<string>,
    options?: WriteFileOptions,
  ): Promise<void>;

  /** Write `data` to the given `path`, by default creating a new file if
   * needed, else overwriting.
   *
   * ```ts
   * const encoder = new TextEncoder();
   * const data = encoder.encode("Hello world\n");
   * await Deno.writeFile("hello1.txt", data);  // overwrite "hello1.txt" or create it
   * await Deno.writeFile("hello2.txt", data, { create: false });  // only works if "hello2.txt" exists
   * await Deno.writeFile("hello3.txt", data, { mode: 0o777 });  // set permissions on new file
   * await Deno.writeFile("hello4.txt", data, { append: true });  // add data to the end of the file
   * ```
   *
   * Requires `allow-write` permission, and `allow-read` if `options.create` is
   * `false`.
   *
   * @tags allow-read, allow-write
   * @category File System
   */
  export function writeFile(
    path: string | URL,
    data: Uint8Array | ReadableStream<Uint8Array>,
    options?: WriteFileOptions,
  ): Promise<void>;

  /** Options for writing to a file.
   *
   * @category File System
   */
  export interface WriteFileOptions {
    /** If set to `true`, will append to a file instead of overwriting previous
     * contents.
     *
     * @default {false} */
    append?: boolean;
    /** Sets the option to allow creating a new file, if one doesn't already
     * exist at the specified path.
     *
     * @default {true} */
    create?: boolean;
    /** If set to `true`, no file, directory, or symlink is allowed to exist at
     * the target location. When createNew is set to `true`, `create` is ignored.
     *
     * @default {false} */
    createNew?: boolean;
    /** Permissions always applied to file. */
    mode?: number;
    /** An abort signal to allow cancellation of the file write operation.
     *
     * If the signal becomes aborted the write file operation will be stopped
     * and the promise returned will be rejected with an {@linkcode AbortError}.
     */
    signal?: AbortSignal;
  }

  /** Asynchronously reads and returns the entire contents of a file as an UTF-8
   *  decoded string. Reading a directory throws an error.
   *
   * ```ts
   * const data = await Deno.readTextFile("hello.txt");
   * console.log(data);
   * ```
   *
   * Requires `allow-read` permission.
   *
   * @tags allow-read
   * @category File System
   */
  export function readTextFile(
    path: string | URL,
    options?: ReadFileOptions,
  ): Promise<string>;

  /** Reads and resolves to the entire contents of a file as an array of bytes.
   * `TextDecoder` can be used to transform the bytes to string if required.
   * Reading a directory returns an empty data array.
   *
   * ```ts
   * const decoder = new TextDecoder("utf-8");
   * const data = await Deno.readFile("hello.txt");
   * console.log(decoder.decode(data));
   * ```
   *
   * Requires `allow-read` permission.
   *
   * @tags allow-read
   * @category File System
   */
  export function readFile(
    path: string | URL,
    options?: ReadFileOptions,
  ): Promise<Uint8Array>;

  /**
   * Options which can be set when using {@linkcode Deno.readFile} or
   * {@linkcode Deno.readFileSync}.
   *
   * @category File System */
  export interface ReadFileOptions {
    /**
     * An abort signal to allow cancellation of the file read operation.
     * If the signal becomes aborted the readFile operation will be stopped
     * and the promise returned will be rejected with an AbortError.
     */
    signal?: AbortSignal;
  }

  /** Resolves to the absolute normalized path, with symbolic links resolved.
   *
   * ```ts
   * // e.g. given /home/alice/file.txt and current directory /home/alice
   * await Deno.symlink("file.txt", "symlink_file.txt");
   * const realPath = await Deno.realPath("./file.txt");
   * const realSymLinkPath = await Deno.realPath("./symlink_file.txt");
   * console.log(realPath);  // outputs "/home/alice/file.txt"
   * console.log(realSymLinkPath);  // outputs "/home/alice/file.txt"
   * ```
   *
   * Requires `allow-read` permission for the target path.
   *
   * Also requires `allow-read` permission for the `CWD` if the target path is
   * relative.
   *
   * @tags allow-read
   * @category File System
   */
  export function realPath(path: string | URL): Promise<string>;

  /** Serves HTTP requests with the given handler.
   *
   * The below example serves with the port `8000` on hostname `"127.0.0.1"`.
   *
   * ```ts
   * Deno.serve((_req) => new Response("Hello, world"));
   * ```
   *
   * @category HTTP Server
   */
  export function serve(handler: ServeHandler<Deno.NetAddr>): HttpServer<Deno.NetAddr>;
  /** Serves HTTP requests with the given option bag and handler.
   *
   * You can specify the socket path with `path` option.
   *
   * ```ts
   * Deno.serve(
   *   { path: "path/to/socket" },
   *   (_req) => new Response("Hello, world")
   * );
   * ```
   *
   * You can stop the server with an {@linkcode AbortSignal}. The abort signal
   * needs to be passed as the `signal` option in the options bag. The server
   * aborts when the abort signal is aborted. To wait for the server to close,
   * await the promise returned from the `Deno.serve` API.
   *
   * ```ts
   * const ac = new AbortController();
   *
   * const server = Deno.serve(
   *    { signal: ac.signal, path: "path/to/socket" },
   *    (_req) => new Response("Hello, world")
   * );
   * server.finished.then(() => console.log("Server closed"));
   *
   * console.log("Closing server...");
   * ac.abort();
   * ```
   *
   * By default `Deno.serve` prints the message
   * `Listening on path/to/socket` on listening. If you like to
   * change this behavior, you can specify a custom `onListen` callback.
   *
   * ```ts
   * Deno.serve({
   *   onListen({ path }) {
   *     console.log(`Server started at ${path}`);
   *     // ... more info specific to your server ..
   *   },
   *   path: "path/to/socket",
   * }, (_req) => new Response("Hello, world"));
   * ```
   *
   * @category HTTP Server
   */
  export function serve(
    options: ServeUnixOptions,
    handler: ServeHandler<Deno.UnixAddr>,
  ): HttpServer<Deno.UnixAddr>;
  /** Serves HTTP requests with the given option bag and handler.
   *
   * You can specify an object with a port and hostname option, which is the
   * address to listen on. The default is port `8000` on hostname `"0.0.0.0"`.
   *
   * You can change the address to listen on using the `hostname` and `port`
   * options. The below example serves on port `3000` and hostname `"127.0.0.1"`.
   *
   * ```ts
   * Deno.serve(
   *   { port: 3000, hostname: "127.0.0.1" },
   *   (_req) => new Response("Hello, world")
   * );
   * ```
   *
   * You can stop the server with an {@linkcode AbortSignal}. The abort signal
   * needs to be passed as the `signal` option in the options bag. The server
   * aborts when the abort signal is aborted. To wait for the server to close,
   * await the promise returned from the `Deno.serve` API.
   *
   * ```ts
   * const ac = new AbortController();
   *
   * const server = Deno.serve(
   *    { signal: ac.signal },
   *    (_req) => new Response("Hello, world")
   * );
   * server.finished.then(() => console.log("Server closed"));
   *
   * console.log("Closing server...");
   * ac.abort();
   * ```
   *
   * By default `Deno.serve` prints the message
   * `Listening on http://<hostname>:<port>/` on listening. If you like to
   * change this behavior, you can specify a custom `onListen` callback.
   *
   * ```ts
   * Deno.serve({
   *   onListen({ port, hostname }) {
   *     console.log(`Server started at http://${hostname}:${port}`);
   *     // ... more info specific to your server ..
   *   },
   * }, (_req) => new Response("Hello, world"));
   * ```
   *
   * To enable TLS you must specify the `key` and `cert` options.
   *
   * ```ts
   * const cert = "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----\n";
   * const key = "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n";
   * Deno.serve({ cert, key }, (_req) => new Response("Hello, world"));
   * ```
   *
   * @category HTTP Server
   */
  export function serve(
    options: ServeTcpOptions | (ServeTcpOptions & TlsCertifiedKeyPem),
    handler: ServeHandler<Deno.NetAddr>,
  ): HttpServer<Deno.NetAddr>;
  /** Serves HTTP requests with the given option bag.
   *
   * You can specify an object with the path option, which is the
   * unix domain socket to listen on.
   *
   * ```ts
   * const ac = new AbortController();
   *
   * const server = Deno.serve({
   *   path: "path/to/socket",
   *   handler: (_req) => new Response("Hello, world"),
   *   signal: ac.signal,
   *   onListen({ path }) {
   *     console.log(`Server started at ${path}`);
   *   },
   * });
   * server.finished.then(() => console.log("Server closed"));
   *
   * console.log("Closing server...");
   * ac.abort();
   * ```
   *
   * @category HTTP Server
   */
  export function serve(
    options: ServeUnixOptions & ServeInit<Deno.UnixAddr>,
  ): HttpServer<Deno.UnixAddr>;
  /** Serves HTTP requests with the given option bag.
   *
   * You can specify an object with a port and hostname option, which is the
   * address to listen on. The default is port `8000` on hostname `"0.0.0.0"`.
   *
   * ```ts
   * const ac = new AbortController();
   *
   * const server = Deno.serve({
   *   port: 3000,
   *   hostname: "127.0.0.1",
   *   handler: (_req) => new Response("Hello, world"),
   *   signal: ac.signal,
   *   onListen({ port, hostname }) {
   *     console.log(`Server started at http://${hostname}:${port}`);
   *   },
   * });
   * server.finished.then(() => console.log("Server closed"));
   *
   * console.log("Closing server...");
   * ac.abort();
   * ```
   *
   * @category HTTP Server
   */
  export function serve(
    options: (ServeTcpOptions | (ServeTcpOptions & TlsCertifiedKeyPem)) &
      ServeInit<Deno.NetAddr>,
  ): HttpServer<Deno.NetAddr>;

  /** A handler for HTTP requests. Consumes a request and returns a response.
   *
   * If a handler throws, the server calling the handler will assume the impact
   * of the error is isolated to the individual request. It will catch the error
   * and if necessary will close the underlying connection.
   *
   * @category HTTP Server
   */
  export type ServeHandler<Addr extends Deno.Addr = Deno.Addr> = (
    request: Request,
    info: ServeHandlerInfo<Addr>,
  ) => Response | Promise<Response>;

  /** Options which can be set when calling {@linkcode Deno.serve}.
   *
   * @category HTTP Server
   */
  export interface ServeOptions<Addr extends Deno.Addr = Deno.Addr> {
    /** An {@linkcode AbortSignal} to close the server and all connections. */
    signal?: AbortSignal;

    /** The handler to invoke when route handlers throw an error. */
    onError?: (error: unknown) => Response | Promise<Response>;

    /** The callback which is called when the server starts listening. */
    onListen?: (localAddr: Addr) => void;
  }

  /**
   * Options that can be passed to `Deno.serve` to create a server listening on
   * a TCP port.
   *
   * @category HTTP Server
   */
  export interface ServeTcpOptions extends ServeOptions<Deno.NetAddr> {
    /** The transport to use. */
    transport?: "tcp";

    /** The port to listen on.
     *
     * Set to `0` to listen on any available port.
     *
     * @default {8000} */
    port?: number;

    /** A literal IP address or host name that can be resolved to an IP address.
     *
     * __Note about `0.0.0.0`__ While listening `0.0.0.0` works on all platforms,
     * the browsers on Windows don't work with the address `0.0.0.0`.
     * You should show the message like `server running on localhost:8080` instead of
     * `server running on 0.0.0.0:8080` if your program supports Windows.
     *
     * @default {"0.0.0.0"} */
    hostname?: string;

    /** Sets `SO_REUSEPORT` on POSIX systems. */
    reusePort?: boolean;
  }

  /**
   * Options that can be passed to `Deno.serve` to create a server listening on
   * a Unix domain socket.
   *
   * @category HTTP Server
   */
  export interface ServeUnixOptions extends ServeOptions<Deno.UnixAddr> {
    /** The transport to use. */
    transport?: "unix";

    /** The unix domain socket path to listen on. */
    path: string;
  }

  /** Additional information for an HTTP request and its connection.
   *
   * @category HTTP Server
   */
  export interface ServeHandlerInfo<Addr extends Deno.Addr = Deno.Addr> {
    /** The remote address of the connection. */
    remoteAddr: Addr;
    /** The completion promise */
    completed: Promise<void>;
  }

  /**
   * @category HTTP Server
   */
  export interface ServeInit<Addr extends Deno.Addr = Deno.Addr> {
    /** The handler to invoke to process each incoming request. */
    handler: ServeHandler<Addr>;
  }

  /** An instance of the server created using `Deno.serve()` API.
   *
   * @category HTTP Server
   */
  export interface HttpServer<Addr extends Deno.Addr = Deno.Addr>
    extends AsyncDisposable {
    /** A promise that resolves once server finishes - eg. when aborted using
     * the signal passed to {@linkcode ServeOptions.signal}.
     */
    finished: Promise<void>;

    /** The local address this server is listening on. */
    addr: Addr;

    /**
     * Make the server block the event loop from finishing.
     *
     * Note: the server blocks the event loop from finishing by default.
     * This method is only meaningful after `.unref()` is called.
     */
    ref(): void;

    /** Make the server not block the event loop from finishing. */
    unref(): void;

    /** Gracefully close the server. No more new connections will be accepted,
     * while pending requests will be allowed to finish.
     */
    shutdown(): Promise<void>;
  }

  /** @category Network */
  export interface NetAddr {
    transport: "tcp" | "udp";
    hostname: string;
    port: number;
  }

  /** @category Network */
  export interface UnixAddr {
    transport: "unix" | "unixpacket";
    path: string;
  }

  /** @category Network */
  export type Addr = NetAddr | UnixAddr;

  /**
   * Provides certified key material from strings. The key material is provided in
   * `PEM`-format (Privacy Enhanced Mail, https://www.rfc-editor.org/rfc/rfc1422) which can be identified by having
   * `-----BEGIN-----` and `-----END-----` markers at the beginning and end of the strings. This type of key is not compatible
   * with `DER`-format keys which are binary.
   *
   * Deno supports RSA, EC, and PKCS8-format keys.
   *
   * ```ts
   * const key = {
   *  key: "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n",
   *  cert: "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----\n" }
   * };
   * ```
   *
   * @category Network
   */
  export interface TlsCertifiedKeyPem {
    /** The format of this key material, which must be PEM. */
    keyFormat?: "pem";
    /** Private key in `PEM` format. RSA, EC, and PKCS8-format keys are supported. */
    key: string;
    /** Certificate chain in `PEM` format. */
    cert: string;
  }
}
