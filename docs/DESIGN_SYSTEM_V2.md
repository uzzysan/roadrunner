# RoadRunner Design System v2 — Connected Route

Status: obowiązująca specyfikacja UI/UX od 2026-09-18. Autor kierunku: agent GPT-6, lead UI/UX. Wersja tokenów: `2.0.0`.

Dokument obejmuje cały system: aplikację React Native/Expo, przyszłą aplikację Flutter, panel Leptos i każdą powierzchnię web/desktop. Zastępuje decyzje wizualne z `DESIGN_SYSTEM.md`, sekcję Logo w `FRONTEND.md` i wcześniejszą paletę pomarańczową. Nie zmienia zakresu funkcji ani uprawnień. Obecność wzorca w tej specyfikacji nie oznacza, że funkcja została już zaimplementowana.

Źródło prawdy dla wartości: [`../design/tokens.json`](../design/tokens.json). Źródło prawdy dla zachowania i komponowania interfejsu: ten dokument. Zmiana tokenu wymaga równoczesnej zmiany dokumentacji, adapterów oraz weryfikacji kontrastu. Nie wolno tworzyć konkurencyjnej palety w ekranie. Reguły pracy agentów: [`agents/ui-ux.md`](agents/ui-ux.md).

Plansza poglądowa: [`Connected Route — preview`](../assets/brand/design-preview.svg). Przykładowe dane na planszy służą wyłącznie prezentacji kierunku.

## 1. Kierunek i hierarchia

RoadRunner pomaga pewnie odbyć podróż i skoordynować transport. Estetyka: spokojna, precyzyjna, ludzka. Atramentowy granat buduje czytelność, petrol oznacza podstawową interakcję, mięta rozjaśnia tryb ciemny, a jasna limonka jest oszczędnym akcentem marki. Informacja o bezpieczeństwie ma własną, konsekwentną semantykę.

Każdy ekran odpowiada kolejno: „Gdzie jestem?”, „Co wiem o podróży?”, „Co mogę teraz zrobić?”. Jedna dominująca akcja na sekcję. Najpierw czas, kierunek i stan; dopiero potem metadane. Nie wypełniać ekranu ozdobnymi kartami. Duże liczby służą odjazdom i pozostałemu czasowi, a nie marketingowym statystykom bez danych.

Stałe reguły:

- Czytelne etykiety obok ikon; żadnych emoji jako funkcjonalnych ikon lub statusów.
- Kolor nigdy nie jest jedynym nośnikiem stanu. Każdy status ma nazwę oraz ikonę/kształt.
- Prawdziwe dane i uczciwe stany. Brak danych nie oznacza zera; awaria nie oznacza pustej listy; utrata GPS nie oznacza bezpiecznego zakończenia przejazdu.
- Stan oczekiwania nie udaje sukcesu. Zakup, walidacja biletu i odebranie dziecka wymagają właściwego potwierdzenia domenowego.
- Preferencja motywu: System / Jasny / Ciemny, zapamiętywana lokalnie. Pierwsze uruchomienie respektuje system.
- Nowe ekrany budujemy ze wspólnych komponentów; wyjątek musi być opisany wraz z powodem i zakresem.

## 2. Tożsamość i logo

Autorski znak „Connected Route” to litera **R** złożona z zaokrąglonej linii trasy. Pion jest początkiem przejazdu, łuk łączy przystanki, a ukośny odcinek pokazuje dalszą drogę. Znak jest geometryczny; nie przedstawia ptaka, maskotki ani cudzego bohatera. Wordmark „RoadRunner” jest narysowany wektorowo, bez zależności od zewnętrznego fontu. Pisownia produktu: `RoadRunner`.

Zasoby w [`../assets/brand/`](../assets/brand/):

| Zasób | Przeznaczenie |
| --- | --- |
| `roadrunner-lockup-light.svg` | Znak petrol + granatowy napis na jasnej powierzchni |
| `roadrunner-lockup-dark.svg` | Znak mint + jasny napis na ciemnej powierzchni |
| `roadrunner-lockup-mono.svg` | Druk jednokolorowy lub inline SVG; kolor przez `currentColor` |
| `roadrunner-mark-light.svg`, `roadrunner-mark-dark.svg`, `roadrunner-mark-mono.svg` | Sam znak, np. kompaktowy nagłówek |
| `roadrunner-app-icon.svg` | Master 1024 × 1024: pełne granatowe tło i centralny znak mint |

Pole ochronne = 8 jednostek siatki znaku 64 × 64, liczone na zewnątrz jego viewBox. Lockup zachowuje tę samą wysokość siatki. Minimalny rozmiar: znak 24 × 24 px/dp, lockup 136 px szerokości; zalecane odpowiednio 32 i 176. Przy 16 px użyć uproszczonego eksportu favicon sprawdzonego wizualnie, nie pełnego wordmarku. Nie zmieniać proporcji, grubości linii, odstępu znaku od napisu ani układu liter. Nie stosować gradientu, cienia, obrysu, animowanego rysowania ani znaku jako patternu pod treścią. Na zdjęciu lub mapie logo musi leżeć na pełnej, jednolitej powierzchni.

App icon ma pełny kwadrat; zaokrąglenie narzuca system operacyjny. Eksporty rastrowe generować z mastera, nie skalować starszych PNG. Dla Android adaptive icon wyodrębnić ten sam centralny znak jako foreground, pozostawić tło `#102C36` i sprawdzić maski. Nie umieszczać wordmarku w ikonie. SVG zawierają `title` i `desc`; przy widocznym napisie obok znaku ukryć dekoracyjny znak przed czytnikiem, aby uniknąć podwójnego odczytu. Interaktywny lockup ma nazwę akcji, np. „RoadRunner — strona główna”.

## 3. Semantyczne kolory

Nazwy są kontraktem między platformami. W kodzie ekranów używać semantycznych nazw, nie HEX, nazw pigmentu ani indeksu koloru.

| Token | Light | Dark | Zastosowanie |
| --- | --- | --- | --- |
| `background` | `#F5F7F7` | `#0C1D25` | Tło ekranu |
| `surface` | `#FFFFFF` | `#132E38` | Karty, pola, nawigacja |
| `surfaceRaised` | `#FFFFFF` | `#1A3944` | Dialogi, popovery |
| `surfaceMuted` | `#EAF0EF` | `#1D3D47` | Drugorzędne powierzchnie |
| `text` | `#102C36` | `#F0F7F7` | Główna treść |
| `textSecondary` | `#48616A` | `#C0D2D7` | Metadane i opis |
| `textMuted` | `#536B73` | `#A6BDC5` | Pomoc, placeholder; nadal czytelny |
| `border` | `#CBD6D8` | `#345561` | Wyłącznie dekoracyjne separatory |
| `borderStrong` | `#71868D` | `#78939C` | Granice pól i rozpoznawalnych kontrolek |
| `primary` | `#006B63` | `#5DE0CB` | CTA, linki, wybrana nawigacja |
| `primaryHover` | `#005C55` | `#85EBD9` | Hover CTA |
| `primaryPressed` | `#004D47` | `#3ECEB7` | Wciśnięte CTA |
| `onPrimary` | `#FFFFFF` | `#073B35` | Tekst/ikona na trzech stanach CTA |
| `primaryContainer` | `#D7F5ED` | `#104A44` | Wybrana karta/chip |
| `onPrimaryContainer` | `#005249` | `#A5F3E4` | Tekst na wybranej powierzchni |
| `accent` | `#CDEB78` | `#CDEB78` | Mały akcent marki, nigdy ostrzeżenie |
| `onAccent` | `#24380B` | `#24380B` | Tekst na akcencie |
| `focus` | `#006B63` | `#5DE0CB` | Obramowanie fokusu |
| `success` | `#166534` | `#86EFAC` | Potwierdzony pozytywny status |
| `successContainer` | `#E8F5EC` | `#123C2A` | Tło pozytywnego statusu |
| `onSuccessContainer` | `#14532D` | `#BBF7D0` | Treść pozytywnego statusu |
| `warning` | `#854D0E` | `#FCD34D` | Sytuacja wymagająca uwagi |
| `warningContainer` | `#FFF2CC` | `#43320F` | Tło ostrzeżenia |
| `onWarningContainer` | `#713F12` | `#FDE68A` | Treść ostrzeżenia |
| `danger` | `#B42318` | `#FFB4A7` | Błąd, destrukcja, krytyczny alert |
| `onDanger` | `#FFFFFF` | `#60160F` | Treść wypełnionego danger CTA |
| `dangerContainer` | `#FEEBE7` | `#4A2423` | Tło błędu/alertu |
| `onDangerContainer` | `#912018` | `#FFDAD3` | Treść błędu/alertu |
| `info` | `#075985` | `#9DD8FF` | Neutralna informacja operacyjna |
| `infoContainer` | `#E5F2FC` | `#163E57` | Tło informacji |
| `onInfoContainer` | `#0C4A6E` | `#C6E8FF` | Treść informacji |
| `disabled` | `#DDE5E5` | `#294650` | Nieaktywna kontrolka |
| `onDisabled` | `#526870` | `#A6BDC5` | Czytelna treść nieaktywnej kontrolki |
| `inverse` | `#102C36` | `#E5F0F1` | Kontrastowa powierzchnia pomocnicza |
| `onInverse` | `#FFFFFF` | `#102C36` | Treść na `inverse` |

`border` nie ma gwarancji 3:1 i nie może sam identyfikować pola, przycisku, checkboxa ani fokusu. `accent` nie zastępuje `success` i nie jest kolorem tekstu na białym tle. Kolor `danger` w dark jest jasny; treść na nim zawsze `onDanger`, nie biała. Na wypełnionych statusach używać par `*Container` + `on*Container`. Nie nakładać przezroczystości na tekst ani całe przyciski w disabled.

Kontrast policzono dla 52 zadeklarowanych par w każdym motywie: tekstowe minimum light **4,59:1**, dark **5,13:1**; kontrolki/focus minimum light **3,31:1**, dark **3,56:1**. Dokładne pary i progi znajdują się w `contrastPairs` JSON. Pomiar używa względnej luminancji sRGB bez zaokrąglania przed porównaniem. To walidacja tokenów, a nie certyfikat zgodności całej aplikacji: kompozycję, grafikę mapy, font scaling i zachowanie trzeba sprawdzać na ekranach.

Wymagamy co najmniej 4,5:1 dla całej treści tekstowej; dla potrzebnych do obsługi kształtów i granic 3:1. Jest to zgodne z kierunkiem [WCAG 2.2, kontrast tekstu](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) oraz [kontrast elementów nietekstowych](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html). Nie stosować wyjątku dla dużego tekstu jako sposobu naprawy złej palety.

## 4. Typografia, geometria i ruch

Domyślnie font systemowy: iOS system/SF, Android system/Roboto, web `ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`. Jest to świadoma decyzja o szybkości, jakości renderowania PL i zgodności ze skalowaniem systemu. Nie pobierać fontu z sieci podczas uruchomienia. Inter z v1 nie jest wymagany. Wordmark ma własne krzywe SVG, niezależne od fontu interfejsu.

| Rola | Rozmiar / interlinia | Waga | Zastosowanie |
| --- | --- | --- | --- |
| `display` | 32 / 40 | 700 | Powitanie lub hero, najwyżej raz |
| `h1` | 28 / 36 | 700 | Tytuł ekranu |
| `h2` | 22 / 30 | 600 | Sekcja |
| `h3` | 18 / 26 | 600 | Karta lub dialog |
| `body` | 16 / 24 | 400 | Zwykła treść, pola |
| `bodyStrong` | 16 / 24 | 600 | Główne etykiety i akcje |
| `label` | 14 / 20 | 600 | Etykieta i status |
| `caption` | 14 / 20 | 400 | Metadane, podpowiedzi |
| `metric` | 36 / 44 | 700 | Najbliższy odjazd lub licznik |

Jednostki: px przy bazie web 16 px, dp i skalowalne sp/rozmiary tekstu na native. Web zapisuje typografię w rem. Nie wyłączać skalowania tekstu. Przy 200% powiększenia akcje zawijają się i sekcje rosną; nie ucinać nazw przystanków, tekstu alertu, kwoty ani czasu ważności. Zegary i liczby mają cyfry tabelaryczne tam, gdzie platforma je wspiera. Unikać CAPS w zdaniach, rozstrzelonych etykiet i nadmiaru bold.

Spacing: `none=0`, `xs=4`, `sm=8`, `md=12`, `lg=16`, `xl=24`, `xxl=32`, `section=48`, `hero=64`. Typowy ekran: padding 16 na małym telefonie, 24 od 600, 32 na desktop; przerwa sekcji 24/32; wnętrze karty 16/24. Radius: `sm=8` kontrolki/etykiety, `md=12` przyciski/pola, `lg=16` karty, `xl=24` sheet/dialog, `pill=999` tylko badge/chip. Nie nadawać wszystkim elementom formy kapsuły.

Elevation: poziom 0 bez cienia, poziom 1 (karta) opcjonalny cień `0 2px 8px rgba(16,44,54,.06)`, poziom 2 (popover/sheet) `0 8px 24px rgba(16,44,54,.14)`. Native odpowiednio 0/1/4 elevation. W dark używać `surfaceRaised` i obramowania; unikać jasnego halo. Modal overlay: czerń 48%, ale treść dialogu pozostaje pełna i kryjąca. Cienie nie są jedyną granicą interaktywnej kontrolki.

Ruch: instant 0 ms, fast 120 ms (pressed/hover), standard 180 ms (przejście), slow 240 ms (sheet). Tylko opacity i transform, krzywa ease-out. Preferencja reduced-motion wyłącza przesunięcia, pulsowanie i shimmer; stan zmienia się natychmiast lub przez krótkie zanikanie bez ruchu. Nie animować stale markerów pojazdów ani krytycznych alarmów. Aktualizacje GPS nie przewijają i nie odbierają fokusu.

## 5. Siatka i nawigacja

| Szerokość | Kompozycja |
| --- | --- |
| 320–599 | Jedna kolumna, margines 16, mobilne zakładki |
| 600–1023 | Jedna/dwie kolumny zależnie od treści, margines 24, rail opcjonalny |
| 1024+ | Panel z sidebar 240, treść max 1200, margines 32; formularz max 480 |

Breakpoint jest wynikiem dostępnej szerokości, nie nazwy urządzenia. Safe area i klawiatura nie mogą zasłaniać CTA. Listy zachowują pozycję po powrocie. Modal i panel szczegółów nie usuwają kontekstu trasy. Nazwy zakładek są zawsze widoczne; aktywna ma ikonę/wypełniony znacznik + `primary`, pozostałe `textSecondary`.

Pasażer: **Start / Mapa / Bilety / Konto**. Start pokazuje bieżącą/najbliższą podróż albo wyraźny wybór „Znajdź połączenie”; aktywny bilet ma krótki dostęp. Guardian: sekcja **Przejazdy dzieci** z osobnymi kartami dzieci i chronologią, nigdy wspólny niejednoznaczny status. Driver: **Bieżący kurs / Przystanki / Zdarzenia**, duży kolejny przystanek; operacje wymagające uwagi wykonywane podczas postoju. Chaperone: **Lista dzieci / Przystanek / Zdarzenia**, pojedyncza, jawna akcja dla konkretnego dziecka. Controller: **Skanuj / Wyniki / Synchronizacja**, czytelny tryb offline i ostatnia synchronizacja. Admin: sidebar **Operacje, Trasy i rozkłady, Flota, Transport szkolny, Użytkownicy, Raporty, Ustawienia**, widoczna organizacja i rola.

To docelowa architektura informacji dla dostępnych uprawnień. Nie dodawać pustych zakładek dla niezaimplementowanych modułów. Przełączanie roli wymaga jawnej, autoryzowanej zmiany kontekstu i zachowania aktualnego statusu sesji. Ukrycie przycisku nie zastępuje autoryzacji.

## 6. Kontrakty komponentów

### Akcje, formularze i wybór

- `Button`: primary (wypełniony), secondary (surface + borderStrong), quiet (tekst primary), danger (danger + onDanger). Min. 48 wysokości i szerokości obszaru trafienia, padding poziomy 16/24, radius 12, tekst 16/24 semibold. Ikona 20/24 i odstęp 8. W loading zachowuje rozmiar, ma nazwę trwającej operacji, blokuje ponowne wysłanie. Disabled pokazuje przyczynę, jeśli nie wynika z kontekstu.
- `IconButton`: ikona 24 w polu 48 × 48, nazwa dostępności. Tooltip na web nie jest jedyną etykietą dla ważnej akcji. Kontrolki bezpieczeństwa min. 56 × 56, odstęp co najmniej 8.
- `TextField`: trwała etykieta nad polem, height min. 52, borderStrong 1, radius 12, padding 16. Placeholder jest przykładem, nie nazwą pola. Helper pod polem, error z ikoną i konkretną instrukcją. Required opisane tekstowo; błąd skojarzony semantycznie z polem. Klawiatura i autofill zgodne z typem danych; hasło z nazwanym przyciskiem pokaż/ukryj.
- Focus: obręcz `focus` 2 px z odsunięciem 2 px i neutralną przerwą na wypełnionych przyciskach; widoczna bez przesuwania layoutu. Nie ukrywać systemowego focus bez zastępstwa. Kolejność Tab zgodna z układem.
- Checkbox/radio/switch mają podpis i obszar 48. Zmiana stanu nie przenosi użytkownika na inny ekran. Checkbox służy wyborowi; switch natychmiastowej, odwracalnej preferencji. Wiersz wyboru zaznaczyć również checkmarkiem i dostępnym `selected`/`checked`.
- Wyszukiwarka: nazwana, z wyczyszczeniem i wynikiem liczbowym; debounce nie może ukrywać stanu loading. Wyniki „brak”, „błąd”, „offline” są różne.

### Karty, listy, tabele i dialogi

- `Card`: surface, radius 16, padding 16/24, cienki dekoracyjny border; interaktywna dodatkowo zachowuje widoczny focus. Nie zagnieżdżać przycisku wewnątrz całkowicie klikalnej karty z inną akcją. Preferować listę z separatorami dla gęstych danych.
- `ListRow`: główna etykieta 16, metadane 14, minimum 56 wysokości; może urosnąć. Akcja i status są w stałych miejscach. Nie ukrywać akcji wyłącznie pod gestem swipe.
- `StatusBadge`: ikona 16 + etykieta 14 semibold, padding 4/8, kontrastowa para container. Statusy z tą samą nazwą i znaczeniem na wszystkich platformach.
- `DataTable`: nazwa tabeli, nagłówki kolumn, opis sortowania, fokusowalne akcje wiersza, widoczne zaznaczenie, klawiatura. Liczby wyrównane do prawej, tekst do lewej. Sticky header nie zasłania fokusu. Na małym ekranie lista kart z tymi samymi nazwami pól albo jawny scroll poziomy tylko w tabeli. Filtry zachowują wartości po błędzie; akcja zbiorcza pokazuje liczbę i skutki.
- `Dialog`: surfaceRaised, radius 24, max 480/560 zależnie od treści; tytuł, opis, akcje. Web przenosi focus do dialogu, zamyka Esc jeśli operacja na to pozwala, więzi focus i oddaje go do wywołującego. Native ma poprawną semantykę modal i systemowy Back. Destrukcyjne potwierdzenie nazywa obiekt i skutek, np. „Usuń trasę 12”; nie używać samego „Tak”.
- `BottomSheet`: uchwyt jest dekoracyjny; istnieje nazwany przycisk zamknięcia. Nie wymaga precyzyjnego przeciągania. Krytycznej operacji nie usuwa przypadkowe dotknięcie tła.
- `Toast`: tylko potwierdzenia odwracalne, pozycja nad nawigacją, nie zasłania akcji. Informacje wymagające działania mają trwały banner lub inline state. Nie polegać na znikającym toast do potwierdzenia bezpieczeństwa dziecka.

## 7. Wzorce transportowe

### Mapy i dane aktualizowane na żywo

Mapa jest dodatkiem do dostępnej listy przystanków/pojazdów, nie jedyną drogą wykonania zadania. Przełącznik „Mapa / Lista” zachowuje filtr i wybór. Wersja listowa umożliwia wybór trasy, przystanku i szczegółów bez gestów mapy. Przy odmowie lokalizacji wyszukiwanie ręczne pozostaje dostępne.

Przystanek = koło z kontrastowym środkiem; wybrany przystanek = dodatkowy zewnętrzny pierścień i etykieta, **nie czerwony marker**. Pojazd = kształt kapsuły/strzałki z ikoną autobusu i numerem linii. Pozycja użytkownika = punkt z pierścieniem dokładności i etykietą „Twoja lokalizacja”, bez ciągłego pulsu. Trasa wybrana ma grubszy ciągły przebieg, inne cieńszy; dostępny wykaz powtarza numer i kierunek.

Wszystkie markery mają jasny obrys oraz ciemny kontur zewnętrzny, aby oddzielić je od nieprzewidywalnego tła mapy. Kolory linii z API traktować jako dane, nie semantyczne tokeny UI: numer na neutralnej plakietce; jeśli używany jest kolor jako tło, obliczyć kontrast czerni/bieli, inaczej powrócić do primaryContainer. Nie używać samego koloru do odróżniania linii. Route IDs i litery stanowią legendę. Granice, etykiety i grafika mapy wymagają osobnej kontroli wizualnej w light/dark.

Pasek pod mapą pokazuje czas ostatniej aktualizacji. Stale GPS ma opis „Ostatnia pozycja: 14:32”, a nie „Na żywo”. Próg świeżości musi wynikać z kontraktu telemetrii; frontend nie wymyśla go per ekran. Nie animować ruchu pojazdu bez nowych danych. Nie ukrywać atrybucji dostawcy map. Motyw kontrolek i popupów pochodzi z tych samych tokenów; samo przyciemnienie mapy nie jest pełnym dark mode.

### Rozkłady i przystanki

Wiersz odjazdu: numer linii → kierunek → czas → status. Czas absolutny `14:32` pozostaje dostępny obok `za 5 min`; opóźnienie ma `+4 min` i opis, nie sam czerwony tekst. Odróżniać czas rozkładowy od prognozy live ikoną i etykietą. Pokazać datę dnia obsługi, strefę czasową przewoźnika gdy kontekst niejednoznaczny, dostępność dla wózków i numer stanowiska gdy znane. Nie prezentować nieznanych udogodnień jako braku. Długie nazwy zawijają się. Wybrany dzień i filtry są dostępne klawiaturą i dla czytnika.

### Bilety, płatność i skaner QR

Karta biletu pokazuje stan, typ, trasę/strefę, okres ważności i identyfikator. Rozróżnić: oczekuje na płatność, gotowy do użycia, aktywny, wykorzystany, wygasły, anulowany, błąd weryfikacji. Status bierze się z domeny, nie z koloru ani samego zegara telefonu. Błąd sieci przy weryfikacji to „Nie udało się sprawdzić biletu”, nigdy „Bilet nieważny”.

QR to wyjątek od motywu: czarny kod na czystej bieli, niezależnie od dark mode; cztery moduły quiet zone z każdej strony, bez logo, gradientu, zaokrąglania modułów i nałożonego tekstu. Wielkość minimalna obszaru prezentacji 240 dp, a końcowy moduł co najmniej 4 fizyczne px; jeśli gęstość kodu tego wymaga, powiększyć lub otworzyć pełny ekran. Generator zachowuje rzeczywistą treść i wymagany poziom korekcji błędów. Testować skan na realnym urządzeniu. Dostępny opis pokazuje typ i ważność biletu, nie czyta surowego sekretu/tokena QR.

Skaner: stabilna ramka, instrukcja, latarka, zamknięcie, uprawnienie kamery z alternatywną instrukcją. Wynik jest pełną kartą z ikoną i nazwą stanu: check „Bilet ważny”, cross „Bilet nieważny”, cloud/slash „Nie zweryfikowano”. Dźwięk/haptyka są dodatkiem, możliwym do wyłączenia. Tryb offline musi wskazywać ograniczenia i status synchronizacji. Nie symulować offline-valid bez rzeczywistego mechanizmu walidacji.

Zakup: przed potwierdzeniem pokaż bilet, zakres, cenę, walutę, ewentualne opłaty i sposób płatności. Kwoty nie zależą od języka UI. Oczekiwanie na potwierdzenie dostawcy zachowuje widoczny stan i zapobiega podwójnemu zakupowi. Dane płatnicze nie trafiają do toastów i logów UI.

### Transport szkolny i krytyczne alerty

Nazwy ról w interfejsie: Guardian = **opiekun prawny**, Chaperone = **opiekun przejazdu**. Nie używać samego „opiekun”, gdy obie role występują w jednym procesie. Lista dzieci ma jednoznaczny kontekst kursu i przystanku; dane osobowe tylko w uprawnionym widoku. Push na zablokowanym ekranie zawiera minimalną treść („Aktualizacja przejazdu”), szczegóły po uwierzytelnieniu.

Chronologia: zaplanowany → potwierdzono wejście → w przejeździe → potwierdzono wyjście/przekazanie, z czasem i źródłem potwierdzenia. Zdarzenie GPS może opisać przyjazd pojazdu, ale samo nie dowodzi wejścia, wyjścia ani bezpiecznego przekazania dziecka. To doprecyzowanie prezentacji wobec historycznej propozycji `AutoGPS` w `AI_DEVELOPMENT_PLAN_PHASE4.md`; wdrożenie potwierdzeń wymaga kontraktu domenowego.

| Poziom | Prezentacja | Zachowanie |
| --- | --- | --- |
| Informacja, np. zbliżanie pojazdu | `infoContainer`, ikona info, czas | Nie przerywa pracy, trafia do historii |
| Uwaga, np. opóźnienie lub brak świeżych danych | `warningContainer`, trójkąt, konkretny opis | Trwały banner, dostępna instrukcja działania |
| Krytyczny incydent lub brak wymaganego potwierdzenia | `dangerContainer`, ikona alert, nagłówek „Wymaga pilnej uwagi” | Trwały panel wysoko na ekranie, czas/źródło, odpowiedzialny kontekst, nazwana akcja i historia |

Nie każdy błąd formularza staje się krytycznym alertem. Alert krytyczny nie miga, nie znika automatycznie i nie jest zasłonięty toastem. Czytnik ogłasza nowy alert raz; kolejne pozycje GPS nie powtarzają komunikatu. Akcja „Potwierdź odczyt” nie oznacza rozwiązania zdarzenia; stan „Rozwiązano” wymaga potwierdzenia z systemu. Akcja kontaktu ma rzeczywisty, uprawniony cel. Nie wstawiać przykładowych numerów alarmowych, fikcyjnych osób ani autozakończenia. W stanie offline wyraźnie rozróżniać „zapisano na urządzeniu” i „dostarczono”.

## 8. Stany, dostępność i język

Każdy ekran danych ma: initial loading, refresh z zachowaniem treści, empty, error z retry, offline, partial/stale, permission denied i success tam, gdzie dotyczy. Skeleton odtwarza szkielet treści bez migotania; przy reduced-motion statyczny. Czytnik dostaje krótką nazwę ładowania, nie wiele pustych kształtów. Retry nie kasuje wprowadzonych danych. Nie blokować całej aplikacji awarią jednej sekcji.

Touch target domyślnie 48 × 48, safety 56 × 56. To reguła produktu, bardziej zachowawcza niż minimum [WCAG 2.2 Target Size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html). Jeśli ikona jest mniejsza, powiększyć obszar, pilnując braku nakładania hit areas. Test klawiatury obejmuje Tab/Shift+Tab/Enter/Space/Esc i przywracanie fokusu. Czytnik ma nagłówki, nazwy, role, value/state i komunikaty live tam, gdzie potrzebne; nie etykietować całego formularza w sposób ukrywający jego dzieci. TalkBack/VoiceOver nie odczytuje dekoracyjnych ikon.

Komunikaty: krótkie, spokojne, konkretne. Najpierw skutek, potem następny krok. „Nie udało się pobrać rozkładu. Spróbuj ponownie.” zamiast surowego błędu HTTP. „Lokalizacja jest niedostępna. Wyszukaj przystanek.” zamiast straszącego błędu. Unikać obietnic „zawsze”, „bezpiecznie” i „na żywo” bez danych. CTA nazywa działanie: „Pokaż bilet”, „Spróbuj ponownie”, „Zapisz trasę”. W stanach bezpieczeństwa żadnych żartów, wykrzykników ani języka marketingowego.

PL i EN są obowiązkowe dla nowych oraz zmienianych widoków. Wszystkie komunikaty, etykiety dostępności, alerty, walidacja i tekst generowany w WebView pochodzą z tłumaczeń. Nowe klucze: stabilne przestrzenie nazw + snake_case, zgodnie z `I18N.md`; nie zmieniać istniejących kluczy przy okazji bez migracji. Liczby mnogie przez mechanizm i18n, nie sklejanie. Sprawdzić „1 przystanek / 2 przystanki / 5 przystanków”, polskie diakrytyki i długie nazwy. Zmiana języka aktualizuje już otwarte widoki.

Języki nazywać „Polski” i „English”, bez flag. To zastępuje dawną sugestię flag w `I18N.md`. Daty/liczby/waluty formatować lokalnie, lecz walutę pobierać z transakcji; EN nie zamienia PLN na USD. Czas transportowy domyślnie 24-godzinny w obu językach dla spójności rozkładów; pełne daty i strefa z danych. Rezerwować około 30% więcej miejsca na tłumaczenia i testować realny tekst.

Wydajność: listy wirtualizowane, stabilne klucze, SVG bez filtrów, bez pełnoekranowego blur, brak fontów/ikon zależnych od sieci. Aktualizacja jednego pojazdu nie powinna przebudowywać całego ekranu. Sprawdzić przewijanie i reakcję na input na słabszym Androidzie; nie dodawać ciężkiej biblioteki tylko dla dekoracji.

## 9. Implementacja na platformach

### React Native / Expo — obecna aplikacja

`design/tokens.json` → adapter `mobile/src/theme/` → jeden provider/hook → Paper, React Navigation, wspólne komponenty i ekrany. Adapter zamienia jednostki i strukturę, nie tworzy nowej palety. `PaperProvider`, `NavigationContainer`, StatusBar i WebView muszą otrzymać ten sam wybrany motyw. Obecny `colors.ts` trzeba zastąpić lub zgodnie zmapować; nie pozostawiać starego pomarańczowego exportu jako drugiego źródła prawdy.

Mapowanie Paper: `primary←primary`, `onPrimary←onPrimary`, `primaryContainer←primaryContainer`, `onPrimaryContainer←onPrimaryContainer`, `background←background`, `onBackground←text`, `surface←surface`, `onSurface←text`, `surfaceVariant←surfaceMuted`, `onSurfaceVariant←textSecondary`, `outline←borderStrong`, `outlineVariant←border`, `error←danger`, `onError←onDanger`, `errorContainer←dangerContainer`, `onErrorContainer←onDangerContainer`. Pozostałe statusy udostępnić przez własny kontrakt theme. Paper MD3 default theme rozszerzać kompletnym adapterem zgodnym z zainstalowaną wersją; nie przekazywać niepełnego obiektu na ślepo.

React Navigation: primary, background, card=surface, text, border, notification=danger. Hook motywu reaguje na zmianę systemu i preferencji; przelicza StyleSheet tam, gdzie potrzebne. Kontrolki wspólne: `Screen`, `AppText`, `Button`, `Card`, `TextField`, `StatusBadge`, `EmptyState`, `ErrorState`, `BrandMark`. Istniejący Ionicons z Expo można wykorzystać za pojedynczym wrapperem: 24 px, wariant outline; nie dokładać drugiej biblioteki ikon. Odpowiedniki dostępności RN muszą zachować role, nazwy i stan.

Motyw WebView przekazywać jawnie serializowanymi tokenami oraz tłumaczeniami; treść danych nie trafia bez escapowania do HTML. Nie używać statycznego HTML z dawnymi niebieskimi/czerwonymi markerami. Aktualny brak części komponentów/importów jest kwestią implementacji, nie uzasadnieniem obejścia systemu.

### Flutter — docelowy klient

Ten sam JSON mapować do niemutowalnego `RoadRunnerTokens`/`ThemeExtension`, `ThemeData` i jawnego `ColorScheme`. Mapowanie znaczeń analogiczne do Paper; sukces, warning, info, stale/offline to ThemeExtension. Nie polegać na losowo generowanej palecie `fromSeed` jako źródle finalnych wartości. `TextTheme`, `InputDecorationTheme`, `FilledButtonTheme`, `CardTheme`, `NavigationBarTheme` dziedziczą wartości systemu. Respect `MediaQuery` text scaling i disable animations. Assets SVG współdzielone, app icon z tego samego mastera. Nazwy i stany domenowe pozostają identyczne z RN.

### Leptos / Tailwind / CSS variables — docelowy panel

Generować zmienne `--rr-color-primary`, `--rr-color-text`, `--rr-space-lg`, itd. z JSON; dark pod `[data-theme="dark"]`, system preference rozwiązywana przed paint. Klasy Tailwind odwołują się do zmiennych, np. tło primary = `var(--rr-color-primary)`. Nie stosować równoległych `blue-600`, `gray-500` i arbitralnych HEX w widokach. Nazwy camelCase JSON przechodzą na kebab-case CSS, np. `onPrimaryContainer` → `--rr-color-on-primary-container`.

Leptos komponenty odpowiadają kontraktom: Button, TextField, Card, StatusBadge, DataTable, Dialog, Alert, TransitDeparture, TicketCard. Używać natywnej semantyki HTML (`button`, `label`, `table`, `dialog` lub poprawny dialog ARIA), nie klikalnych divów. Frontend Tauri, jeśli pozostaje, korzysta z tego samego adaptera web. Brak projektu Flutter/Leptos w checkout nie jest powodem tworzenia pustego scaffolda w ramach samej migracji wyglądu.

## 10. Definition of Done każdej zmiany UI

- [ ] Użyto aktualnych tokenów i wspólnych komponentów; nowe wartości mają uzasadnienie w systemie, brak lokalnej palety.
- [ ] Light i dark oraz zmiana preferencji działają w treści, nawigacji, formularzu, popupie i WebView objętym zmianą.
- [ ] Logo pochodzi z `assets/brand`; proporcje, wariant i pole ochronne są poprawne.
- [ ] Każda akcja działa, ma nazwę i stan loading/disabled; brak martwych CTA oraz fikcyjnych danych podszywających się pod produkcyjne.
- [ ] Sprawdzono właściwe stany danych; awaria, offline, stale i empty nie są mylone z sukcesem.
- [ ] Teksty i nazwy dostępności są w PL/EN, daty/waluty poprawne, długie nazwy oraz polskie znaki mieszczą się.
- [ ] Kontrast par spełnia progi; kolor nie jest jedynym znacznikiem. Dynamiczne kolory linii/wykresów mają osobną kontrolę.
- [ ] Touch targets 48/56, logiczna kolejność fokusu, widoczny focus; klawiatura i screen reader mogą wykonać zadanie.
- [ ] Przy 320 px/dp, 200% tekstu i otwartej klawiaturze nie znika kluczowa treść ani akcja; orientacja i safe area są poprawne.
- [ ] Reduced-motion działa. Przewijanie i aktualizacja danych nie powodują nadmiernej pracy na słabszym Androidzie.
- [ ] QR, mapa i safety-critical flow spełniają odpowiednie reguły rozdziału 7, jeśli są dotknięte.
- [ ] Wykonano typecheck/build oraz adekwatny test funkcjonalny; dołączono zrzuty light/dark i opis sprawdzonych stanów. Niewykonane kontrole opisano uczciwie, nie oznaczono ich jako zaliczone.

Weryfikacja kontrastu tokenów jest automatyzowalna. Weryfikacja czytnika, realnego QR, zachowania klawiatury i jakości użycia na telefonie wymaga testu gotowej aplikacji. Nie ogłaszać pełnej zgodności WCAG na podstawie samego koloru.

## 11. Punkt startowy migracji

Audyt źródeł 2026-09-18: dokument v1 używa blue, `mobile/src/theme/colors.ts` orange, `App.tsx` nie podpina wybranego motywu do Paper, a ekran Start i nawigacja mają lokalne kolory. Start/Bilety zawierają twardy PL i akcje bez obsługi. WebView mapy ma własne kolory, czerwony selected i stały puls. Skaner miesza komunikaty systemowe z emoji. To uzasadnia kolejność: wspólne tokeny/provider → komponenty i logo → nawigacja/auth/Start/Konto/Bilety → mapa/rozkłady/zakup/skaner → kontrola dostępności i stanów.

Kolejne funkcje z backlogu rozpoczynać na nowym fundamencie po usunięciu blokujących regresji. Funkcje szkicowane w dokumentach, lecz nieistniejące w aplikacji, zachowują status planowanych. Wnioski szczegółowego audytu i wykonane kontrole utrzymywać w osobnym raporcie implementacyjnym, bez przepisywania historii tej specyfikacji.
