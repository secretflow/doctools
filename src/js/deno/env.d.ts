declare module "*.hbs" {
  const hbs: string;
  export default hbs;
}

declare module "*.wasm" {
  const bin: Uint8Array;
  export default bin;
}
