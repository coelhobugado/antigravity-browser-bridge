# ADR 0005: Desktop Provider Boundary

## Status
Proposed

## Context
To offer capabilities akin to Claude Cowork and GPT-5.6 Work, the system needs a way to interface with desktop OS APIs.

## Decision
Create a new plugin capability `desktop.provider` that abstracts Windows UI Automation, macOS Accessibility, and Linux AT-SPI.
