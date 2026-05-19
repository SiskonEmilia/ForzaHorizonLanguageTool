# FH Language Combo Tool

FH Language Combo Tool pozwala na uzycie roznych jezykow dla glosu i tekstu w Forza Horizon 5 / 6 (Steam, PC).

Na przyklad: polski tekst + angielski glos, lub polski tekst + japonski glos.

## Pobieranie

Pobierz najnowsza wersje ze strony [Releases](https://github.com/SiskonEmilia/ForzaHorizonLanguageTool/releases).

- **Wersja przenoszna**: `FH-Language-Combo-Tool-Portable.exe` — uruchom bezposrednio, bez instalacji
- **Instalator**: `FH-Language-Combo-Tool-Setup.exe` — instaluje w systemie

## Szybki start

1. Pobierz i uruchom narzedzie
2. Zaakceptuj klauzule o wylaczeniu odpowiedzialnosci
3. Narzedzie automatycznie wykrywa instalacje FH5/FH6 w Steam
4. Wybierz **Jezyk glosu** — jezyk, ktory chcesz SLYSZEC (glosy postaci)
5. Wybierz **Jezyk tekstu** — jezyk, ktory chcesz WIDZIEC (menu, napisy)
6. Kliknij **Zastosuj**
7. Potwierdz operacje
8. Uruchom gre — gotowe!

## Jak to dziala

Narzedzie kopiuje plik jezyka tekstu na pozycje pliku jezyka glosu w katalogu `StringTables` gry, a nastepnie automatycznie ustawia jezyk uruchamiania gry na wybrany jezyk glosu. Gra laduje dzwiek glosu z jednego jezyka, ale wyswietla tekst z drugiego.

## Przywracanie

Kliknij **Przywroc kopie zapasowa** w dowolnym momencie, aby cofnac wszystkie zmiany i przywrocic stan poczatkowy.

## Po aktualizacjach gry

Aktualizacje gry moga zresetowac konfiguracje jezykowa. Jesli tak sie stanie, po prostu ponownie zastosuj te same ustawienia — zajmie to kilka sekund.

## Czesto zadawane pytania

**P: Czy musz najpierw pobrac pakiety jezykowe w Steam?**
O: Tak. W Steam kliknij prawym przyciskiem na gre → Wlasciwosci → Jezyk i upewnij sie, ze oba jezyki (glosu i tekstu) sa pobrane.

**P: Czy to jest bezpieczne?**
O: Narzedzie modyfikuje tylko pliki zasobow tekstowych (`.zip` w `StringTables`). Nie dotyka plikow wykonywalnych, zapisow gry ani systemu anti-cheat. Wszystkie zmiany sa archiwizowane i odwracalne.

**P: Czy zostane zbanowany?**
O: To narzedzie nie modyfikuje rozgrywki, danych sieciowych ani komponentow anti-cheat. Zmienia tylko lokalne pliki tekstowe. Jednak korzystasz na wlasne ryzyko.

## Wylaczenie odpowiedzialnosci

To jest nieoficjalne narzedzie. Nie jest powiazane z Playground Games, Turn 10 Studios, Xbox ani Microsoft. Korzystasz na wlasne ryzyko.
