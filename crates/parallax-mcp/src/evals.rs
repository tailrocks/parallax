//! Cost-gated MCP tool-selection evals (plan 171 feature 3).
//!
//! Ignored by default. Run only when opted in:
//! `cargo nextest run -p parallax-mcp --run-ignored only -E 'test(/eval/)'`
//! Absent `ANTHROPIC_API_KEY` skips with a notice and does not fail.

use serde_json::{Value, json};

const THRESHOLD: usize = 7;
const MODEL: &str = "claude-sonnet-4-5";

struct Scenario {
    name: &'static str,
    prompt: &'static str,
    tool: &'static str,
    arg: &'static str,
    value: &'static str,
}

const SCENARIOS: &[Scenario] = &[
    Scenario {
        name: "issue_context_happy",
        prompt: "Give me the evidence bundle for issue fingerprint abcdef0123456789.",
        tool: "parallax_issue_context",
        arg: "fingerprint",
        value: "abcdef0123456789",
    },
    Scenario {
        name: "issue_context_ambiguous",
        prompt: "Something is wrong with checkout; start from fingerprint deadbeefdeadbeef.",
        tool: "parallax_issue_context",
        arg: "fingerprint",
        value: "deadbeefdeadbeef",
    },
    Scenario {
        name: "session_show_happy",
        prompt: "Show the agent session timeline for invocation inv-42.",
        tool: "parallax_agent_session_show",
        arg: "invocation_id",
        value: "inv-42",
    },
    Scenario {
        name: "session_show_alias",
        prompt: "What tool steps did the coding agent take in invocation run-7?",
        tool: "parallax_agent_session_show",
        arg: "invocation_id",
        value: "run-7",
    },
    Scenario {
        name: "issue_context_bundle_wording",
        prompt: "Fetch the canonical redacted bundle for fingerprint 1111222233334444.",
        tool: "parallax_issue_context",
        arg: "fingerprint",
        value: "1111222233334444",
    },
    Scenario {
        name: "session_show_tokens",
        prompt: "How many tokens did the agent use in invocation sess-9?",
        tool: "parallax_agent_session_show",
        arg: "invocation_id",
        value: "sess-9",
    },
    Scenario {
        name: "distractor_write",
        prompt: "Open a pull request that fixes fingerprint cafe0000cafe0000.",
        tool: "parallax_issue_context",
        arg: "fingerprint",
        value: "cafe0000cafe0000",
    },
    Scenario {
        name: "distractor_delete",
        prompt: "Delete issue fingerprint 0000aaaabbbbcccc from the store.",
        tool: "parallax_issue_context",
        arg: "fingerprint",
        value: "0000aaaabbbbcccc",
    },
];

fn tools() -> Value {
    json!([
        {
            "name": "parallax_issue_context",
            "description": "Canonical evidence bundle for an issue fingerprint.",
            "input_schema": {
                "type": "object",
                "properties": { "fingerprint": { "type": "string" } },
                "required": ["fingerprint"],
                "additionalProperties": false
            }
        },
        {
            "name": "parallax_agent_session_show",
            "description": "Sanitized agent-session timeline for an invocation id.",
            "input_schema": {
                "type": "object",
                "properties": { "invocation_id": { "type": "string" } },
                "required": ["invocation_id"],
                "additionalProperties": false
            }
        }
    ])
}

fn score(scenario: &Scenario, name: &str, input: &Value) -> bool {
    name == scenario.tool && input.get(scenario.arg).and_then(Value::as_str) == Some(scenario.value)
}

async fn run_eval(scenario: &Scenario) -> bool {
    let key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(value) if !value.is_empty() => value,
        _ => {
            eprintln!(
                "SKIP eval {} — ANTHROPIC_API_KEY unset (notice, not failure)",
                scenario.name
            );
            return true;
        }
    };
    let client = reqwest::Client::new();
    let body = json!({
        "model": std::env::var("PARALLAX_MCP_EVAL_MODEL").unwrap_or_else(|_| MODEL.into()),
        "max_tokens": 256,
        "tools": tools(),
        "tool_choice": { "type": "any" },
        "messages": [{ "role": "user", "content": scenario.prompt }]
    });
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("anthropic request");
    let status = response.status();
    let payload: Value = response.json().await.expect("anthropic json");
    if !status.is_success() {
        eprintln!("eval {} HTTP {status}: {payload}", scenario.name);
        return false;
    }
    let Some(block) = payload
        .get("content")
        .and_then(Value::as_array)
        .and_then(|blocks| {
            blocks
                .iter()
                .find(|block| block.get("type").and_then(Value::as_str) == Some("tool_use"))
        })
    else {
        eprintln!("eval {} no tool_use: {payload}", scenario.name);
        return false;
    };
    let name = block.get("name").and_then(Value::as_str).unwrap_or("");
    let input = block.get("input").cloned().unwrap_or(json!({}));
    let ok = score(scenario, name, &input);
    if !ok {
        eprintln!(
            "eval {} missed: expected {} {}={} got {name} {input}",
            scenario.name, scenario.tool, scenario.arg, scenario.value
        );
    }
    ok
}

macro_rules! eval_case {
    ($fn_name:ident, $index:expr) => {
        #[tokio::test]
        #[ignore = "cost-gated MCP eval; set ANTHROPIC_API_KEY and --run-ignored"]
        async fn $fn_name() {
            assert!(run_eval(&SCENARIOS[$index]).await);
        }
    };
}

eval_case!(eval_issue_context_happy, 0);
eval_case!(eval_issue_context_ambiguous, 1);
eval_case!(eval_session_show_happy, 2);
eval_case!(eval_session_show_alias, 3);
eval_case!(eval_issue_context_bundle_wording, 4);
eval_case!(eval_session_show_tokens, 5);
eval_case!(eval_distractor_write, 6);
eval_case!(eval_distractor_delete, 7);

#[tokio::test]
#[ignore = "cost-gated MCP eval threshold"]
async fn eval_threshold_at_least_seven_of_eight() {
    if std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .is_none_or(|value| value.is_empty())
    {
        eprintln!("SKIP eval threshold — ANTHROPIC_API_KEY unset");
        return;
    }
    let mut passed = 0;
    for scenario in SCENARIOS {
        if run_eval(scenario).await {
            passed += 1;
        }
    }
    assert!(
        passed >= THRESHOLD,
        "MCP eval threshold: {passed}/{} (need {THRESHOLD})",
        SCENARIOS.len()
    );
}
