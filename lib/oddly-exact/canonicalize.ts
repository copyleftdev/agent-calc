type Expr = Record<string, unknown>;

class ExpressionParser {
  private position = 0;

  constructor(private readonly source: string) {}

  parse(): Expr {
    const expression = this.binary(0);
    this.space();
    if (this.position !== this.source.length) {
      throw new Error(`Unexpected token at character ${this.position + 1}.`);
    }
    return expression;
  }

  private binary(minimumPrecedence: number): Expr {
    let left = this.unary();

    while (true) {
      this.space();
      const operator = this.source[this.position];
      const precedence =
        operator === "+" || operator === "-"
          ? 1
          : operator === "*" || operator === "/"
            ? 2
            : operator === "^"
              ? 3
              : 0;
      const implicit = precedence === 0 && this.startsPrimary();
      const effectivePrecedence = implicit ? 2 : precedence;

      if (effectivePrecedence < minimumPrecedence || effectivePrecedence === 0) {
        break;
      }

      if (!implicit) this.position += 1;
      const right = this.binary(
        effectivePrecedence + (operator === "^" ? 0 : 1),
      );

      if (operator === "^") {
        if (right.kind !== "integer") {
          throw new Error("Oddly Exact requires an integer power exponent.");
        }
        left = {
          kind: "pow",
          base: left,
          exponent: Number(right.value),
        };
      } else {
        left = {
          kind:
            implicit || operator === "*"
              ? "mul"
              : operator === "/"
                ? "div"
                : operator === "+"
                  ? "add"
                  : "sub",
          left,
          right,
        };
      }
    }

    return left;
  }

  private unary(): Expr {
    this.space();
    if (this.source[this.position] === "-") {
      this.position += 1;
      return { kind: "neg", value: this.unary() };
    }
    if (this.source[this.position] === "+") {
      this.position += 1;
      return this.unary();
    }
    return this.primary();
  }

  private primary(): Expr {
    this.space();
    const character = this.source[this.position];

    if (character === "(") {
      this.position += 1;
      const expression = this.binary(0);
      this.space();
      if (this.source[this.position] !== ")") {
        throw new Error("Unclosed parenthesis in expression.");
      }
      this.position += 1;
      return expression;
    }

    const number = this.source
      .slice(this.position)
      .match(/^(?:\d+(?:\.\d*)?|\.\d+)/)?.[0];
    if (number) {
      this.position += number.length;
      if (!number.includes(".")) {
        return { kind: "integer", value: number };
      }
      const [whole, fraction = ""] = number.split(".");
      const numerator = `${whole || "0"}${fraction}`.replace(/^0+(?=\d)/, "");
      return {
        kind: "rational",
        numerator: numerator || "0",
        denominator: `1${"0".repeat(fraction.length)}`,
      };
    }

    const identifier = this.source
      .slice(this.position)
      .match(/^[A-Za-z_][A-Za-z0-9_]*/)?.[0];
    if (!identifier) {
      throw new Error(`Expected a number or symbol at character ${this.position + 1}.`);
    }
    this.position += identifier.length;
    this.space();

    if (this.source[this.position] !== "(") {
      return { kind: "symbol", name: identifier };
    }

    const supported = new Set([
      "sqrt",
      "exp",
      "ln",
      "sin",
      "cos",
      "tan",
      "abs",
      "floor",
      "ceil",
      "round",
    ]);
    if (!supported.has(identifier.toLowerCase())) {
      throw new Error(`Unsupported expression function ${identifier}.`);
    }
    this.position += 1;
    const value = this.binary(0);
    this.space();
    if (this.source[this.position] !== ")") {
      throw new Error(`Unclosed ${identifier} function.`);
    }
    this.position += 1;
    return { kind: identifier.toLowerCase(), value };
  }

  private startsPrimary(): boolean {
    this.space();
    return /[A-Za-z_(.\d]/.test(this.source[this.position] ?? "");
  }

  private space(): void {
    while (/\s/.test(this.source[this.position] ?? "")) this.position += 1;
  }
}

function parseEmbeddedJson(value: string): unknown {
  const trimmed = value.trim();
  if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) return value;
  try {
    return JSON.parse(trimmed);
  } catch {
    return value;
  }
}

export function parseMathExpression(value: string): Expr {
  const embedded = parseEmbeddedJson(value);
  if (embedded !== value && embedded && typeof embedded === "object") {
    return embedded as Expr;
  }
  return new ExpressionParser(value).parse();
}

function splitRelation(value: string) {
  const match = value.match(/^(.*?)(<=|>=|!=|=|<|>)(.*)$/);
  if (!match) throw new Error("Expected an equation or inequality relation.");
  return {
    left: match[1].trim(),
    operator: match[2],
    right: match[3].trim(),
  };
}

function canonicalEquation(value: string): Record<string, unknown> {
  const embedded = parseEmbeddedJson(value);
  if (embedded !== value && embedded && typeof embedded === "object") {
    const object = embedded as Record<string, unknown>;
    if (object.kind === "eq") {
      return { left: object.left, right: object.right };
    }
    return object;
  }
  const relation = splitRelation(value);
  if (relation.operator !== "=") throw new Error("Expected an equality.");
  return {
    left: parseMathExpression(relation.left),
    right: parseMathExpression(relation.right),
  };
}

function canonicalInequality(value: string): Record<string, unknown> {
  const embedded = parseEmbeddedJson(value);
  if (embedded !== value && embedded && typeof embedded === "object") {
    return embedded as Record<string, unknown>;
  }
  const relation = splitRelation(value);
  const names: Record<string, string> = {
    "<": "lt",
    "<=": "lte",
    ">": "gt",
    ">=": "gte",
    "=": "eq",
    "!=": "neq",
  };
  return {
    left: parseMathExpression(relation.left),
    relation: names[relation.operator],
    right: parseMathExpression(relation.right),
  };
}

const EXPRESSION_FIELDS = new Set([
  "expr",
  "expression",
  "lower",
  "upper",
]);

function normalizeStructuredShapes(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(normalizeStructuredShapes);
  if (!value || typeof value !== "object") return value;

  const object = value as Record<string, unknown>;
  if (object.kind === "pow") {
    const rawExponent = object.exponent ?? object.right;
    const exponent =
      rawExponent &&
      typeof rawExponent === "object" &&
      (rawExponent as Record<string, unknown>).kind === "integer"
        ? Number((rawExponent as Record<string, unknown>).value)
        : rawExponent;
    return {
      kind: "pow",
      base: normalizeStructuredShapes(object.base ?? object.left),
      exponent,
    };
  }

  return Object.fromEntries(
    Object.entries(object).map(([key, entry]) => [
      key,
      normalizeStructuredShapes(entry),
    ]),
  );
}

export function canonicalizeToolRequest(
  request: Record<string, unknown>,
): Record<string, unknown> {
  const canonicalize = (
    value: unknown,
    field?: string,
  ): unknown => {
    if (typeof value === "string") {
      if (field === "equation") return canonicalEquation(value);
      if (field === "inequality") return canonicalInequality(value);
      if (EXPRESSION_FIELDS.has(field ?? "")) return parseMathExpression(value);
      const embedded = parseEmbeddedJson(value);
      if (embedded !== value) return canonicalize(embedded, field);
      return value;
    }
    if (Array.isArray(value)) {
      return value.map((entry) => canonicalize(entry));
    }
    if (!value || typeof value !== "object") return value;

    const object = value as Record<string, unknown>;
    if (field === "equation" && object.kind === "eq") {
      return {
        left: canonicalize(object.left, "left"),
        right: canonicalize(object.right, "right"),
      };
    }
    if (object.kind === "pow") {
      const rawExponent = object.exponent ?? object.right;
      const exponent =
        rawExponent &&
        typeof rawExponent === "object" &&
        (rawExponent as Record<string, unknown>).kind === "integer"
          ? Number((rawExponent as Record<string, unknown>).value)
          : rawExponent;
      return {
        kind: "pow",
        base: canonicalize(object.base ?? object.left, "base"),
        exponent,
      };
    }

    return Object.fromEntries(
      Object.entries(object).map(([key, entry]) => [
        key,
        canonicalize(entry, key),
      ]),
    );
  };

  return normalizeStructuredShapes(
    canonicalize(request),
  ) as Record<string, unknown>;
}
