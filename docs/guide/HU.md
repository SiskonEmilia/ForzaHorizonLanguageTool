# FH Language Combo Tool

Az FH Language Combo Tool lehetove teszi, hogy kulonbozo nyelveket hasznalj a hanghoz es a szoveghez a Forza Horizon 5 / 6 jatekban (Steam, PC).

Peldaul: magyar szoveg + angol hang, vagy magyar szoveg + japan hang.

## Letoltes

Toltsd le a legujabb verziot a [Releases](https://github.com/SiskonEmilia/ForzaHorizonLanguageTool/releases) oldalrol.

- **Hordozhato**: `FH-Language-Combo-Tool-Portable.exe` — kozvetlenul futtatható, telepites nem szukseges
- **Telepito**: `FH-Language-Combo-Tool-Setup.exe` — telepiti a rendszeredre

## Gyors kezdes

1. Toltsd le es inditsd el az eszkozt
2. Fogadd el a felelossegkizaro nyilatkozatot
3. Az eszkoz automatikusan eszleli az FH5/FH6 Steam telepiteset
4. Valaszd ki a **Hang nyelvet** — a nyelv, amit HALLANI szeretnel (szereplok hangjai)
5. Valaszd ki a **Szoveg nyelvet** — a nyelv, amit LATNI szeretnel (menuk, feliratok)
6. Kattints az **Alkalmazas** gombra
7. Erositsd meg a muveletet
8. Inditsd el a jatekot — kesz!

## Hogyan mukodik

Az eszkoz atmasolja a szovegnyelv fajlt a hangnyelv fajl helyere a jatek `StringTables` konyvtaraban, majd automatikusan beallitja a jatek inditasi nyelvet a valasztott hangnyelvre. A jatek az egyik nyelvbol tolti be a hangot, de a masik nyelven jeleníti meg a szoveget.

## Visszaallitas

Kattints a **Biztonsagi mentes visszaallitasa** gombra barmikor az osszes valtoztatas visszavonasahoz es az eredeti allapot visszaallitasahoz.

## Jatekfrissitesek utan

A jatekfrissitesek visszaallithatjak a nyelvi beallitasaidat. Ha ez tortennik, egyszeruen alkalmazd ujra ugyanazokat a beallitasokat — csak par masodpercet vesz igenybe.

## Gyakran ismetelt kerdesek

**K: Eloszor le kell toltenem a nyelvi csomagokat a Steam-ben?**
V: Igen. A Steam-ben kattints jobb egergombbal a jatekra → Tulajdonsagok → Nyelv, es gyozodjk meg rola, hogy mindket nyelv (hang es szoveg) le van toltve.

**K: Biztonsagos?**
V: Az eszkoz csak szoveges eroforras fajlokat modosit (`.zip` a `StringTables` mappaban). Nem nyul futtathato fajlokhoz, mentesekhez vagy csalas elleni rendszerekhez. Minden valtoztatas biztonsagi mentesre kerul es visszaallithato.

**K: Ki leszek tiltva?**
V: Ez az eszkoz nem modositja a jatekmenetet, halozati adatokat vagy csalas elleni komponenseket. Csak helyi szoveges fajlokat valtoztat meg. A hasznalat azonban sajat felelossegre tortenik.

## Felelossegkizaro nyilatkozat

Ez egy nem hivatalos eszkoz. Nincs kapcsolatban a Playground Games, Turn 10 Studios, Xbox vagy Microsoft cegekkel. Hasznalat sajat felelossegre.
