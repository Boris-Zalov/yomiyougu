import { describe, it, expect } from "vitest";
import { capitalize } from "./theme";

describe("capitalize", () => {
  it("capitalizes first letter of a string", () => {
    expect(capitalize("hello")).toBe("Hello");
  });

  it("returns empty string for empty input", () => {
    expect(capitalize("")).toBe("");
  });

  it("handles single character", () => {
    expect(capitalize("a")).toBe("A");
  });

  it("handles already capitalized string", () => {
    expect(capitalize("Hello")).toBe("Hello");
  });

  it("only capitalizes first letter", () => {
    expect(capitalize("heLLO wOrLd")).toBe("HeLLO wOrLd");
  });
});
