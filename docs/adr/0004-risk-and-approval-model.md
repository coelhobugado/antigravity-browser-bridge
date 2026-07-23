# ADR 0004: Risk and Approval Model

## Status
Proposed

## Context
Executing high-risk tasks (e.g. submitting payments, sending emails) without checks can lead to irreversible negative outcomes.

## Decision
Adopt a two-phase commit strategy using approval receipts for actions categorized as high impact.
