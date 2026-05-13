# ai-conductor SkillSystem Details

**Target Conductor**: ai-conductor
**Source**: Design Specification §3.7 (around lines 1095-1108), §1.3.2 (around lines 95-114)

## Overview

SkillRegistry registers specialized skills that AI agents (launched as agent-cli processes, see §20) can invoke. Skills are combined with agent-cli persona files (YAML+Markdown) to define the capabilities of each conductor's main agent and sub-agents.

## SkillRegistry

A registry for skill registration, management, and resolution. Implemented in the `skill-system/` crate of ai-conductor.

```
skill-system/
└── src/
    ├── lib.rs      # SkillRegistry
    └── skill.rs    # Skill trait
```

## Skill Trait

A trait that all skills must implement.

```rust
pub trait Skill {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, input: &SkillInput) -> Result<SkillOutput, SkillError>;
}
```

Custom skills implement the `Skill` trait and register with SkillRegistry.

## Default Skills (5 types)

| Skill | Input | Output | Description |
|--------|------|------|------|
| **HDL Generation** | Specification / block diagram | SystemVerilog / Verilog / VHDL source code | AI-powered automatic HDL code generation |
| **Constraint Generation** | Target device / timing requirements | XDC / SDC / PCF constraint files | Automatic generation of device-specific constraints |
| **Testbench Generation** | HDL module definition | Testbench + assertions + coverage | Automatic generation of verification environment skeletons |
| **Schematic Design** | Natural language specification / KG | SKiDL code / KiCad netlist | AI-driven schematic synthesis (integrates with pcb-conductor) |
| **Review** | HDL code / schematics | Review results + modification suggestions | Design quality review and improvement proposals |

## Skill Invocation Flow

```
1. Register skill in SkillRegistry (implement Skill trait)
2. ai-conductor dynamically launches an Agent with the required skill
3. Agent executes the skill and reports the result to ai-conductor
```

## Skill and Conductor Mapping

| Conductor | Skills Used |
|-----------|-----------|
| ai-conductor | All skills (orchestration) |
| rtl-conductor | HDL Generation, Testbench Generation, Review |
| fpga-conductor | HDL Generation, Constraint Generation, Testbench Generation |
| asic-conductor | HDL Generation, Constraint Generation, Testbench Generation |
| pcb-conductor | Schematic Design, Review |
| hal-conductor | HDL Generation (SystemVerilog template) |
| apps-conductor | Review |

## Integration with agent-cli Personas

Skills are used in combination with agent-cli persona files. Each sub-agent's persona file (`.hestia/personas/<name>.md`) contains instructions for skill usage.

```yaml
# Persona file example (ai-planner.md)
skills:
  - hdl_generation
  - constraint_generation
  - testbench_generation
  - documentation_generation
```

## Extensibility

Adding a new skill only requires implementing the `Skill` trait and registering it with SkillRegistry. No changes to core code are needed (Principle 2: extension without modification).

## Related Documentation

- [ai/agent_hierarchy.md](agent_hierarchy.md) — Sub-agent hierarchy
- [ai/message_methods.md](message_methods.md) — ai.* method list
- [ai/workflow_engine.md](workflow_engine.md) — WorkflowEngine details
- [../spec_driven_development.md](../spec_driven_development.md) — Spec-driven development overview