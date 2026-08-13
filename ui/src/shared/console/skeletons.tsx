import { Skeleton } from "@/components/ui/skeleton"

const DEFAULT_COLUMNS = ["w-[28%]", "w-[18%]", "w-[18%]", "w-[36%]"] as const

export function TableSkeleton({
  rows = 8,
  columns = DEFAULT_COLUMNS,
}: {
  rows?: number
  columns?: readonly string[]
}) {
  return (
    <div className="overflow-hidden rounded-xl">
      <table className="w-full table-fixed">
        <tbody>
          {Array.from({ length: rows }, (_, row) => (
            <tr key={row}>
              {columns.map((width, index) => (
                <td key={index} className={`p-2 ${width}`}>
                  <Skeleton className="h-4 w-full" />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

export function CardsSkeleton({ count = 4 }: { count?: number }) {
  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      {Array.from({ length: count }, (_, index) => (
        <Skeleton key={index} className="h-32 w-full" />
      ))}
    </div>
  )
}
