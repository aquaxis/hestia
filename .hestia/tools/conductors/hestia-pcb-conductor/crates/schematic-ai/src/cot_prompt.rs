//! Chain-of-thought prompt construction for schematic generation

use crate::SchematicRequest;

/// Build a chain-of-thought prompt for schematic generation.
pub fn build_cot_prompt(request: &SchematicRequest) -> String {
    format!(
        r#"You are an expert electronics engineer designing a PCB schematic.

## Requirements
{description}

## Constraints
{constraints}

## Step-by-step reasoning
1. Identify the core functional blocks needed.
2. Select appropriate ICs and passive components.
3. Determine the interconnections and signal flow.
4. Add decoupling, protection, and termination as needed.
5. Validate against electrical rules.

Please generate a complete KiCad schematic in S-expression format."#,
        description = request.description,
        constraints = request.constraints.join("\n"),
    )
}

/// Build a refinement prompt that asks the AI to improve a previous schematic.
pub fn build_refinement_prompt(
    original: &str,
    feedback: &str,
) -> String {
    format!(
        r#"The following schematic was generated but needs improvement.

## Original Schematic
{original}

## Feedback
{feedback}

Please improve the schematic addressing the feedback above."#,
        original = original,
        feedback = feedback,
    )
}