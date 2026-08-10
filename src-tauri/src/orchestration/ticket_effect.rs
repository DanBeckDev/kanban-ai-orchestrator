use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::domain::{TicketEffectAction, TicketEffectProposal};

use super::redacted_summary;

pub const MAX_TICKET_EFFECT_EVIDENCE: usize = 20;
pub const MAX_TICKET_EFFECT_INPUT_CHARS: usize = 2_000;
pub const MAX_TICKET_EFFECT_DESCRIPTION_CHARS: usize = 4_000;
pub const MAX_TICKET_EFFECT_CRITERIA: usize = 12;
pub const MAX_TICKET_EFFECT_CRITERION_CHARS: usize = 300;
pub const MAX_TICKET_EFFECT_TITLE_CHARS: usize = 160;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketEffectInput {
    pub action: TicketEffectAction,
    pub prompt: String,
    pub task: TicketEffectTask,
    pub evidence: Vec<TicketEffectEvidence>,
    output_contract: &'static str,
}

impl TicketEffectInput {
    pub fn new(
        action: TicketEffectAction,
        prompt: &str,
        task: TicketEffectTask,
        evidence: Vec<TicketEffectEvidence>,
    ) -> Result<Self, TicketEffectInputError> {
        if prompt.trim().is_empty() {
            return Err(TicketEffectInputError::MissingPrompt);
        }
        if evidence.len() > MAX_TICKET_EFFECT_EVIDENCE {
            return Err(TicketEffectInputError::TooMuchEvidence);
        }
        let prompt = bounded_redacted(prompt, MAX_TICKET_EFFECT_INPUT_CHARS);
        if prompt.is_empty() {
            return Err(TicketEffectInputError::MissingPrompt);
        }
        Ok(Self {
            action,
            prompt,
            task: task.redacted(),
            evidence: evidence
                .into_iter()
                .map(TicketEffectEvidence::redacted)
                .collect(),
            output_contract: "Return exactly one JSON object with action, recommendation, rationale, and proposal. action must exactly match the requested action. recommendation and rationale must be short plain-language summaries. proposal may contain only a full refinement for refine_specification, worker guidance for give_worker_guidance, or an evidence explanation for explain_evidence. Do not use Markdown, logs, credentials, commands, or fields outside this contract.",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketEffectTask {
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub state: String,
}

impl TicketEffectTask {
    fn redacted(self) -> Self {
        Self {
            title: bounded_redacted(&self.title, MAX_TICKET_EFFECT_TITLE_CHARS),
            description: bounded_redacted(&self.description, MAX_TICKET_EFFECT_DESCRIPTION_CHARS),
            acceptance_criteria: self
                .acceptance_criteria
                .into_iter()
                .take(MAX_TICKET_EFFECT_CRITERIA)
                .map(|criterion| bounded_redacted(&criterion, MAX_TICKET_EFFECT_CRITERION_CHARS))
                .collect(),
            state: bounded_redacted(&self.state, 40),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketEffectEvidence {
    pub kind: String,
    pub result: String,
    pub summary: String,
}

impl TicketEffectEvidence {
    fn redacted(self) -> Self {
        Self {
            kind: bounded_redacted(&self.kind, 80),
            result: bounded_redacted(&self.result, 80),
            summary: bounded_redacted(&self.summary, 600),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TicketEffectRecommendation {
    pub action: TicketEffectAction,
    pub recommendation: String,
    pub rationale: String,
    #[serde(default)]
    pub proposal: TicketEffectProposal,
}

impl TicketEffectRecommendation {
    pub fn validate_against(
        &mut self,
        input: &TicketEffectInput,
    ) -> Result<(), TicketEffectRecommendationError> {
        if self.action != input.action {
            return Err(TicketEffectRecommendationError::UnexpectedAction);
        }
        self.recommendation = redacted_summary(&self.recommendation);
        self.rationale = redacted_summary(&self.rationale);
        if self.recommendation.is_empty() || self.rationale.is_empty() {
            return Err(TicketEffectRecommendationError::MissingSummary);
        }
        self.proposal = normalized_proposal(self.action, std::mem::take(&mut self.proposal))?;
        Ok(())
    }
}

fn normalized_proposal(
    action: TicketEffectAction,
    proposal: TicketEffectProposal,
) -> Result<TicketEffectProposal, TicketEffectRecommendationError> {
    match action {
        TicketEffectAction::RefineSpecification => refinement(proposal),
        TicketEffectAction::GiveWorkerGuidance => guidance(proposal),
        TicketEffectAction::ExplainEvidence => explanation(proposal),
        TicketEffectAction::PrepareStart
        | TicketEffectAction::PrepareRestart
        | TicketEffectAction::ReturnForCorrection
        | TicketEffectAction::RecoverInterrupted => empty_proposal(proposal),
    }
}

fn refinement(
    proposal: TicketEffectProposal,
) -> Result<TicketEffectProposal, TicketEffectRecommendationError> {
    let title = bounded_redacted(
        proposal
            .title
            .as_deref()
            .ok_or(TicketEffectRecommendationError::IncompleteRefinement)?,
        MAX_TICKET_EFFECT_TITLE_CHARS,
    );
    let description = bounded_redacted(
        proposal
            .description
            .as_deref()
            .ok_or(TicketEffectRecommendationError::IncompleteRefinement)?,
        MAX_TICKET_EFFECT_DESCRIPTION_CHARS,
    );
    if title.is_empty()
        || description.is_empty()
        || proposal.acceptance_criteria.is_empty()
        || proposal.acceptance_criteria.len() > MAX_TICKET_EFFECT_CRITERIA
        || proposal.worker_guidance.is_some()
        || proposal.evidence_explanation.is_some()
    {
        return Err(TicketEffectRecommendationError::IncompleteRefinement);
    }
    let acceptance_criteria = proposal
        .acceptance_criteria
        .iter()
        .map(|criterion| bounded_redacted(criterion, MAX_TICKET_EFFECT_CRITERION_CHARS))
        .collect::<Vec<_>>();
    if acceptance_criteria.iter().any(String::is_empty) {
        return Err(TicketEffectRecommendationError::IncompleteRefinement);
    }
    Ok(TicketEffectProposal {
        title: Some(title),
        description: Some(description),
        acceptance_criteria,
        ..TicketEffectProposal::default()
    })
}

fn guidance(
    proposal: TicketEffectProposal,
) -> Result<TicketEffectProposal, TicketEffectRecommendationError> {
    let guidance = bounded_redacted(
        proposal
            .worker_guidance
            .as_deref()
            .ok_or(TicketEffectRecommendationError::MissingGuidance)?,
        MAX_TICKET_EFFECT_DESCRIPTION_CHARS,
    );
    if guidance.is_empty()
        || proposal.title.is_some()
        || proposal.description.is_some()
        || !proposal.acceptance_criteria.is_empty()
        || proposal.evidence_explanation.is_some()
    {
        return Err(TicketEffectRecommendationError::MissingGuidance);
    }
    Ok(TicketEffectProposal {
        worker_guidance: Some(guidance),
        ..TicketEffectProposal::default()
    })
}

fn explanation(
    proposal: TicketEffectProposal,
) -> Result<TicketEffectProposal, TicketEffectRecommendationError> {
    let explanation = bounded_redacted(
        proposal
            .evidence_explanation
            .as_deref()
            .ok_or(TicketEffectRecommendationError::MissingExplanation)?,
        MAX_TICKET_EFFECT_DESCRIPTION_CHARS,
    );
    if explanation.is_empty()
        || proposal.title.is_some()
        || proposal.description.is_some()
        || !proposal.acceptance_criteria.is_empty()
        || proposal.worker_guidance.is_some()
    {
        return Err(TicketEffectRecommendationError::MissingExplanation);
    }
    Ok(TicketEffectProposal {
        evidence_explanation: Some(explanation),
        ..TicketEffectProposal::default()
    })
}

fn empty_proposal(
    proposal: TicketEffectProposal,
) -> Result<TicketEffectProposal, TicketEffectRecommendationError> {
    if proposal == TicketEffectProposal::default() {
        Ok(proposal)
    } else {
        Err(TicketEffectRecommendationError::UnexpectedProposal)
    }
}

pub fn bounded_redacted(value: &str, maximum: usize) -> String {
    redacted_summary(value).chars().take(maximum).collect()
}

#[derive(Debug, Eq, PartialEq)]
pub enum TicketEffectInputError {
    MissingPrompt,
    TooMuchEvidence,
}

impl fmt::Display for TicketEffectInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPrompt => formatter.write_str("ticket AI needs a prompt"),
            Self::TooMuchEvidence => {
                formatter.write_str("ticket AI can receive at most 20 evidence items")
            }
        }
    }
}

impl Error for TicketEffectInputError {}

#[derive(Debug, Eq, PartialEq)]
pub enum TicketEffectRecommendationError {
    UnexpectedAction,
    MissingSummary,
    IncompleteRefinement,
    MissingGuidance,
    MissingExplanation,
    UnexpectedProposal,
}

impl fmt::Display for TicketEffectRecommendationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnexpectedAction => "ticket AI selected an action it was not asked to prepare",
            Self::MissingSummary => "ticket AI needs concise recommendation and rationale",
            Self::IncompleteRefinement => "ticket AI refinement needs complete task details",
            Self::MissingGuidance => "ticket AI worker guidance is missing",
            Self::MissingExplanation => "ticket AI evidence explanation is missing",
            Self::UnexpectedProposal => {
                "ticket AI returned details that are not safe for this action"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for TicketEffectRecommendationError {}
