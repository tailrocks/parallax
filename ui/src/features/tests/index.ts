// Public facade for tests (Plan 155). Named exports only.

export { TestsContent, TestsPage } from "@/features/tests/components/tests-page"
export {
  TestCaseDetailContent,
  TestCaseDetailRoutePage,
} from "@/features/tests/components/test-case-detail-page"
export { loadTestCaseDetail, loadTests } from "@/features/tests/api/tests-api"
export { flakyLabel, rollupLabel, suiteLabel } from "@/features/tests/model/test-summary"
export type {
  TestExplorerRow,
  TestFlaky,
  TestResultRef,
  TestRollup,
  TestsData,
} from "@/features/tests/model/test-summary"
export { patchTestsSearch, validateTestsSearch } from "@/features/tests/model/tests-search"
export type {
  TestExplorerSort,
  TestFlakyFilter,
  TestRollupFilter,
  TestsSearch,
  TestsSearchPatch,
} from "@/features/tests/model/tests-search"
export { identitySourceLabel } from "@/features/tests/model/test-detail"
export type {
  TestCaseDetail,
  TestCaseDetailData,
  TestIdentitySource,
  TestVariantDetail,
} from "@/features/tests/model/test-detail"
export { TestsError } from "@/features/tests/model/tests-error"
