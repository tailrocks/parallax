// Plan 153 — client-reachable reexport of the sole storage owner.
// Implementation lives in browser-storage.ts so unit tests are not stubbed by
// the TanStack Start `.client.ts` virtual module.
export {
  readBrowserStorage,
  writeBrowserStorage,
  removeBrowserStorage,
  type BrowserStorage,
  type BrowserStorageKind,
} from "@/platform/storage/browser-storage"
