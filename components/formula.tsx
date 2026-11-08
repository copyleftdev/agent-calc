"use client";

import katex from "katex";
import { useMemo } from "react";

export function Formula({ expression }: { expression: string }) {
  const markup = useMemo(() => {
    if (!expression.trim()) return "";

    return katex.renderToString(expression, {
      displayMode: true,
      throwOnError: false,
      strict: "warn",
      trust: false,
      output: "htmlAndMathml",
    });
  }, [expression]);

  if (!markup) return null;

  return (
    <div
      className="formula"
      aria-label={`Mathematical expression: ${expression}`}
      dangerouslySetInnerHTML={{ __html: markup }}
    />
  );
}
