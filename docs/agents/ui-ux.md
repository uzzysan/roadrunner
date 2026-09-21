# UI/UX contract for all RoadRunner agents

Applies to every UI change in React Native/Expo, Flutter, Leptos, web and desktop. Product direction and design ownership: GPT-6. Agents implementing application code: **gpt-5.6-sol**, as explicitly requested by the user. The coordinator assigns non-overlapping files and reviews integration.

Before editing UI, read [`../DESIGN_SYSTEM_V2.md`](../DESIGN_SYSTEM_V2.md) and the relevant component/domain sections. Numeric source of truth: [`../../design/tokens.json`](../../design/tokens.json). Brand source: [`../../assets/brand/`](../../assets/brand/). Historical `DESIGN_SYSTEM.md` and orange/blue screen literals are not design authority.

1. Use semantic tokens through a shared theme adapter. Never add a screen-local palette or a competing theme provider. Carry the active theme into navigation, dialogs and embedded map HTML.
2. Reuse component contracts. New UI must include light/dark, PL/EN, meaningful names/roles/states, 48 dp touch targets (56 for safety), keyboard focus, reduced motion and scalable text.
3. Use actual data and explicit loading/empty/error/offline/stale/permission states. An unavailable feature must not expose a dead action or a fake success.
4. Keep transport and safety meanings distinct. A selected stop is not red; GPS is not evidence that a child has been handed over; acknowledging an alert is not resolving it. QR stays black on white with its quiet zone.
5. Follow the full Definition of Done in section 10. Report evidence and unperformed checks separately; do not claim accessibility compliance from token contrast alone.
6. A necessary new design pattern belongs in the system and shared component before it spreads into screens. Update token version, mappings and contrast checks together when values change. State deliberate exceptions and their scope.

Workflow required by the user: design guidance → audit current application against it → implement and verify the migration → continue product backlog using the new guidance. Do not replace the current application stack as a side effect of visual work. Respect the repository's code/document exploration instructions and refresh indexes after edits.
