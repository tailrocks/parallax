export function networkEscapeReason(input: RequestInfo | URL) {
  const target =
    typeof input === "string"
      ? input
      : input instanceof URL
        ? input.href
        : input.url
  return `unexpected test network request: ${target}`
}
