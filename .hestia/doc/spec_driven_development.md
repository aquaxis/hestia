# Spec-Driven Development

**Scope**: Spec-Driven Development (SDD)
**Source**: Design specification §1.3.1 (around lines 79-94), §3.6 (around lines 1078-1094), §19.1 (around lines 4021-4099)

---

## 1. Concept

A development methodology in which AI automatically generates design data (HDL code, constraint files, schematics, testbenches, etc.) from specifications written in natural language.

---

## 2. Why It Is Needed

In hardware design, translating specifications into HDL code requires enormous effort. Ambiguities and interpretation variations in specifications cause design errors. Automated AI conversion reduces effort and prevents divergence between specifications and implementations.

---

## 3. Functional Overview

1. The engineer writes the specification in natural language (using `REQ:` / `CON:` / `IF:` prefixes)
2. `SpecParser` converts the specification into a structured `DesignSpec`
3. The AI engine generates HDL code, constraint files, and testbenches from the `DesignSpec`
4. The generated code is quality-assured through an automated verification pipeline

```
Specification (natural language) → SpecParser → DesignSpec → AI generation engine → HDL / constraints / testbench
```

---

## 4. SDD Process (§19.1 Details)

```
┌─────────────────────────┐
│ 1. Specification (natural language) │  REQ: Clock frequency 100 MHz
│   REQ: / CON: / IF:      │  CON: Resource LUT < 5000
│                          │  IF: AXI4-Lite slave
└────────────┬─────────────┘
             ▼
┌─────────────────────────┐
│ 2. SpecParser            │  Input: text
│   Syntax parsing → AST   │  Output: DesignSpec { requirements: [...],
│                          │                     constraints: [...],
│                          │                     interfaces: [...] }
└────────────┬─────────────┘
             ▼
┌─────────────────────────┐
│ 3. AI generation engine  │  Skill invocation:
│   HDL generation /       │    - HDL generation (FR-AI-CONCEPT-02)
│   constraint generation  │    - Constraint generation (XDC / SDC)
│   / testbench generation │    - Testbench generation (FR-AI-PRAC-02)
└────────────┬─────────────┘
             ▼
┌─────────────────────────┐
│ 4. Automated verification │  - HDL static analysis (svls / veridian)
│   Syntax / types /       │  - Synthesis dry run (small configuration)
│   synthesizability        │  - Testbench execution (Verilator)
└────────────┬─────────────┘
             ▼
┌─────────────────────────┐
│ 5. Reverse verification  │  Re-extract specification from generated HDL,
│   (spec diff)            │  report differences against the original DesignSpec
│   Implementation → Spec   │
└─────────────────────────┘
```

---

## 5. SpecParser Specification → DesignSpec Conversion (§3.6)

`SpecParser` converts natural language specifications into structured `DesignSpec`. It automatically analyzes requirements, constraints, and interfaces using `REQ:` / `CON:` / `IF:` prefixes.

```rust
pub struct SpecParser;

impl SpecParser {
    pub fn parse(spec_text: &str) -> Result<DesignSpec, SpecError> {
        // Lines starting with REQ: → requirements
        // Lines starting with CON: → constraints
        // Lines starting with IF:  → interface definitions
        // Error if no mandatory requirements exist
    }
}
```

### DesignSpec AST (Key Fields)

```rust
pub struct DesignSpec {
    pub metadata:    SpecMetadata,        // source_path, author, timestamp
    pub requirements: Vec<Requirement>,   // Structured REQ: lines
    pub constraints:  Vec<Constraint>,    // Structured CON: lines
    pub interfaces:   Vec<InterfaceDecl>, // Structured IF: lines
    pub free_text:    String,             // Non-prefix body text
}

pub struct Requirement {
    pub id:          String,   // Auto-numbered (e.g., REQ-001)
    pub description: String,   // Body text
    pub priority:    Priority, // MUST / SHOULD / MAY
    pub tags:        Vec<String>,
}

pub struct Constraint {
    pub id:       String,   // CON-001
    pub kind:     ConstraintKind,  // Timing / Resource / Power / Area
    pub text:     String,
    pub numeric:  Option<NumericConstraint>, // e.g. LUT < 5000
}

pub struct InterfaceDecl {
    pub id:       String,            // IF-001
    pub name:     String,            // AXI4-Lite / APB / Wishbone
    pub role:     InterfaceRole,     // Master / Slave
    pub signals:  Vec<SignalDecl>,
}
```

The AST is serialized to JSON via `serde` and stored in `action-log`.

---

## 6. AI Generation Engine HDL / Constraint / Testbench Generation

The AI generation engine takes `DesignSpec` as input and generates the following through the skill system (§1.3.2):

- **HDL generation skill**: Automatic SystemVerilog / Verilog / VHDL source code generation
- **Constraint generation skill**: Automatic XDC / SDC / PCF constraint file generation
- **Testbench generation skill**: Automatic testbench skeleton + assertion + coverage generation

---

## 7. Automated Verification Pipeline

Generated code is quality-assured through the following automated verification pipeline:

1. **HDL static analysis**: Syntax and type checking via svls / veridian
2. **Synthesis dry run**: Synthesizability verification on a small configuration
3. **Testbench execution**: Simulation execution via Verilator
4. **Reverse verification**: Re-extract specifications from generated HDL and report differences against the original DesignSpec

If differences are detected in reverse verification, they are corrected through the `Feedback Loop` (§19.10) in the specification or generated artifacts.

---

## 8. REQ / CON / IF Prefix Conventions

Line-prefix markers in specifications that SpecParser uses to automatically extract structured data:

| Prefix | Corresponding structure | Content |
|--------------|----------|------|
| `REQ:` | `Requirement` | Requirement definition. Priority (MUST / SHOULD / MAY) can be attached |
| `CON:` | `Constraint` | Design constraint. Type (Timing / Resource / Power / Area) is auto-detected |
| `IF:` | `InterfaceDecl` | Interface definition. Name, role, and signal list are structured |

`SpecParser` returns an error if no mandatory requirements (MUST) exist.

---

## 9. Operational Rules

- Specifications are placed in `.hestia/specs/<project-name>.md`
- `SpecParser` returns an error if no mandatory requirements (MUST) exist
- Generated artifacts (HDL / constraints / testbenches) are placed in `.hestia/generated/<project-name>/` with the corresponding `DesignSpec` hash embedded as metadata
- If differences are detected in reverse verification, they are corrected through the `Feedback Loop` (§19.10) in the specification or generated artifacts

---

## 10. Implementation Location

`ai-conductor/crates/spec-driven/` (existing) + `spec-driven/src/parser.rs` extension.

---

## Related Documentation

- [Hestia Flow](hestia_flow.md) — Systematic guide to 9 AI utilization concepts (SDD corresponds to §1.3.1)
- [Architecture Overview](architecture_overview.md) — Position of design principle 7 "AI Utilization"
- [Shared Services](shared_services.md) — RAG-based specification-related information search and injection
- [Security](security.md) — Generated artifact intellectual property protection
- `.hestia/doc/ai/skills_system.md` — Skill system (HDL generation / constraint generation / testbench generation skills)
- `.hestia/doc/ai/workflow_engine.md` — Workflow engine (automated workflow execution including SDD)