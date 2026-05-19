# FH Language Combo Tool

FH Language Combo Tool mahdollistaa eri kielten kayton aanelle ja tekstille Forza Horizon 5 / 6 -pelissa (Steam, PC).

Esimerkiksi: suomenkielinen teksti + englanninkielinen aani, tai suomenkielinen teksti + japaninkielinen aani.

## Lataaminen

Lataa uusin versio [Releases](https://github.com/SiskonEmilia/ForzaHorizonLanguageTool/releases)-sivulta.

- **Kannettava versio**: `FH-Language-Combo-Tool-Portable.exe` — aja suoraan, asennusta ei tarvita
- **Asennusohjelma**: `FH-Language-Combo-Tool-Setup.exe` — asentaa jarjestelmaasi

## Pikaopas

1. Lataa ja kaynnista tyokalu
2. Hyvaksy vastuuvapauslauseke
3. Tyokalu tunnistaa automaattisesti FH5/FH6 Steam-asennuksesi
4. Valitse **Aanikieli** — kieli, jonka haluat KUULLA (hahmojen aanet)
5. Valitse **Tekstikieli** — kieli, jonka haluat NAHDA (valikot, tekstitykset)
6. Napsauta **Kayta**
7. Vahvista toiminto
8. Kaynnista peli — valmis!

## Miten se toimii

Tyokalu kopioi tekstikielitiedoston aanikielitiedoston paikalle pelin `StringTables`-hakemistossa ja asettaa automaattisesti pelin kaynnistyskieleksi valitun aanikielen. Peli lataa aaniraidan yhdella kielella mutta nayttaa tekstin toisella.

## Palautus

Napsauta **Palauta varmuuskopio** milloin tahansa peruuttaaksesi kaikki muutokset ja palataksesi alkuperaiseen tilaan.

## Pelipaivitysten jalkeen

Pelipaivitykset saattavat nollata kieliasetuksesi. Jos nain tapahtuu, kayta samat asetukset uudelleen — se vie vain muutaman sekunnin.

## Usein kysytyt kysymykset

**K: Pitaako minun ensin ladata kielipaketit Steamissa?**
V: Kylla. Napsauta Steamissa pelia hiiren oikealla painikkeella → Ominaisuudet → Kieli ja varmista, etta molemmat kielet (aani ja teksti) on ladattu.

**K: Onko tama turvallista?**
V: Tyokalu muokkaa vain tekstiresurssitiedostoja (`.zip` `StringTables`-kansiossa). Se ei koske suoritettavia tiedostoja, tallennuksia tai huijauksenestoa. Kaikki muutokset varmuuskopioidaan ja ne voidaan kumota.

**K: Saanko porttikiellon?**
V: Tama tyokalu ei muokkaa pelattavuutta, verkkoliikennetta tai huijaukseneston komponentteja. Se muuttaa vain paikallisia tekstitiedostoja. Kaytto on kuitenkin omalla vastuullasi.

## Vastuuvapauslauseke

Tama on epavirallinen tyokalu. Ei yhteydessa Playground Gamesiin, Turn 10 Studiosiin, Xboxiin tai Microsoftiin. Kaytto omalla vastuulla.
