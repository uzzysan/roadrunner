# RoadRunner brand v2

Canonical, original vector identity: **Connected Route**. See [the complete design system](../../docs/DESIGN_SYSTEM_V2.md) for usage, clear space, minimum size and accessibility.

Open [design-preview.svg](design-preview.svg) for the visual direction: passenger light, guardian dark and operations desktop. The board is an illustrative design artifact; its example journeys and states are not live application data. The title text on the board is a specimen, while shipped product logos must use the vector lockups below.

| File | Use |
| --- | --- |
| `roadrunner-lockup-light.svg` | Full identity on light surfaces |
| `roadrunner-lockup-dark.svg` | Full identity on dark surfaces |
| `roadrunner-lockup-mono.svg` | Single color; inline `currentColor`, black by default |
| `roadrunner-mark-light.svg` | Compact symbol on light surfaces |
| `roadrunner-mark-dark.svg` | Compact symbol on dark surfaces |
| `roadrunner-mark-mono.svg` | Single-color compact symbol |
| `roadrunner-app-icon.svg` | 1024-square master for launcher exports |

Production assets use only vector paths/groups, no embedded font or external resource. Keep accessible names when the image communicates identity; hide redundant decorative images from assistive technology. Repeated inline assets must receive instance-unique title/description IDs from their wrapper; external images use appropriate alt text.

Do not recolor a light asset for dark mode at runtime, stretch the symbol, rebuild the lettering with a font, or place the mark inside a QR code. Use the matching variant. An externally loaded monochrome SVG does not inherit host text color; use it inline or explicitly export the intended single color.

Validation on 2026-09-18: all eight SVG files parse as well-formed XML, include `viewBox`, `title` and `desc`, and contain no script, event attributes or external references. UI tokens passed all 104 declared contrast checks. Token contrast is not whole-application accessibility certification.
