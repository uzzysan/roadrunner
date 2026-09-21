# RoadRunner design language

This file is the stable, tracked entry point referenced by Linear and architecture
documentation. It supersedes historical client-specific palettes and screenshots.

## Product language

- Use one semantic design-token contract across Expo, Leptos and Flutter; do not copy raw
  visual values into individual clients.
- Support light and dark themes as separately validated palettes rather than mechanical color
  inversion.
- Use Inter where available and retain full Polish-diacritic coverage.
- Prefer calm transit information hierarchy; reserve urgent styling for safety-critical state.
- Approved brand assets and machine-readable tokens must be tracked in this repository before
  they are treated as release dependencies.

## Accessibility and resilience

Every user-facing change must satisfy WCAG 2.2 AA, keyboard use, visible focus, reduced motion,
200% zoom, and status communication through text/icon plus color. PL and EN content ship
together. Loading, empty, error, stale and offline states are part of the feature, not a later
polish pass.

Touch targets must be at least 44 by 44 CSS pixels or the native-platform equivalent. Safety
and validation feedback must remain understandable without sound, vibration or color.

## Client lifecycle

React Native/Expo remains the deployed and UX-reference client. Flutter is the target mobile
client and Leptos is the target administration client. New token or component specifications
must be introduced through this entry point (or a document linked from it) and reviewed for all
active clients.
