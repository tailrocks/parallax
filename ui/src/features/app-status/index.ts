// Public facade for app-status (Plan 143). Named exports only.

export { loadAppStatus } from "@/features/app-status/api/load-app-status"
export {
  classifyHealth,
  DEFAULT_ENDPOINT_LABEL,
} from "@/features/app-status/model/app-status"
export type { AppStatus } from "@/features/app-status/model/app-status"
