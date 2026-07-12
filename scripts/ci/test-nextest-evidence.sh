#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
fixture="$root/scripts/fixtures/nextest-evidence"
evidence="$root/target/nextest"
state=$(mktemp)
rm "$state"
trap 'rm -f "$state"' EXIT

run_fixture() {
  local profile=$1
  local test_name=$2
  local expected=$3
  local destination="$evidence/fixture-$test_name"
  rm -rf "$fixture/target/nextest/$profile" "$destination"
  set +e
  PARALLAX_NEXTEST_FIXTURE_STATE="$state" cargo nextest run \
    --manifest-path "$fixture/Cargo.toml" --profile "$profile" \
    --no-tests=fail -E "test(/::$test_name$/)"
  local status=$?
  set -e
  mkdir -p "$destination"
  cp "$fixture/target/nextest/$profile/junit.xml" "$destination/junit.xml"
  if [[ $expected == pass ]]; then
    [[ $status -eq 0 ]] || return 1
    cargo xtask nextest-evidence --profile "fixture-$test_name"
  else
    [[ $status -ne 0 ]] || return 1
    if cargo xtask nextest-evidence --profile "fixture-$test_name"; then
      printf 'invalid %s evidence passed validation\n' "$test_name" >&2
      return 1
    fi
  fi
}

run_fixture fixture pass pass
run_fixture retry retry_pass fail
run_fixture fixture persistent_fail fail
run_fixture fixture slow_pass pass
run_fixture timeout timeout fail

if cargo nextest run --manifest-path "$fixture/Cargo.toml" --profile fixture \
  --no-tests=fail -E 'test(/::missing_test$/)'; then
  printf 'zero-test selection passed\n' >&2
  exit 1
fi

mkdir -p "$evidence/fixture-malformed"
printf '<testsuites tests="1"' > "$evidence/fixture-malformed/junit.xml"
if cargo xtask nextest-evidence --profile fixture-malformed; then
  printf 'malformed JUnit passed validation\n' >&2
  exit 1
fi

printf 'nextest-generated evidence fixtures passed\n'
