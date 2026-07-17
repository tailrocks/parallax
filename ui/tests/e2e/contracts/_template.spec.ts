/**
 * Feature-plan template for Playwright product contracts.
 *
 * Copy to `<surface>.spec.ts`, register rows in `ui/test-matrix.json`, add a
 * dataset seed in `parallax_test_support::browser` when new fixtures are
 * required, and keep assertions in the scenario (screen objects only for
 * reused locators). This file is policy-checked but is not a counted test.
 *
 * Stable id form: `@pw-<surface>-<case>`
 */

import { productTest as test, expect } from "../fixtures/test"

// Intentionally empty: template is not selected as a real case.
// Feature plans add real tests in sibling `*.spec.ts` files.
void test
void expect
