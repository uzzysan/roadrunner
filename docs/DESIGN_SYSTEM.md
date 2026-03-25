# RoadRunner Design System

## Paleta Kolorów

### Light Theme
```css
--primary: #2563EB;           /* Niebieski - główny */
--primary-hover: #1D4ED8;     /* Ciemniejszy niebieski */
--primary-light: #DBEAFE;     /* Jasny niebieski */
--secondary: #64748B;         /* Szary - drugorzędny */
--background: #FFFFFF;        /* Białe tło */
--surface: #F8FAFC;           /* Jasne tło kart */
--border: #E2E8F0;            /* Granice */
--text-primary: #0F172A;      /* Główny tekst */
--text-secondary: #64748B;    /* Drugorzędny tekst */
--text-muted: #94A3B8;        /* Wyciszony tekst */
--success: #10B981;           /* Sukces */
--success-light: #D1FAE5;     /* Jasny sukces */
--warning: #F59E0B;           /* Ostrzeżenie */
--warning-light: #FEF3C7;     /* Jasne ostrzeżenie */
--error: #EF4444;             /* Błąd */
--error-light: #FEE2E2;       /* Jasny błąd */
--info: #3B82F6;              /* Informacja */
--info-light: #DBEAFE;        /* Jasna informacja */
```

### Dark Theme
```css
--primary: #3B82F6;           /* Niebieski - główny */
--primary-hover: #60A5FA;     /* Jaśniejszy niebieski */
--primary-light: #1E3A8A;     /* Ciemny niebieski */
--secondary: #94A3B8;         /* Szary - drugorzędny */
--background: #0F172A;        /* Ciemne tło */
--surface: #1E293B;           /* Tło kart */
--border: #334155;            /* Granice */
--text-primary: #F8FAFC;      /* Główny tekst */
--text-secondary: #94A3B8;    /* Drugorzędny tekst */
--text-muted: #64748B;        /* Wyciszony tekst */
--success: #34D399;           /* Sukces */
--success-light: #064E3B;     /* Ciemny sukces */
--warning: #FBBF24;           /* Ostrzeżenie */
--warning-light: #78350F;     /* Ciemne ostrzeżenie */
--error: #F87171;             /* Błąd */
--error-light: #7F1D1D;       /* Ciemny błąd */
--info: #60A5FA;              /* Informacja */
--info-light: #1E3A8A;        /* Ciemna informacja */
```

## Typografia

### Font
- **Rodzina**: Inter (Google Fonts)
- **URL**: `https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap`

### Skala
| Nazwa | Rozmiar | Waga | Użycie |
|-------|---------|------|--------|
| H1 | 32px (2rem) | 600 | Główne nagłówki |
| H2 | 24px (1.5rem) | 600 | Sekcje |
| H3 | 20px (1.25rem) | 600 | Podsekcje |
| H4 | 18px (1.125rem) | 600 | Karty |
| Body | 16px (1rem) | 400 | Główny tekst |
| Small | 14px (0.875rem) | 400 | Drugorzędny tekst |
| XSmall | 12px (0.75rem) | 400 | Etykiety |

### Line Height
- Nagłówki: 1.2
- Body: 1.5
- Small: 1.4

## Komponenty

### Button
```
Primary:
- Background: var(--primary)
- Text: white
- Padding: 12px 24px
- Border-radius: 8px
- Font-weight: 500
- Hover: var(--primary-hover)
- Transition: all 200ms

Secondary:
- Background: transparent
- Border: 1px solid var(--border)
- Text: var(--text-primary)
- Hover: var(--surface)

Danger:
- Background: var(--error)
- Text: white
- Hover: darker red

Ghost:
- Background: transparent
- Text: var(--primary)
- Hover: var(--primary-light)
```

### Input
```
- Background: var(--background)
- Border: 1px solid var(--border)
- Border-radius: 8px
- Padding: 12px 16px
- Font-size: 16px
- Focus: border-color var(--primary), ring 2px var(--primary-light)
- Error: border-color var(--error)
- Placeholder: var(--text-muted)
```

### Card
```
- Background: var(--surface)
- Border-radius: 12px
- Padding: 24px
- Shadow: 0 1px 3px rgba(0,0,0,0.1)
- Border: 1px solid var(--border) [opcjonalnie]
```

### Modal
```
- Overlay: rgba(0,0,0,0.5)
- Background: var(--background)
- Border-radius: 16px
- Padding: 32px
- Max-width: 500px
- Shadow: 0 25px 50px -12px rgba(0,0,0,0.25)
```

## Breakpoints

```
Mobile: < 640px
Tablet: 640px - 1024px
Desktop: > 1024px
```

## Spacing

```
4px  - xs
8px  - sm
16px - md
24px - lg
32px - xl
48px - 2xl
64px - 3xl
```

## Ikony

- **Biblioteka**: Lucide React
- **Rozmiary**: 16px, 20px, 24px, 32px
- **Stroke width**: 2px

## Dostępność (WCAG 2.1 AA)

- Kontrast tekstu: minimum 4.5:1
- Kontrast UI: minimum 3:1
- Focus indicators: widoczne
- ARIA labels: wszystkie interaktywne elementy
- Keyboard navigation: pełne wsparcie
