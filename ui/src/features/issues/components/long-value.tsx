import { CopyButton } from "@/shared/console/copy-button"

const LONG_VALUE_LENGTH = 48

export function LongValue({ value }: { value: string }) {
  const copyable = value.length >= LONG_VALUE_LENGTH

  return (
    <span className="flex min-w-0 items-start gap-1">
      <span title={value} className="block min-w-0 font-mono break-all">
        {value}
      </span>
      {copyable ? <CopyButton value={value} /> : null}
    </span>
  )
}
