# Audyt UI v2 — React Native / Expo

Status: audyt statyczny, 2026-09-18. Zakres: bieżąca aplikacja `mobile/`; bez implementacji i bez zmian w backlogu. Podstawa oceny: `docs/DESIGN_SYSTEM_V2.md`, `docs/agents/ui-ux.md`, `design/tokens.json` i `assets/brand/README.md`.

## Wniosek wykonawczy

Aplikacja nie jest gotowa do migracji ekran po ekranie. Najpierw trzeba naprawić graf zależności i zbudować jeden fundament v2: adapter tokenów, provider preferencji System/Jasny/Ciemny, wspólne komponenty, spójny motyw Paper/React Navigation/StatusBar/WebView oraz działające i18n. Obecny osiągalny ekran mapy importuje nieistniejące moduły, a pięć istniejących ekranów nie jest w ogóle zarejestrowanych w nawigacji.

Stan statyczny:

- **P0: 5**, **P1: 10**, **P2: 6**.
- 14 plików TSX w `mobile/`, w tym pusty artefakt `mobile/src/screens/.tsx`.
- 4 osiągalne zakładki: Start, Mapa, Bilety, Konto; 2 osiągalne ekrany auth.
- 5 odłączonych ekranów: zakup, skaner QR, szczegóły trasy, szczegóły przystanku, tracking.
- 100 dopasowanych linii z lokalnymi literałami HEX w 12 plikach; stary `theme/colors.ts` nie ma importerów.
- 0 wystąpień `accessibilityLabel`, `accessibilityRole`, `accessibilityState`, `accessibilityLiveRegion` i `hitSlop`.
- 0 implementacji stanów `offline`/`stale` i 0 obsługi reduced motion.
- Katalogi PL i EN mają po 170 kluczy o zgodnej strukturze, ale 16 statycznie używanych kluczy nie istnieje w żadnym z nich.
- Brak UI transportu szkolnego, guardian/chaperone/controller i krytycznych alertów. To luka funkcjonalna, nie istniejąca implementacja do „przestylowania”; nie należy tworzyć atrap w ramach migracji.

## Metoda i ograniczenia

Przejrzano wszystkie indeksowane pliki React Native/Expo, graf importerów, symbole ekranów, osadzone HTML map, katalogi PL/EN, konfigurację Expo i zależności pakietu. Audyt nie uruchamiał aplikacji na urządzeniu, dlatego nie stanowi potwierdzenia TalkBack/VoiceOver, realnego skanowania QR, klawiatury, 200% tekstu ani kontrastu warstwy mapowej.

`npm run typecheck` nie istnieje. Próba `npx tsc --noEmit` nie wykonała kompilacji, ponieważ lokalny TypeScript nie był dostępny i `npx` odwołał się do niewłaściwego pakietu `tsc`. Poniższe problemy kompilacji wynikają z grafu źródeł i zależności, nie z ukończonego typechecku.

## P0 — blokery migracji i wydania

### P0-1. Osiągalna zakładka Mapa ma nierozwiązywalny graf zależności

`AppNavigator.MainTabs` bezwarunkowo importuje `MapScreen`, a `MapScreen` importuje nieistniejące w repozytorium `../components/ThemedView`, `ThemedText`, `Card`, `Button`, `../hooks/useTheme` i `../services/api`. Te same braki występują w `RouteDetailsScreen` i `StopDetailsScreen`. `mobile/package.json` nie deklaruje też użytych bezpośrednio `expo-location`, `expo-camera` ani `react-native-webview`.

**Pliki/symbole:** `mobile/src/navigation/AppNavigator.tsx::MainTabs`, `mobile/src/screens/MapScreen.tsx::MapScreen`, `mobile/src/screens/RouteDetailsScreen.tsx::RouteDetailsScreen`, `mobile/src/screens/StopDetailsScreen.tsx::StopDetailsScreen`, `mobile/src/screens/QRScannerScreen.tsx::QRScannerScreen`, `mobile/package.json`.

**Warunek zamknięcia:** jeden rzeczywisty klient API, istniejące wspólne komponenty i hook motywu, jawne zależności, działający typecheck/bundle zanim zacznie się ocena wizualna.

### P0-2. Design System v2 nie jest podłączony do aplikacji

`App` przekazuje do `PaperProvider` motyw domyślny, `NavigationContainer` nie otrzymuje motywu, a `StatusBar` ma tylko `auto`. `mobile/src/theme/colors.ts` zawiera starą paletę pomarańczową, lecz nie ma importerów. Nie ma preferencji System/Jasny/Ciemny ani adaptera `design/tokens.json`. `mobile/app.json` wymusza `userInterfaceStyle: light` i wskazuje stare ikony/splash.

**Pliki/symbole:** `mobile/App.tsx::App`, `mobile/src/navigation/AppNavigator.tsx::AppNavigator`, `mobile/src/theme/colors.ts::{brandColors,lightTheme,darkTheme}`, `mobile/app.json`.

**Warunek zamknięcia:** jeden provider/hook wylicza aktywny motyw i zasila Paper, Navigation, StatusBar, komponenty i WebView; preferencja jest trwała, a pierwsze uruchomienie respektuje system.

### P0-3. i18n nie jest inicjalizowane, a osiągalne UI odwołuje się do brakujących kluczy

`App.tsx` wykonuje wyłącznie side-effect import `./src/i18n`; `initializeI18n()` nie jest wywoływane nigdzie. Dodatkowo brakuje 16 używanych kluczy: `auth.alreadyHaveAccount`, `auth.createAccount`, `auth.dontHaveAccount`, `auth.loginButton`, `auth.phone`, `auth.registerButton`, `auth.verify`, `errors.error`, `errors.invalidCredentials`, `errors.somethingWentWrong`, `navigation.home`, `navigation.map`, `navigation.profile`, `navigation.tickets`, `validation.emailRequired`, `validation.passwordMatch`.

**Pliki/symbole:** `mobile/App.tsx::App`, `mobile/src/i18n/index.ts::initializeI18n`, `mobile/src/i18n/locales/{pl,en}.json`, `AppNavigator.MainTabs`, `LoginScreen`, `RegisterScreen`, `TicketsScreen`.

**Warunek zamknięcia:** aplikacja czeka na inicjalizację bez flasha kluczy; wszystkie używane klucze istnieją w PL/EN; przełączenie języka aktualizuje otwarte widoki.

### P0-4. Graf nawigacji i podstawowe akcje są niespójne

`BuyTicketScreen`, `QRScannerScreen`, `RouteDetailsScreen`, `StopDetailsScreen` i `TrackingScreen` mają po 0 importerów. Mapa próbuje nawigować do niezarejestrowanego `StopDetails`. Karty Start, CTA „Kup bilet” oraz pozycje Konto (profil, hasło, MFA, język, powiadomienia) nie mają `onPress`. Typy nawigacji są zduplikowane i sprzeczne między `AppNavigator.tsx` i `types/index.ts` (`Main` vs `MainTabs`, `Home` vs `Routes`).

**Pliki/symbole:** `mobile/src/navigation/AppNavigator.tsx::{RootStackParamList,MainTabParamList,MainTabs,AppNavigator}`, `mobile/src/types/index.ts::{RootStackParamList,MainTabParamList}`, `HomeScreen`, `TicketsScreen`, `ProfileScreen`, pięć odłączonych ekranów.

**Warunek zamknięcia:** jedno źródło typów tras; każda widoczna akcja działa albo nie jest eksponowana; ekrany domenowe są jawnie zarejestrowane lub usunięte jako niegotowe.

### P0-5. Osadzone mapy nie mają bezpiecznego kontraktu danych

HTML z `MapScreen` buduje popup przez interpolację `stop.name`, `stop.address`, numeru i koloru linii bez escapowania. `RouteDetailsScreen` wstawia `stop_name` do HTML popupu, a kolor z API do CSS. WebView dopuszcza `originWhitelist=['*']`, ładuje skrypt i CSS z publicznego CDN oraz wyłącza atrybucję OpenStreetMap. Motyw i tłumaczenia nie są serializowane z aplikacji.

**Pliki/symbole:** `mobile/src/screens/MapScreen.tsx::MAP_HTML_TEMPLATE`, `RouteDetailsScreen.tsx::ROUTE_MAP_HTML`, `StopDetailsScreen.tsx::MINI_MAP_HTML`.

**Warunek zamknięcia:** escapowany/ustrukturyzowany most danych, ograniczony origin i polityka nawigacji, zachowana atrybucja, jawne tokeny i PL/EN w WebView; zależności mapy nie mogą być pojedynczym sieciowym punktem awarii bez stanu błędu.

## P1 — wymagane w migracji ekranów

### P1-1. Formularze auth nie spełniają kontraktu pól i błędów

Pola mają tylko placeholder, brak trwałych etykiet, helper/error powiązanego z polem, oznaczenia required i pokaż/ukryj hasło. Jeden błąd „emailRequired” obsługuje kilka pustych pól, błędy API są przekazywane surowo, a walidacja żyje wyłącznie w `Alert.alert`.

**Pliki/symbole:** `mobile/src/screens/auth/LoginScreen.tsx::LoginScreen`, `RegisterScreen.tsx::RegisterScreen`.

### P1-2. Brak semantyki dostępności i zbyt małe cele dotykowe

W całym `mobile/` nie ma jawnych nazw/ról/stanów dostępności. Ikona czyszczenia wyszukiwarki ma obszar samej ikony 20 px, zamknięcie skanera 44×44, chipy linii/dni i część przycisków są niższe niż 48 dp. Zaznaczenie filtrów nie ma `selected`, loading nie jest ogłaszany, ikony dekoracyjne nie są ukryte.

**Pliki/symbole:** wszystkie ekrany; szczególnie `MapScreen` search/filter, `QRScannerScreen` close button, `RouteDetailsScreen`/`StopDetailsScreen` filtry i taby.

### P1-3. Stany API są niepełne i semantycznie zlewają się

Brak wspólnego rozróżnienia initial loading, refresh z treścią, empty, error+retry, offline, partial/stale i permission denied. Mapa po błędzie pokazuje pustą mapę po zniknięciu spinnera; route/stop po dowolnym błędzie pokazują `notFound`; Tickets zawsze pokazuje empty bez requestu; brak zachowania danych podczas retry.

**Pliki/symbole:** `MapScreen.loadData`, `RouteDetailsScreen.loadRouteData`, `StopDetailsScreen.loadStopData`, `TicketsScreen`, `BuyTicketScreen.purchaseTicket`, `QRScannerScreen.handleBarCodeScanned`.

### P1-4. Mapa nie ma dostępnej alternatywy ani poprawnych stanów lokalizacji

Brak przełącznika Mapa/Lista. Odmowa lokalizacji jest ignorowana; ręczne wyszukiwanie wysyła request, lecz nie zapisuje ani nie renderuje wyniku (`showSearchResults` pozostaje martwym stanem). Atrybucja jest ukryta. Wybrany przystanek staje się czerwony, pozycja użytkownika stale pulsuje, brak czasu aktualizacji/stale i etykiet markerów.

**Pliki/symbole:** `MapScreen::{requestLocationPermission,handleSearch,renderRouteFilter,MAP_HTML_TEMPLATE}`.

### P1-5. Lista biletów nie przedstawia rzeczywistych biletów ani stanów domenowych

Ekran zawsze pokazuje „Brak biletów”, a CTA jest martwe. Nie ma fetchu, statusów pending/ready/active/used/expired/cancelled/verification error, identyfikatora, ważności, trasy/strefy, odświeżenia ani prezentacji QR.

**Plik/symbol:** `mobile/src/screens/TicketsScreen.tsx::TicketsScreen`.

### P1-6. Zakup wykorzystuje atrapę katalogu i zbyt wcześnie ogłasza sukces

`TICKET_TYPES` zawiera zakodowane nazwy, ważność, ceny, walutę i emoji; dane nie pochodzą z oferty/transakcji. Sukces jest ogłaszany bez jawnego stanu płatności/potwierdzenia dostawcy. Brak zakresu/strefy, opłat i sposobu płatności. Cała karta i zagnieżdżony przycisk uruchamiają tę samą akcję; `selectedTicket` nigdy nie jest ustawiane.

**Pliki/symbole:** `mobile/src/screens/BuyTicketScreen.tsx::{TICKET_TYPES,BuyTicketScreen,handleBuyTicket,purchaseTicket}`.

### P1-7. Skaner QR nie spełnia kontraktu kontrolera

Uprawnienie jest żądane automatycznie; brak trwałego stanu odmowy z instrukcją ustawień, latarki i trybu offline/synchronizacji. Wyniki są emoji w alertach zamiast pełnych kart z ikoną i nazwą stanu. Brak rozróżnienia „nie zweryfikowano” od nieważności na poziomie stałego widoku, brak opcji dźwięku/haptyki i wymaganych celów 56 dp dla krytycznej obsługi.

**Plik/symbol:** `mobile/src/screens/QRScannerScreen.tsx::QRScannerScreen`.

### P1-8. Marka v2 nie jest używana w aplikacji ani konfiguracji launchera

Brak referencji do `assets/brand`, `BrandMark` i lockupów. Logowanie, nagłówki i Start nie prezentują zatwierdzonego znaku. Expo nadal wskazuje `mobile/assets/icon.png`, `adaptive-icon.png`, `splash-icon.png` i białe tło zamiast eksportów z mastera Connected Route.

**Pliki:** `mobile/App.tsx`, ekrany auth/Start, `mobile/app.json`; nowy wrapper marki.

### P1-9. PL/EN jest tylko częściowe

Home, Profile, Tickets empty, BuyTicket, QRScanner, Tracking oraz teksty popupów mapy mają twardy polski. `LANGUAGES` nadal eksponuje flagi, choć kontrakt wymaga nazw „Polski”/„English”. Brak dostępnego selektora języka, mimo widocznej martwej pozycji „Język”.

**Pliki/symbole:** `HomeScreen`, `ProfileScreen`, `TicketsScreen`, `BuyTicketScreen`, `QRScannerScreen`, `TrackingScreen`, trzy stałe HTML map, `mobile/src/i18n/index.ts::LANGUAGES`.

### P1-10. Dane transportowe używają niekontrolowanych kolorów i niepełnej semantyki czasu

Kolory tras z API są bezpośrednio używane jako tła z białym tekstem oraz jako tekst na powierzchni, bez obliczenia kontrastu i neutralnego fallbacku. Rozkłady pokazują same godziny: bez rozróżnienia rozkład/live, dnia obsługi, strefy czasowej, opóźnienia i dostępności. Nieznany/nieobecny status nie ma osobnej prezentacji.

**Pliki/symbole:** `MapScreen.renderRouteFilter`, `RouteDetailsScreen::{renderStopsList,renderSchedule,ROUTE_MAP_HTML}`, `StopDetailsScreen::{renderRoutesList,renderSchedule,MINI_MAP_HTML}`.

## P2 — porządek i jakość po usunięciu blokerów

### P2-1. Typografia, spacing, radius i elevation są lokalne

Ekrany używają m.in. 10/12/13 px tekstu, paddingów 20/30/40 oraz wielu promieni i cieni poza kontraktem. Należy przejść na role typograficzne i tokeny; 14 px jest minimalnym rozmiarem roli `caption`, a karta v2 ma radius 16 i elevation 0/1 zależnie od potrzeby.

**Pliki:** wszystkie pliki ekranów ze `StyleSheet.create`.

### P2-2. Brak infrastruktury reduced motion

Nie ma odczytu preferencji ani wspólnego czasu animacji. Puls markera użytkownika działa stale. Każda przyszła animacja musi korzystać z tokenów 0/120/180/240 i mieć wariant bez przesunięcia/shimmeru.

**Pliki:** nowy provider/hook ruchu; `MapScreen.MAP_HTML_TEMPLATE`.

### P2-3. Layout nie ma udokumentowanej odporności na 320 dp, 200% tekstu i duże dane

Są stałe wymiary, poziome chipy i `numberOfLines={1}` dla nazw przystanków. Route/stop renderują duże zbiory przez `.map` wewnątrz `ScrollView`, bez wirtualizacji. Safe area nie otacza poszczególnych CTA, a skaner pozycjonuje close przez stałe `top: 50`.

**Pliki:** `MapScreen`, `RouteDetailsScreen`, `StopDetailsScreen`, `QRScannerScreen`, auth.

### P2-4. Formatowanie czasu, liczby mnogiej i waluty jest ręczne

Zakup używa `toFixed(2) + PLN`, a ekrany rozkładów `substring(0, 5)`. Brak formatterów locale, jawnej strefy i pluralizacji. EN nie może zmieniać waluty transakcji, ale format powinien wynikać z locale i danych.

**Pliki:** `BuyTicketScreen`, `RouteDetailsScreen`, `StopDetailsScreen`; nowy moduł formatterów.

### P2-5. Repo zawiera martwe i konkurencyjne artefakty UI

`mobile/src/screens/.tsx`, `TrackingScreen`-placeholder i stary `theme/colors.ts` nie mają importerów. W `types/index.ts` istnieją konkurencyjne typy motywu/nawigacji. Należy je usunąć albo włączyć do jednego kontraktu, nie pozostawiać jako alternatywnych wzorców dla kolejnych agentów.

### P2-6. Źródła design systemu różnią się liczbą kontroli kontrastu

`DESIGN_SYSTEM_V2.md` mówi o 52 parach na motyw, a `assets/brand/README.md` o 104 kontrolach. `design/tokens.json` zawiera pozycje `Item 1`–`Item 50`, czyli 50 par / 100 sprawdzeń dla dwóch motywów. Wartości kolorów są spójne z tabelą, ale liczba deklarowanych kontroli wymaga decyzji właściciela designu GPT-6 przed zmianą tokenów lub ogłoszeniem 104 zaliczonych testów.

## Macierz ekranów

| Powierzchnia | Osiągalność | Motyw/marka | PL/EN | Stany danych | A11y i główne ryzyko | Priorytet |
| --- | --- | --- | --- | --- | --- | --- |
| Bootstrap aplikacji | Tak | domyślny Paper, brak v2 | i18n nieuruchomione | brak jawnego init | brak kontrolowanego focus/status | P0 |
| Login + MFA | Tak | lokalny blue/light | część kluczy brak | loading + alert error; brak offline | placeholder jako label, brak show/hide | P0/P1 |
| Rejestracja | Tak | lokalny blue/light | część kluczy brak | loading + alert error | błędy nieskojarzone z polami | P0/P1 |
| Nawigacja dolna | Tak po auth | lokalny active blue | 4 brakujące klucze | n/d | brak ikon i aktywnego znacznika poza kolorem | P0 |
| Start | Tak | lokalny blue | twardy PL | statyczny, bez danych | 3 martwe karty-akcje | P0 |
| Konto | Tak | lokalny blue/red | twardy PL | logout bez stanu; reszta martwa | destrukcja bez wspólnego kontraktu | P0/P1 |
| Bilety | Tak | lokalny blue | mixed: tytuł key, reszta PL | zawsze empty | martwe CTA, brak ticket semantics | P0/P1 |
| Zakup biletu | Nie | lokalny blue | twardy PL | loading/success/error alert | atrapa cen, emoji, nested touch | P1 przed rejestracją |
| Skaner QR | Nie | lokalny blue/black | twardy PL | permission/valid/invalid/error częściowe | brak offline, latarki, pełnej karty wyniku | P1 przed rejestracją |
| Mapa | Rejestrowana, obecnie blokuje build | częściowy nieistniejący theme + statyczny HTML | część native key, popup PL | loading + alert; brak retry/offline/permission | brak listy, atrybucji, unsafe HTML, pulse | P0/P1 |
| Szczegóły trasy | Nie | dynamiczny kolor API + statyczna mapa | częściowe key, popup PL | loading; error=>notFound; empty schedule | brak kontrastu, brak daty/live | P0/P1 |
| Szczegóły przystanku | Nie | dynamiczny kolor API + statyczna mapa | częściowe key | loading; error=>notFound; empty schedule | brak kontrastu i listowej alternatywy mapy | P0/P1 |
| Tracking | Nie | Paper default | twardy PL | placeholder | emoji i fikcyjna obietnica przyszłej mapy | P2: nie eksponować |
| Transport szkolny / safety | Brak implementacji | n/d | n/d | n/d | brak ekranów i role-based IA; nie tworzyć atrap | backlog, obowiązują guardraile v2 |

## Ocena przekrojowa

### Tokeny i light/dark

Wartości `design/tokens.json` tworzą kompletną semantyczną paletę light/dark, spacing, radius, rozmiary, motion i typografię. Problemem aplikacji nie jest brak wartości, tylko brak adaptera i powszechne omijanie tokenów. `border` nie może identyfikować kontrolki; dynamiczne kolory linii wymagają osobnego algorytmu kontrastu. Do czasu wyjaśnienia P2-6 nie wolno twierdzić, że wykonano 104 kontrole.

### Dostępność

Nie ma dowodu na obsługę screen readera, focus order, stanu selected/disabled/loading ani nazw IconButton. Native `Text` domyślnie skaluje się, ale stałe wysokości, małe fonty i pojedyncze linie nie gwarantują pracy przy 200%. Migracja musi użyć wspólnych komponentów, bo naprawianie tych problemów per ekran ponownie rozproszy kontrakt.

### Stany API i offline

Obecne ekrany mieszają brak zasobu, pusty wynik i awarię. Wspólne `AsyncState`/komponenty powinny rozróżniać co najmniej: initial loading, refresh, empty, error+retry, offline, stale/partial, permission denied i success. Dane zachowujemy podczas refresh/retry; awaria sekcji nie blokuje całej aplikacji.

### Mapa, bilet i safety

- Mapa musi pozostać dodatkiem do listy, pokazywać źródło/czas danych, atrybucję i brak/stale lokalizacji bez obietnicy „na żywo”.
- QR biletu, gdy powstanie widok biletu, pozostaje czarny na białym z quiet zone 4 modułów i minimum 240 dp; żaden istniejący ekran tego jeszcze nie implementuje.
- Błąd sieci walidacji nie może stać się „bilet nieważny”; istniejący catch używa ogólnego błędu, ale wynik powinien być trwałą kartą „Nie zweryfikowano”.
- Nie znaleziono implementacji przekazania dziecka ani krytycznych alertów. GPS nie może później automatycznie potwierdzać wejścia/wyjścia/przekazania, a „Potwierdź odczyt” nie może oznaczać „Rozwiązano”.

## Lista plików do zmiany w migracji

### Fundament i integracja

- `mobile/App.tsx`
- `mobile/app.json`
- `mobile/package.json`
- `mobile/src/api/client.ts`
- `mobile/src/navigation/AppNavigator.tsx`
- `mobile/src/i18n/index.ts`
- `mobile/src/i18n/locales/pl.json`
- `mobile/src/i18n/locales/en.json`
- `mobile/src/theme/colors.ts` — zastąpić adapterem v2 albo usunąć po migracji
- `mobile/src/types/index.ts`
- `mobile/src/screens/.tsx` — usunąć po potwierdzeniu, że to artefakt
- nowe `mobile/src/theme/*`, `mobile/src/hooks/useTheme.ts`
- nowe `mobile/src/components/{Screen,AppText,Button,IconButton,Card,TextField,StatusBadge,EmptyState,ErrorState,BrandMark}.tsx`
- nowe testy adaptera, preferencji motywu, i18n i komponentów wspólnych

### Ekrany pasażera/konta/biletów

- `mobile/src/screens/auth/LoginScreen.tsx`
- `mobile/src/screens/auth/RegisterScreen.tsx`
- `mobile/src/screens/HomeScreen.tsx`
- `mobile/src/screens/ProfileScreen.tsx`
- `mobile/src/screens/TicketsScreen.tsx`
- `mobile/src/screens/BuyTicketScreen.tsx`
- `mobile/src/screens/QRScannerScreen.tsx`

### Mapa i transport

- `mobile/src/screens/MapScreen.tsx`
- `mobile/src/screens/RouteDetailsScreen.tsx`
- `mobile/src/screens/StopDetailsScreen.tsx`
- `mobile/src/screens/TrackingScreen.tsx` — usunąć placeholder albo pozostawić nieosiągalny do czasu realnej funkcji
- nowe testy sanitizacji mostu WebView, kontrastu kolorów linii, stanów mapy/listy i permission/offline

`design/tokens.json` nie powinien być zmieniany przez agenta implementacyjnego przed decyzją GPT-6 dotyczącą brakujących dwóch deklarowanych par kontrastu. Zatwierdzone SVG w `assets/brand/` są źródłem, nie miejscem do lokalnego retuszu.

## Bezkolizyjny plan dla 3 agentów `gpt-5.6-sol`

Wszyscy trzej agenci implementacyjni używają modelu **gpt-5.6-sol** i zaczynają od przeczytania pełnego kontraktu UI/UX. Własność designu i decyzje o tokenach pozostają po stronie GPT-6.

### Agent A — foundation/theme/navigation/shared

**Wyłączna własność plików:** `mobile/App.tsx`, `mobile/app.json`, `mobile/package.json`, `mobile/src/api/client.ts`, `mobile/src/navigation/AppNavigator.tsx`, `mobile/src/i18n/**`, `mobile/src/theme/**`, `mobile/src/hooks/**`, `mobile/src/components/**`, `mobile/src/types/index.ts`, `mobile/src/screens/.tsx`, testy fundamentu.

**Zakres:** naprawa grafu i zależności; adapter tokenów; provider i trwała preferencja System/Jasny/Ciemny; motywy Paper/Navigation/StatusBar; wrapper marki i ikon; wspólne komponenty/stany; inicjalizacja PL/EN; jedno typowanie tras; rejestracja tylko gotowych ekranów; launcher/splash z zatwierdzonych eksportów. Agent A jako jedyny edytuje katalogi tłumaczeń — B/C przekazują mu manifest nowych kluczy.

### Agent B — auth/home/profile/tickets/QR

**Wyłączna własność plików:** `LoginScreen.tsx`, `RegisterScreen.tsx`, `HomeScreen.tsx`, `ProfileScreen.tsx`, `TicketsScreen.tsx`, `BuyTicketScreen.tsx`, `QRScannerScreen.tsx` oraz testy tych ekranów.

**Zakres:** migracja na komponenty A; usunięcie martwych CTA/atrap; pełne stany auth, konta, listy biletu, zakupu i walidacji; ticket/QR semantics; a11y 48/56; PL/EN przez manifest kluczy. Bez edycji nawigacji, theme, klienta API i locale.

### Agent C — map/route/stop/tracking

**Wyłączna własność plików:** `MapScreen.tsx`, `RouteDetailsScreen.tsx`, `StopDetailsScreen.tsx`, `TrackingScreen.tsx` oraz testy tej grupy.

**Zakres:** dostępna para Mapa/Lista, permission/manual search, sanitizowany i tematyzowany most WebView, atrybucja, stale timestamp, kontrast kolorów tras, komplet stanów API, semantyka rozkładu/przystanku. Placeholder Tracking pozostaje niewidoczny albo zostaje usunięty w tym pliku; bez tworzenia fikcyjnego live tracking. Bez edycji plików A/B.

### Kolejność integracji

1. **Preflight GPT-6/coordinator:** rozstrzyga 50 vs 52 par kontrastu i zatwierdza eksporty launchera; nie blokuje pracy nad adapterem istniejących wartości.
2. **Agent A, etap 1:** dostarcza kompilujący fundament i stabilne publiczne API komponentów/motywu/nawigacji. Dopiero po tym B i C rozpoczynają zmiany ekranów.
3. **Agenci B i C równolegle:** pracują wyłącznie w swoich grupach; wysyłają A listę kluczy i wymaganych tras, bez dotykania wspólnych plików.
4. **Agent A, etap 2:** integruje manifesty locale i rejestruje wyłącznie ukończone ekrany. Coordinator rozwiązuje błędy na granicach; nie przenosi własności plików.
5. **Wspólna bramka:** lint, typecheck/bundle, testy, light/dark, PL/EN, 320 dp, 200% tekstu, klawiatura, TalkBack/VoiceOver, reduced motion, offline/stale, realne urządzenie QR i screenshoty wymaganych stanów.

## Kryteria rozpoczęcia dalszego backlogu

Dalsze funkcje można rozpocząć dopiero, gdy:

- P0-1–P0-5 są zamknięte i aplikacja buduje się bez nierozwiązywalnych importów;
- wszystkie osiągalne ekrany używają jednego motywu v2 i wspólnych komponentów;
- nie ma widocznych martwych CTA ani fikcyjnych danych podszywających się pod produkcyjne;
- PL/EN, light/dark, stany API i podstawowy audyt a11y przechodzą dla każdego osiągalnego flow;
- niewykonane kontrole manualne są jawnie zapisane, a zgodność WCAG nie jest deklarowana wyłącznie na podstawie tokenów.
