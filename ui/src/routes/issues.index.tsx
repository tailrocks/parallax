import { Link, createFileRoute } from "@tanstack/react-router"
import { graphql, relativeTime } from "@/lib/api"
import type { Issue } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

export const Route = createFileRoute("/issues/")({
  loader: () =>
    graphql<{ issues: Issue[] }>(`
      {
        issues {
          fingerprint
          title
          errorType
          culprit
          service
          status
          firstSeenNanos
          lastSeenNanos
          eventCount
          lastTraceId
        }
      }
    `),
  component: IssuesPage,
})

function IssuesPage() {
  const { issues } = Route.useLoaderData()
  return (
    <div className="space-y-4">
      <h1 className="text-lg font-semibold">Issues</h1>
      {issues.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No issues yet — connect an app with{" "}
          <code>OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317</code> and
          errors will group here.
        </p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Issue</TableHead>
              <TableHead className="w-24 text-right">Events</TableHead>
              <TableHead className="w-28">Last seen</TableHead>
              <TableHead className="w-28">Age</TableHead>
              <TableHead className="w-32">Service</TableHead>
              <TableHead className="w-24">Status</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {issues.map((issue) => (
              <TableRow key={issue.fingerprint}>
                <TableCell className="max-w-xl">
                  <Link
                    to="/issues/$fingerprint"
                    params={{ fingerprint: issue.fingerprint }}
                    className="block truncate font-medium hover:underline"
                  >
                    {issue.title}
                  </Link>
                  {issue.culprit ? (
                    <span className="block truncate text-xs text-muted-foreground">
                      {issue.culprit}
                    </span>
                  ) : null}
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  {issue.eventCount}
                </TableCell>
                <TableCell>{relativeTime(issue.lastSeenNanos)}</TableCell>
                <TableCell>{relativeTime(issue.firstSeenNanos)}</TableCell>
                <TableCell>
                  <Badge variant="outline">{issue.service}</Badge>
                </TableCell>
                <TableCell>
                  <Badge
                    variant={
                      issue.status === "open" ? "destructive" : "secondary"
                    }
                  >
                    {issue.status}
                  </Badge>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </div>
  )
}
