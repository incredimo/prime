# Cognitive Load Governor (CLG)

The CLG converts the model's Brilliance Score **B ∈ [0,1]** into Modes **M1..M4** and enforces plan size, prompting style, and verification cadence.

## Modes
- **M4**: B ≥ 0.85
- **M3**: 0.70–0.84
- **M2**: 0.55–0.69
- **M1**: B < 0.55

Promote after **3 clean turns** (no guardrail hits).
Demote after **2 consecutive** guardrail hits or 1 severe policy breach.

## Mode → Technique Matrix

| Mode | Plan size                | Prompting style                                     | Verification          | Tool bias                                 | Sampling                                 |
| ---: | ------------------------ | --------------------------------------------------- | --------------------- | ----------------------------------------- | ---------------------------------------- |
|   M4 | macro-plan (4–8 actions) | concise CoT only where needed; structured rationale | spot-checks           | moderate; RAG/code when faster than text  | 1–2 passes                               |
|   M3 | medium batches (2–4)     | ReAct, brief self-ask, short CoT                    | verify critical steps | prefer RAG/code for math/lookup           | 2–3 passes on hard Qs                    |
|   M2 | small steps (1–2)        | ReAct + reflection; **short** reasoning only        | verify each step      | strong tool bias                          | self-consistency (3–5 bags)              |
|   M1 | micro-steps (single)     | template-driven; no free-form CoT                   | verify every step     | tool-first always                         | self-consistency (5–7), fallback recipes |

### Palette & Cadence Defaults
- **Tool palette caps:** M1–M2 → top 3–5 eligible tools; M3–M4 → top 6–8.
- **Verification cadence:**
  - M1/M2: verify **every** step (schema + task oracles).
  - M3: verify **critical** steps (mutations, external calls).
  - M4: **spot-checks** per policy.

Guardrail: **short reasoning bias**. Long CoT used only if SCP shows tolerance.
