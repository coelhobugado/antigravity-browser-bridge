# ADR 0002: Durable Event Store

## Status
Proposed

## Context
Long running tasks need to survive process restarts and maintain state to prevent repeating mechanical steps unnecessarily.

## Decision
Implement a durable event store (using SQLite WAL mode) to log task graphs, evidence, and states.
