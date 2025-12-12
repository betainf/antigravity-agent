import { describe, it, expect } from "vitest";
import { maskEmail, maskName } from "../string-masking.ts";

describe("maskEmail", () => {
  it("should return original string when not a basic valid email", () => {
    expect(maskEmail("not-an-email")).toBe("not-an-email");
    expect(maskEmail("no-at-symbol.com")).toBe("no-at-symbol.com");
    expect(maskEmail("@test.com")).toBe("@test.com");
    expect(maskEmail("user@")).toBe("user@");
    expect(maskEmail("")).toBe("");
  });

  it("should mask local part with length = 1", () => {
    // local: "a" -> "*"
    // domain: "b.c" (两段，视为最后一级域名，保持不变)
    expect(maskEmail("a@b.c")).toBe("*@b.c");
  });

  it("should fully mask local part with length = 2", () => {
    // "xx" -> "**"
    expect(maskEmail("xx@gmail.com")).toBe("**@gmail.com");
    expect(maskEmail("ab@b.com")).toBe("**@b.com");
  });

  it("should mask local part with length = 3", () => {
    // "abc" -> "a*c" （中间 1 个 *）
    expect(maskEmail("abc@b.com")).toBe("a*c@b.com");
  });

  it("should mask local part with length = 4", () => {
    // "abcd" -> "a**d" （中间 2 个 *）
    expect(maskEmail("abcd@test.com")).toBe("a**d@test.com");
  });

  it("should mask local part with length >= 5 and cap middle stars to 3", () => {
    // "abcdef" 长度 6 -> a + 3* + f
    expect(maskEmail("abcdef@gmail.com")).toBe("a***f@gmail.com");

    // 更长的也只显示 3 个 *
    expect(maskEmail("abcdefghijk@domain.com")).toBe("a***k@domain.com");
  });

  it("should keep entire domain when it has only one or two segments", () => {
    expect(maskEmail("foo@localhost")).toBe("f*o@localhost"); // 1 段域
    expect(maskEmail("user@test.io")).toBe("u**r@test.io");   // 2 段域
  });

  it("should keep last-level domain (last two segments) and mask all previous as **", () => {
    // abc@bin.gmail.com -> a*c@**.gmail.com
    expect(maskEmail("abc@bin.gmail.com")).toBe("a*c@**.gmail.com");

    // hello.world@sub.corp.company.com
    // local: "hello.world" -> h***d  （中间最多 3 个 *）
    // domain: "sub.corp.company.com"
    //   parts = ["sub", "corp", "company", "com"]
    //   last two = ["company", "com"]
    //   front -> ["**", "**"]
    expect(maskEmail("hello.world@sub.corp.company.com")).toBe(
      "h***d@**.**.company.com",
    );
  });

  it("should handle unicode characters in local part", () => {
    // "测a试" 长度 3 -> "测*试"
    expect(maskEmail("测a试@例子.公司")).toBe("测*试@例子.公司");
  });

  it("should treat '+' as normal character in local part and cap stars", () => {
    // "user+tag" 长度 8 -> u + 3* + g
    expect(maskEmail("user+tag@gmail.com")).toBe("u***g@gmail.com");
  });
});

describe("maskName", () => {
  it("should return empty string when username is empty", () => {
    expect(maskName("")).toBe("");
  });

  it("should mask single-character username", () => {
    expect(maskName("a")).toBe("*");
    expect(maskName("测")).toBe("*");
    expect(maskName("😀")).toBe("*");
  });

  it("should mask two-character username keeping first char", () => {
    expect(maskName("ab")).toBe("a*");
    expect(maskName("张三")).toBe("张*");
    expect(maskName("a😀")).toBe("a*");
  });

  it("should mask username with length >= 3 keeping first and last char", () => {
    expect(maskName("abc")).toBe("a*c");
    expect(maskName("abcdef")).toBe("a****f");
    expect(maskName("测试名")).toBe("测*名");
    expect(maskName("张三丰")).toBe("张*丰");
  });

  it("should handle unicode / emoji properly", () => {
    // 三个 emoji：😀😃😄
    expect(maskName("😀😃😄")).toBe("😀*😄");

    // 混合中文 + emoji
    expect(maskName("测😀试")).toBe("测*试");
  });

  it("should be effectively idempotent for already masked-like usernames", () => {
    // "a***z" 长度 5 => a + 3 * + z => "a***z"
    expect(maskName("a***z")).toBe("a***z");
  });

  it("should trim spaces before masking", () => {
    // " a" -> trim => "a" -> "*"
    expect(maskName(" a")).toBe("*");

    // "ab " -> trim => "ab" -> "a*"
    expect(maskName("ab ")).toBe("a*");

    // "  abc  " -> trim => "abc" -> "a*c"
    expect(maskName("  abc  ")).toBe("a*c");
  });
});;
