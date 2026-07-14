// Generated from telemetry/semconv/contract.yaml.
// Run `cargo xtask semconv generate`; do not edit by hand.

export const SERVICE_NAME = "service.name" as const;
export const SERVICE_NAMESPACE = "service.namespace" as const;
export const SERVICE_INSTANCE_ID = "service.instance.id" as const;
export const SERVICE_VERSION = "service.version" as const;
export const DEPLOYMENT_ENVIRONMENT_NAME = "deployment.environment.name" as const;
export const DEPLOYMENT_ENVIRONMENT = "deployment.environment" as const;
export const TELEMETRY_SDK_LANGUAGE = "telemetry.sdk.language" as const;
export const TELEMETRY_SDK_NAME = "telemetry.sdk.name" as const;
export const TELEMETRY_SDK_VERSION = "telemetry.sdk.version" as const;
export const EVENT_NAME = "event.name" as const;
export const LOG_OBSERVED_TS_NANOS = "observed_ts_nanos" as const;
export const EXCEPTION_EVENT_NAME = "exception" as const;
export const EXCEPTION_TYPE = "exception.type" as const;
export const EXCEPTION_MESSAGE = "exception.message" as const;
export const EXCEPTION_STACKTRACE = "exception.stacktrace" as const;
export const EXCEPTION_ESCAPED = "exception.escaped" as const;
export const ERROR_TYPE = "error.type" as const;
export const PARALLAX_RUN_ID = "parallax.run.id" as const;
export const PARALLAX_SOURCE = "parallax.source" as const;
export const JACKIN_OPERATION = "jackin.operation" as const;
export const REQUEST_DURATION_METRICS = ["http.server.request.duration", "rpc.server.duration", ] as const;
export const CPU_METRICS = ["process.cpu.utilization", "process.cpu.usage", "system.cpu.utilization", ] as const;
export const MEMORY_METRICS = ["process.memory.usage", "process.memory.virtual", "system.memory.usage", ] as const;
export const BUNDLE_WINDOW_METRICS = ["process.cpu.utilization", "process.memory.usage", "tokio.runtime.alive_tasks", ] as const;
