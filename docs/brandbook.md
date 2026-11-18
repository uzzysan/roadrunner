# RoadRunner · Connected Route · Księga marki / Brandbook

![RoadRunner Connected Route](../assets/brand/roadrunner-lockup-light.svg)

**PL** Jeden język wizualny obejmuje mobilny klient referencyjny Expo, docelowy klient mobilny Flutter i panel administracyjny Leptos. Ten przewodnik pomaga stosować [specyfikację UI](DESIGN_SYSTEM_V2.md) i [wspólne tokeny](../design/tokens.json).

**EN** The Expo reference client, target Flutter mobile client and target Leptos administration client share one visual language. Use this guide with the [UI specification](DESIGN_SYSTEM_V2.md) and [shared tokens](../design/tokens.json).

## Znak / Identity

**PL** Linia trasy tworzy literę R. W tekście pisz „RoadRunner”. Logotyp jest rysunkiem wektorowym. Używaj gotowego pliku na jednolitym tle i zachowaj pole ochronne 8 jednostek wokół siatki 64 × 64. Sam znak ma co najmniej 24 × 24 px/dp (zalecane 32), a pełny logotyp co najmniej 136 px szerokości (zalecane 176). Przy 16 px użyj eksportu favicon.

**EN** A route line forms the R. Write “RoadRunner” in prose. The lettering is vector artwork. Place the supplied asset on a solid surface and leave 8 units clear around its 64 × 64 grid. Minimum mark size is 24 × 24 px/dp (32 recommended). Minimum lockup width is 136 px (176 recommended). Use the favicon export at 16 px.

| Zastosowanie / Placement | Zasób / Artwork |
|---|---|
| Logowanie / Sign-in | [Jasne tło / Light surface](../assets/brand/roadrunner-lockup-light.svg) · [Ciemne tło / Dark surface](../assets/brand/roadrunner-lockup-dark.svg) |
| Wąski nagłówek / Compact header | [Jasny / Light](../assets/brand/roadrunner-mark-light.svg) · [Ciemny / Dark](../assets/brand/roadrunner-mark-dark.svg) |
| Druk jednobarwny / Single-ink print | [Mono](../assets/brand/roadrunner-lockup-mono.svg) |
| Ikona aplikacji / Launcher | [Master](../assets/brand/roadrunner-app-icon.svg) |

## Kolor i typografia / Colour and type

| Token | Jasny / Light | Ciemny / Dark | Funkcja / Role |
|---|---|---|---|
| `background` | `#F5F7F7` | `#0C1D25` | Tło / Canvas |
| `surface` | `#FFFFFF` | `#132E38` | Powierzchnia / Surface |
| `text` | `#102C36` | `#F0F7F7` | Tekst / Text |
| `primary` | `#006B63` | `#5DE0CB` | Główna akcja / Primary action |
| `onPrimary` | `#FFFFFF` | `#073B35` | Etykieta akcji / Action label |
| `accent` | `#CDEB78` | `#CDEB78` | Akcent / Accent |
| `success` | `#166534` | `#86EFAC` | Potwierdzenie / Confirmation |
| `warning` | `#854D0E` | `#FCD34D` | Ostrzeżenie / Warning |
| `danger` | `#B42318` | `#FFB4A7` | Zagrożenie / Danger |

**PL** Przycisk łączy primary z onPrimary, a status opóźnienia łączy warningContainer z onWarningContainer, ikoną i tekstem „Opóźnienie 6 min”. Limonka akcentuje markę. Do opisów stosuj Inter 16/24 i zachowuj pełne nazwy przystanków, np. „Łódź Fabryczna”. W tabelach wyrównuj cyfry godzin. Przykładowy układ przedstawia [plansza Connected Route](../assets/brand/design-preview.svg).

**EN** Pair primary with onPrimary for a button. For a delay, pair warningContainer with onWarningContainer, an icon and the label “6 min late”. Lime provides a sparing brand accent. Set body copy in Inter at 16/24 and preserve full stop names such as “Łódź Fabryczna”. Align time digits in timetables. See the [Connected Route preview](../assets/brand/design-preview.svg).

## Ekrany robocze / Working screens

| Widok / View | Hierarchia i przykład / Hierarchy and example |
|---|---|
| Pasażer / Passenger | Linia i kierunek → przystanek → godzina. Odróżnij „Plan 07:40” i „Prognoza 07:46”. / Show line and destination, then stop and time. Distinguish “Scheduled 07:40” from “Expected 07:46”. |
| Kierowca / Driver | Na postoju pokaż następny przystanek, kurs i jedną główną czynność. Przy braku sieci podaj czas ostatniej aktualizacji. / At a stop, show the next stop, trip and one primary action. When offline, show the last update time. |
| Opiekun / Chaperone | Grupuj listę według przystanku. Po potwierdzeniu wejścia pokaż przy nazwisku „W pojeździe”. / Group the roster by stop. After boarding confirmation, show “On board” beside the rider. |
| Dyspozytor / Dispatcher | W tabeli wyrównaj godziny, zachowaj nagłówki kolumn, a wybrany kurs wyróżnij również na mapie. / Align times under persistent column headings. Reflect the selected trip on the map. |

**PL** Buduj ekran wokół kursu, przystanku lub biletu. Status ma wyglądać jak opis. Przyciski opisuj czasownikiem, np. „Pokaż kurs”. Badania GOV.UK wykazały, że użytkownicy brali kontrastowe etykiety statusu za przyciski. [GOV.UK Design System — Tag](https://design-system.service.gov.uk/components/tag/)

**EN** Organise each screen around a trip, stop or ticket. Style status as descriptive text. Label an action with a verb, such as “Show trip”. GOV.UK research found that people mistook high-contrast status tags for buttons. [GOV.UK Design System — Tag](https://design-system.service.gov.uk/components/tag/)


## Odbiór projektu / Design acceptance

**PL** Sprawdź oba motywy przy powiększeniu 200%, długiej nazwie przystanku i wyłączonych animacjach. Przejdź klawiaturą do szczegółów kursu i z powrotem. Sprawdź widoczność fokusu oraz stany ładowania, braku wyników, błędu, nieaktualnych danych i braku sieci. Token touch określa cel 48 × 48, a touchSafety 56 × 56. Minimalny rozmiar projektu to 44 × 44 piksele CSS lub odpowiednik natywny. [Zasady projektu](design-language.md) · [WCAG 2.2](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html)

**EN** Review both themes at 200% zoom, with a long stop name and reduced motion enabled. Navigate to trip details and back by keyboard. Check visible focus and the loading, empty, error, stale and offline states. The touch token specifies a 48 × 48 target and touchSafety specifies 56 × 56. The project minimum is 44 × 44 CSS pixels or the native equivalent. [Project requirements](design-language.md) · [WCAG 2.2](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html)
