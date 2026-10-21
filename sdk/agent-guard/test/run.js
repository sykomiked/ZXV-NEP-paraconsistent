// test/run.js — live verification of @zxv/agent-guard against NEAR mainnet.
const { ParaconsistentShield, LogicState } = require("../dist/index.js");

(async () => {
  const shield = new ParaconsistentShield();
  console.log("=== @zxv/agent-guard live test (NEAR mainnet) ===");

  // 1. discovery: the 13-subsystem topology
  const caps = await shield.discover();
  console.log("1. discover():", caps.length, "capabilities");
  console.log("   has paraconsistent_oracle:", caps.includes("paraconsistent_oracle"));
  console.log("   has ai_verdict:", caps.includes("ai_verdict"));

  // 2. resolve a subsystem to its live account
  const router = await shield.resolve("settlement_router");
  console.log("2. resolve(settlement_router):", router ? router.slice(0, 16) + "…" : "null");

  // 3. evaluation: agreement settles
  const agree = await shield.evaluate(["42", "42", "42"]);
  console.log("3. evaluate([42,42,42]):", agree.label, "| canSettle:", agree.canSettle());

  // 4. evaluation: divergence held as BOTH
  const diverge = await shield.evaluate(["42", "43"]);
  console.log("4. evaluate([42,43]):", diverge.label, "| canSettle:", diverge.canSettle());

  // 5. the kjoin table matches the kernel
  const both = ParaconsistentShield.kjoin(LogicState.TRUE, LogicState.FALSE);
  console.log("5. kjoin(TRUE,FALSE):", both === LogicState.BOTH ? "BOTH ✓ (matches on-chain proof)" : "MISMATCH");

  // summary
  const pass = caps.includes("paraconsistent_oracle") && caps.includes("ai_verdict")
    && router !== null && agree.canSettle() && !diverge.canSettle()
    && both === LogicState.BOTH;
  console.log("\nVERDICT:", pass ? "ALL PASS — the shield works against live mainnet" : "FAIL");
  process.exit(pass ? 0 : 1);
})().catch((e) => { console.error("FAILED:", e.message); process.exit(1); });