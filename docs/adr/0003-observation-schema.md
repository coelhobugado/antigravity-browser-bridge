# ADR 0003: Observation Schema

## Status
Proposed

## Context
Need to define how page state, DOM, and trust metrics are encapsulated and passed to the model.

## Decision
Create an `ObservationPacket` that aggregates structured state, visual elements, temporal state, and confidence metrics.
