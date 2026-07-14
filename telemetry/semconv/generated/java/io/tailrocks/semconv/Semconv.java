// Generated from telemetry/semconv/contract.yaml.
// Run `cargo xtask semconv generate`; do not edit by hand.
package io.tailrocks.semconv;

public final class Semconv {
    private Semconv() {}

    public static final String SERVICE_NAME = "service.name";
    public static final String SERVICE_NAMESPACE = "service.namespace";
    public static final String SERVICE_INSTANCE_ID = "service.instance.id";
    public static final String SERVICE_VERSION = "service.version";
    public static final String DEPLOYMENT_ENVIRONMENT_NAME = "deployment.environment.name";
    public static final String DEPLOYMENT_ENVIRONMENT = "deployment.environment";
    public static final String TELEMETRY_SDK_LANGUAGE = "telemetry.sdk.language";
    public static final String TELEMETRY_SDK_NAME = "telemetry.sdk.name";
    public static final String TELEMETRY_SDK_VERSION = "telemetry.sdk.version";
    public static final String EVENT_NAME = "event.name";
    public static final String LOG_OBSERVED_TS_NANOS = "observed_ts_nanos";
    public static final String EXCEPTION_EVENT_NAME = "exception";
    public static final String EXCEPTION_TYPE = "exception.type";
    public static final String EXCEPTION_MESSAGE = "exception.message";
    public static final String EXCEPTION_STACKTRACE = "exception.stacktrace";
    public static final String EXCEPTION_ESCAPED = "exception.escaped";
    public static final String ERROR_TYPE = "error.type";
    public static final String PARALLAX_RUN_ID = "parallax.run.id";
    public static final String PARALLAX_SOURCE = "parallax.source";
    public static final String JACKIN_OPERATION = "jackin.operation";
    public static final String[] REQUEST_DURATION_METRICS = {"http.server.request.duration", "rpc.server.duration", };
    public static final String[] CPU_METRICS = {"process.cpu.utilization", "process.cpu.usage", "system.cpu.utilization", };
    public static final String[] MEMORY_METRICS = {"process.memory.usage", "process.memory.virtual", "system.memory.usage", };
    public static final String[] BUNDLE_WINDOW_METRICS = {"process.cpu.utilization", "process.memory.usage", "tokio.runtime.alive_tasks", };
}
