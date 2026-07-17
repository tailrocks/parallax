import { describe, expect, it } from "vitest";

import {
  parseWhereClause,
  quoteWhereValue,
  serializeWhereClause,
  whereClauseFromSearch,
  type WhereFilter,
} from "../where-clause";

describe("parseWhereClause", () => {
  it("empty input parses to no filters", () => {
    expect(parseWhereClause("")).toEqual({ ok: true, filters: [] });
    expect(parseWhereClause("   ")).toEqual({ ok: true, filters: [] });
  });

  it("single equality", () => {
    expect(parseWhereClause('service = "checkout"')).toEqual({
      ok: true,
      filters: [{ key: "service", op: "=", value: "checkout" }],
    });
  });

  it("bare literal value", () => {
    expect(parseWhereClause("http.status_code >= 500")).toEqual({
      ok: true,
      filters: [{ key: "http.status_code", op: ">=", value: "500" }],
    });
  });

  it("AND chain with mixed operators", () => {
    expect(
      parseWhereClause(
        'service = "checkout" AND attr.http.route != "/health" AND duration > 100',
      ),
    ).toEqual({
      ok: true,
      filters: [
        { key: "service", op: "=", value: "checkout" },
        { key: "attr.http.route", op: "!=", value: "/health" },
        { key: "duration", op: ">", value: "100" },
      ],
    });
  });

  it("CONTAINS and NOT CONTAINS", () => {
    expect(parseWhereClause('body CONTAINS "timeout"')).toEqual({
      ok: true,
      filters: [{ key: "body", op: "CONTAINS", value: "timeout" }],
    });
    expect(parseWhereClause('body NOT CONTAINS "ok"')).toEqual({
      ok: true,
      filters: [{ key: "body", op: "NOT CONTAINS", value: "ok" }],
    });
  });

  it("case-insensitive keywords", () => {
    expect(
      parseWhereClause('a = 1 and b contains "x" AND c not contains "y"'),
    ).toEqual({
      ok: true,
      filters: [
        { key: "a", op: "=", value: "1" },
        { key: "b", op: "CONTAINS", value: "x" },
        { key: "c", op: "NOT CONTAINS", value: "y" },
      ],
    });
  });

  it("values with spaces and unicode", () => {
    expect(parseWhereClause('message = "café touché — done"')).toEqual({
      ok: true,
      filters: [{ key: "message", op: "=", value: "café touché — done" }],
    });
  });

  it("escaped quotes and backslashes", () => {
    expect(parseWhereClause('path = "C:\\\\tmp\\"x\\""')).toEqual({
      ok: true,
      filters: [{ key: "path", op: "=", value: 'C:\\tmp"x"' }],
    });
  });

  it("single-quoted strings", () => {
    expect(parseWhereClause("name = 'hello world'")).toEqual({
      ok: true,
      filters: [{ key: "name", op: "=", value: "hello world" }],
    });
  });

  it("unterminated string reports position", () => {
    expect(parseWhereClause('service = "checkout')).toMatchObject({
      ok: false,
      error: { message: "unterminated string", start: 10 },
    });
  });

  it("missing operator reports position", () => {
    expect(parseWhereClause("service checkout")).toMatchObject({
      ok: false,
      error: { message: expect.stringContaining("expected operator") },
    });
  });

  it("missing value after operator", () => {
    expect(parseWhereClause("service =")).toMatchObject({
      ok: false,
      error: { message: expect.stringContaining("expected value") },
    });
  });

  it("trailing AND rejected", () => {
    expect(parseWhereClause("a = 1 AND")).toMatchObject({
      ok: false,
      error: {
        message: expect.stringContaining("expected condition after AND"),
      },
    });
  });

  it("OR rejected — grammar is AND-only", () => {
    expect(parseWhereClause("a = 1 OR b = 2")).toMatchObject({
      ok: false,
      error: { message: expect.stringContaining("AND-only") },
    });
  });

  it("NOT without CONTAINS rejected", () => {
    expect(parseWhereClause("a NOT = 1")).toMatchObject({
      ok: false,
      error: {
        message: expect.stringContaining("expected CONTAINS after NOT"),
      },
    });
  });
});

describe("serializeWhereClause round-trip", () => {
  const cases: WhereFilter[][] = [
    [],
    [{ key: "service", op: "=", value: "checkout" }],
    [
      { key: "service", op: "=", value: "front end" },
      { key: "http.request.method", op: "!=", value: "GET" },
    ],
    [{ key: "body", op: "NOT CONTAINS", value: 'quoted "value"' }],
    [{ key: "msg", op: "CONTAINS", value: "café — unicode" }],
    [{ key: "duration", op: ">=", value: "1500" }],
    [{ key: "k", op: "=", value: "" }],
  ];

  for (const filters of cases) {
    it(`round-trips ${JSON.stringify(filters)}`, () => {
      const clause = serializeWhereClause(filters);
      expect(parseWhereClause(clause)).toEqual({ ok: true, filters });
    });
  }
});

describe("quoteWhereValue", () => {
  it("bare-safe values stay unquoted", () => {
    expect(quoteWhereValue("checkout")).toBe("checkout");
    expect(quoteWhereValue("500")).toBe("500");
    expect(quoteWhereValue("/health")).toBe("/health");
  });

  it("spaces and specials quoted", () => {
    expect(quoteWhereValue("hello world")).toBe('"hello world"');
    expect(quoteWhereValue("")).toBe('""');
    expect(quoteWhereValue('say "hi"')).toBe('"say \\"hi\\""');
  });
});

describe("whereClauseFromSearch", () => {
  it("undefined and invalid inputs yield empty list", () => {
    expect(whereClauseFromSearch(undefined)).toEqual([]);
    expect(whereClauseFromSearch("garbage ===")).toEqual([]);
  });

  it("valid clause parses", () => {
    expect(whereClauseFromSearch("a = 1")).toEqual([
      { key: "a", op: "=", value: "1" },
    ]);
  });
});
