/**
 * @zxv/agent-guard — the paraconsistent agent shield for NEAR.
 *
 * Evaluates conflicting LLM outputs or dual oracle feeds through the
 * ZXV four-valued gate. Contradictions are held as BOTH (0x03), never
 * coerced, never a panic; only TRUE settles.
 *
 * ```ts
 * import { ParaconsistentShield } from "@zxv/agent-guard";
 *
 * const shield = new ParaconsistentShield();
 * const verdict = await shield.evaluate(["yes", "no"]);
 * if (verdict.canSettle()) {
 *   await contract.executeTrade(...);
 * } else {
 *   console.log("Divergence contained: state held as BOTH (0x03)");
 * }
 * ```
 *
 * Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
 * SPDX-License-Identifier: Apache-2.0
 */

/** The four foundation states of the paraconsistent gate. */
export enum LogicState {
  NEITHER = 0x00,
  TRUE = 0x01,
  FALSE = 0x02,
  BOTH = 0x03,
}

/** The result of an evaluation: the output + the gate verdict. */
export interface Verdict {
  /** the resolved output (when TRUE) */
  output: string | null;
  /** the gate state */
  state: LogicState;
  /** true only when the state is TRUE */
  canSettle(): boolean;
  /** human-readable label */
  label: string;
}

function stateLabel(s: LogicState): string {
  switch (s) {
    case LogicState.NEITHER: return "NEITHER";
    case LogicState.TRUE: return "TRUE";
    case LogicState.FALSE: return "FALSE";
    default: return "BOTH";
  }
}

/**
 * The paraconsistent shield.
 *
 * - `evaluate(assertions)`: kjoin the evidence; agreement -> TRUE,
 *   divergence -> BOTH (held).
 * - `discover()`: read the ZXV discovery registry (`vm_capabilities`)
 *   so agents find the live subsystem accounts without config.
 */
export class ParaconsistentShield {
  readonly registry: string;
  readonly rpc: string;

  constructor(opts?: { registry?: string; rpc?: string }) {
    this.registry = opts?.registry ?? "7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a";
    this.rpc = opts?.rpc ?? "https://rpc.mainnet.fastnear.com";
  }

  /** The kernel's kjoin: (a | b) & 3. */
  static kjoin(a: LogicState, b: LogicState): LogicState {
    return ((a | b) & 0x03) as LogicState;
  }

  /**
   * Evaluate a set of assertions (LLM outputs, oracle prices, agent
   * reports). All-equal -> TRUE (settleable); any divergence -> BOTH
   * (held); empty -> NEITHER.
   */
  async evaluate(assertions: string[]): Promise<Verdict> {
    if (assertions.length === 0) {
      return this.verdict(null, LogicState.NEITHER);
    }
    const first = assertions[0];
    const agree = assertions.every((a) => a === first);
    return agree
      ? this.verdict(first, LogicState.TRUE)
      : this.verdict(null, LogicState.BOTH);
  }

  /**
   * Read the discovery registry: the live subsystem capability list.
   * Any agent can find the whole ZXV topology with one call — no config.
   */
  async discover(): Promise<string[]> {
    return this.view<string[]>(this.registry, "vm_capabilities", {});
  }

  /**
   * Resolve a named subsystem to its live contract account.
   */
  async resolve(name: string): Promise<string | null> {
    const cap = await this.view<{ contract?: string } | null>(
      this.registry, "vm_capability", { name },
    );
    return cap?.contract ?? null;
  }

  private verdict(output: string | null, state: LogicState): Verdict {
    const label = stateLabel(state);
    return {
      output,
      state,
      canSettle: () => state === LogicState.TRUE,
      label,
    };
  }

  /** Minimal JSON-RPC view call (no dependencies). */
  private async view<T>(accountId: string, method: string, args: Record<string, unknown>): Promise<T> {
    const argsBase64 = Buffer.from(JSON.stringify(args)).toString("base64");
    const body = {
      jsonrpc: "2.0",
      id: "zxv-guard",
      method: "query",
      params: {
        request_type: "call_function",
        finality: "final",
        account_id: accountId,
        method_name: method,
        args_base64: argsBase64,
      },
    };
    const res = await fetch(this.rpc, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const data = (await res.json()) as { result?: { result: number[] } };
    const bytes = data.result?.result ?? [];
    const text = Buffer.from(bytes).toString("utf8");
    return JSON.parse(text) as T;
  }
}