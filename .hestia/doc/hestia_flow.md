# Hestia Flow — Systematic Guide to AI Utilization Concepts

**Scope**: AI utilization concepts and development process
**Source**: Design specification §1.3 (around lines 75-409), §19 (around lines 4002-5201)

---

## Overview

AI utilization in Hestia consists of 9 concepts. Below is a systematic explanation of the definition, purpose, and functionality of each concept.

---

## 1. Spec-Driven Development (§1.3.1 / §19.1)

### Concept

A development methodology in which AI automatically generates design data (HDL code, constraint files, schematics, testbenches, etc.) from specifications written in natural language.

### Why It Is Needed

In hardware design, translating specifications into HDL code requires enormous effort. Ambiguities and interpretation variations in specifications cause design errors. Automated AI conversion reduces effort and prevents divergence between specifications and implementations.

### How It Works

1. The engineer writes the specification in natural language (using `REQ:` / `CON:` / `IF:` prefixes)
2. `SpecParser` converts the specification into a structured `DesignSpec`
3. The AI engine generates HDL code, constraint files, and testbenches from the `DesignSpec`
4. The generated code is quality-assured through an automated verification pipeline

```
Specification (natural language) → SpecParser → DesignSpec → AI generation engine → HDL / constraints / testbench
```

### SDD Process (§19.1 Details)

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

## 2. Skill System (§1.3.2 / §3.7)

### Concept

A mechanism for registering, managing, and invoking the specialized design capabilities of AI agents as plugins.

### Why It Is Needed

Hardware design requires diverse specialized knowledge including circuit design, HDL generation, constraint generation, and testbench generation. Managing each skill as an independent plugin enables flexible addition, updating, and combination of skills.

### How It Works

1. Register skills in `SkillRegistry` (implementing the `Skill` trait)
2. ai-conductor dynamically launches agents with the required skills
3. Agents execute skills and report results to ai-conductor

### Default Skills

| Skill | Input | Output |
|--------|------|------|
| HDL generation | Specification / block diagram | SystemVerilog / Verilog / VHDL source code |
| Constraint generation | Target device / timing requirements | XDC / SDC / PCF constraint files |
| Testbench generation | HDL module definition | Testbench + assertions + coverage |
| Schematic design | Natural language specification / KG | SKiDL code / KiCad netlist |
| Review | HDL code / schematic | Review results + modification suggestions |

---

## 3. Skeleton-Driven Development (§1.3.3)

### Concept

A development methodology in which AI automatically generates project templates or module skeletons, and engineers implement the details.

### Why It Is Needed

Setting up new projects or adding new modules requires creating boilerplate code (port definitions, clock/reset connections, basic state machine structures, etc.), which is time-consuming. Automatic skeleton generation allows engineers to focus on implementing core logic.

### How It Works

1. The engineer specifies a module overview (name, inputs/outputs, functionality)
2. AI generates a project template or module skeleton
3. The generated output includes port definitions, state machine templates, interface definitions, and constraint file templates
4. The engineer implements core logic in the generated skeleton

```
Module overview → AI skeleton generation → Port definitions + SM template + Constraint template → Engineer implementation
```

---

## 4. Test-Driven Development for Hardware (§1.3.4 / §19.2)

### Concept

A hardware version of test-driven development where AI generates testbenches first, followed by design implementation.

### Why It Is Needed

In hardware design, verification accounts for 60-70% of the entire process. Creating testbenches is labor-intensive but essential for quality assurance. AI-automated testbench generation significantly reduces verification effort and improves test coverage.

### How It Works

1. AI automatically generates testbenches from specifications or module interfaces
2. Testbenches include stimulus generation, expected value comparison, assertions, and coverage points
3. Engineers implement the DUT (Design Under Test)
4. A feedback loop iterates design improvements until tests pass

```
Specification → AI testbench generation → Test execution → FAIL → Design improvement → Test execution → PASS
```

### TDD Process (§19.2 Details)

```
Specification or module interface
        ↓
AI testbench generation (skill-system)
        ├── Stimulus generation (boundary values / random / sequences)
        ├── Expected value generation (reference model / specification formulas)
        ├── Assertion generation (SVA / psl)
        └── Coverage point generation (covergroup / cross)
        ↓
(Test-first) Test execution → FAIL (DUT not implemented)
        ↓
Design implementation (engineer or AI generation)
        ↓
Test execution → PASS/FAIL
        ↓ On FAIL
Feedback Loop (§19.10)
  - Fix by PatcherAgent / engineer
        ↓
All tests PASS → Coverage analysis → Sign-off
```

---

## 5. Orchestration (§1.3.5 / §19.4)

### Concept

A mechanism that automatically controls and coordinates workflows spanning multiple conductors (fpga / asic / pcb / debug).

### Why It Is Needed

In real-world hardware development, there are cross-domain flows such as "FPGA prototyping → ASIC conversion → test board (PCB) design → debugging." Rather than operating each conductor individually, defining and automatically executing workflows eliminates manual coordination effort and errors.

### How It Works

1. Define workflows as DAGs (Directed Acyclic Graphs) in YAML format
2. WorkflowEngine determines execution order via topological sort
3. Execute steps whose dependencies are satisfied (supports parallel execution)
4. Each step sends a structured message to the target conductor's agent-cli peer
5. Diamond dependencies (branch → merge) are supported

```yaml
# Workflow definition example
steps:
  - id: fpga_synth
    conductor: fpga
    method: build/start
    params: { target: artix7 }
  - id: pcb_design
    conductor: pcb
    method: build/start
    depends_on: [fpga_synth]
  - id: debug_setup
    conductor: debug
    method: connect
    depends_on: [fpga_synth, pcb_design]
```

### Execution Engine (WorkflowEngine)

- Topological sort using Kahn's algorithm
- Parses `depends_on` as a DAG, allows diamond dependencies (branch → merge)
- Executes up to `max_parallel` steps in parallel
- Each step's output (`${step.outputs.*}`) can be referenced in subsequent steps
- On failure: `abort` / `continue` / `rollback` (automatically generates reverse steps for rollback)

---

## 6. Harness-Driven Development (§1.3.6)

### Concept

A development methodology in which AI automatically generates test harnesses (the skeleton of the verification environment surrounding the DUT) and uses them as the foundation for verification.

### Why It Is Needed

Building SystemVerilog / UVM-based verification environments requires advanced expertise and can take weeks. Automatic test harness generation dramatically reduces verification environment setup time, allowing engineers to focus on designing test scenarios.

### How It Works

1. Analyze the DUT's port definitions and interface specifications
2. AI automatically generates the following harness components:
   - Clock and reset generation circuits
   - Bus interface drivers (AXI / AHB / Wishbone etc.)
   - Monitors (signal observation / protocol checking)
   - Scoreboards (expected value comparison)
   - Coverage collectors
3. Engineers add test scenarios and run verification

```
DUT port definitions → AI harness generation → Clock/reset + Driver + Monitor + Scoreboard
                                     ↓
                              Engineer adds test scenarios
                                     ↓
                              Verification execution → Coverage report
```

---

## 7. Sustainable Upgrade (§1.3.7)

### Concept

A maintenance methodology in which AI agents automatically generate, verify, and apply patches for vendor tool version upgrades, minimizing human intervention.

### Why It Is Needed

FPGA / ASIC / PCB vendor tools have 1-2 major releases per year, and each version upgrade requires modifications to TCL scripts, log parsers, and constraint formats. This maintenance cost accumulates over time. AI agent-driven automatic tracking enables the sustainable maintenance of the development environment.

### How It Works

Each agent is implemented as an agent-cli process:

1. **WatcherAgent**: Monitors vendor sites every 6 hours to detect new versions
2. **ProbeAgent**: Runs test builds with new versions using standard test project suites to detect incompatibilities
3. **PatcherAgent**: Leverages agent-cli's internal tools / Tool Use functionality (backends: Claude / Codex / Ollama / llama.cpp) to automatically generate patches
4. **ValidatorAgent**: Verifies patches in a sandbox environment and calculates confidence scores
5. **HumanReviewGate**: Determines automatic application or manual review based on confidence
6. **UpgradeManager**: Controls gradual rollout (Canary → Staging → Production) based on semantic versioning

Each agent operates as an independent agent-cli process, exchanging progress, patches, and verification results via the agent-cli shared registry (`$XDG_RUNTIME_DIR/agent-cli`) peer discovery + `/send <peer> <text>` IPC.

```
Detection → Testing → Patch generation → Verification → Judgment → Gradual application → Rollback on failure
```

---

## 8. Generative AI Tool Use (§1.3.8)

### Concept

An agent loop mechanism in which generative AI (LLM) calls external tools and functions to iteratively solve problems.

### Why It Is Needed

LLMs excel at understanding and generating natural language but cannot directly perform external operations such as file reading, command execution, or database searches. The Tool Use functionality allows LLMs to call external tools, add the results to their context, and continue reasoning, enabling them to solve real-world problems.

### How It Works

1. Present tool definitions (name, arguments, description) to the LLM in advance
2. The LLM analyzes the task and requests the necessary tool calls (`tool_use` response)
3. The execution environment runs the tool and returns the result to the LLM (`tool_result`)
4. The LLM determines the next action based on the result (iteration)
5. The loop continues until a final answer is obtained (with a maximum retry limit)

### Usage Example in Hestia (PatcherAgent)

```
LLM ← Tool definitions (6 types)
  │
  ├── read_adapter_manifest()  → Get adapter.toml contents
  ├── read_error_log()         → Get build error details
  ├── search_breaking_changes() → Search known breaking changes
  ├── read_vendor_changelog()  → Get release notes
  ├── propose_patch()          → Submit a patch proposal
  └── trigger_validation()     → Run verification
  │
  └── Final answer: patch proposal + fix reason + confidence score
```

Through this Tool Use loop, PatcherAgent can autonomously execute a series of workflows including reading actual files, searching known issues, and proposing and verifying patches — not merely generating text.

---

## 9. RAG (Retrieval-Augmented Generation) (§1.3.9 / §19.3)

### Concept

A technique that dramatically improves the accuracy and expertise of responses by storing proprietary data (datasheets, design guidelines, past design assets, error logs, etc.) in a vector database, and automatically searching and injecting relevant documents when querying the LLM.

### Why It Is Needed

LLMs possess general knowledge but may not include domain-specific information such as specific IC datasheet details, internal design guidelines, or past project design patterns. RAG dynamically injects this proprietary data into the LLM's context, preventing hallucinations (factually incorrect responses) while generating accurate, project-specific answers.

### How It Works

```
[Index Construction Phase (Offline)]

Document collection (datasheet PDFs / design guidelines / past HDL / build error logs / adapter.toml etc.)
    ↓
Document loaders (PDF/MD/TOML) → Chunk splitting (500-1000 tokens) → Embedding model (Ollama) → Vector DB (Chroma/Qdrant)

[Query Phase (Online)]

User query → Query embedding → Vector DB similarity search (top-k=5) → Relevant document retrieval → Inject into LLM (Ollama local) → Response generation
```

### Technology Stack

| Component | Technology | Role |
|--------------|------|------|
| LLM execution | Ollama | Local LLM execution (privacy protection) |
| Pipeline control | TypeScript + LangChain | RAG pipeline construction and control |
| Embedding model | Ollama Embedding (nomic-embed-text etc.) | Vectorization of documents and queries |
| Vector DB | Chroma or Qdrant | Vector storage and similarity search |
| Document loader | LangChain DocumentLoader | PDF / Markdown / TOML / source code loading |

### Concrete RAG Use Cases in Hestia

1. **Circuit design assistance**: "How to connect STM32F103 + BME280" → Automatically extract recommended circuits and pin connections from both IC datasheets, improving SKiDL code generation accuracy
2. **HDL design assistance**: "Implement AXI4 bus interface" → Search past HDL library AXI4 implementation patterns, reflecting them in skeleton code generation
3. **Error repair assistance**: "Vivado 2026.1 Synth 8-439 error" → Search past similar errors and fix patches, injecting them into PatcherAgent's context
4. **Constraint design assistance**: "Artix-7 clock constraint settings" → Search timing constraint configuration methods from Vivado UG903, reflecting them in XDC file generation
5. **ASIC flow assistance**: "Sky130 routing rules" → Search DRC rules from PDK documentation, optimizing OpenROAD configuration parameters

---

## Related Documentation

- [Spec-Driven Development](spec_driven_development.md) — SDD detailed specification, DesignSpec AST, operational rules
- [Architecture Overview](architecture_overview.md) — Position of AI utilization in the overall architecture
- [Security](security.md) — API key protection and intellectual property protection
- [Shared Services](shared_services.md) — RAG (rag-conductor) details, ingestion pipeline, self-learning
- [Container Execution](container_execution.md) — Container build CI/CD integration
- `.hestia/doc/ai/skills_system.md` — Skill system detailed specification
- `.hestia/doc/ai/workflow_engine.md` — Workflow engine detailed specification