# Session Capability Profiler (SCP)

SCP probes the connected model and computes a **Brilliance Score B ∈ [0,1]** to choose Mode (M1–M4).

## Signals
- Grammar Fidelity (UCM correctness)
- Instruction Following (attrs, brevity)
- Decomposition (ordered micro-steps)
- Grounded Reasoning (short logic)
- Self-Correction (fix after parser feedback)
- Tool-Use Readiness (ReAct “think→act→observe”)
- CoT Sensitivity (short vs long reasoning tolerance)
- Context Discipline (long prompt robustness)

## Minimal Probe Set (5)
1) Emit valid `get` + `result` pair (grammar)
2) Follow a 3-attribute instruction (instruction following)
3) Split a 3-step task (decomposition)
4) Fix a deliberately broken fence (self-correction)
5) Decide to call a tool (tool-use readiness)

Persist results at `.prime/sessions/<id>/caps.json` and `.prime/sessions/<id>/clg.json`.

## Scoring → Brilliance B

We compute **B ∈ [0,1]** as a weighted average of 5 probe scores:

| Signal                | Probe               | Weight | Scoring |
| --------------------- | ------------------- | :----: | ------- |
| Grammar Fidelity      | p1_grammar.md       |  0.30  | 1.0 perfect · 0.5 fixable · 0.0 invalid |
| Instruction Following | p2_instruction.md   |  0.20  | 1.0 exact · 0.0 otherwise |
| Decomposition         | p3_decompose.md     |  0.20  | 0.0–1.0 clarity & step quality |
| Self-Correction       | p4_fix_fence.md     |  0.15  | 1.0 fixed · 0.0 not fixed |
| Tool-Use Readiness    | p5_tool_decision.md |  0.15  | 1.0 correct choice · 0.0 otherwise |

**Formula:** `B = Σ(weight_i * score_i)` → Modes: M4 ≥ 0.85, M3 0.70–0.84, M2 0.55–0.69, M1 < 0.55.
