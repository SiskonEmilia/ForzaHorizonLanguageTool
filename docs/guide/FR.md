# FH Language Combo Tool

FH Language Combo Tool vous permet d'utiliser des langues differentes pour la voix et le texte dans Forza Horizon 5 / 6 (Steam, PC).

Par exemple : texte en francais + voix en anglais, ou texte en francais + voix en japonais.

## Telechargement

Telechargez la derniere version depuis la page [Releases](https://github.com/SiskonEmilia/ForzaHorizonLanguageTool/releases).

- **Portable** : `FH-Language-Combo-Tool-Portable.exe` — a executer directement, sans installation
- **Installateur** : `FH-Language-Combo-Tool-Setup.exe` — s'installe sur votre systeme

## Demarrage Rapide

1. Telechargez et lancez l'outil
2. Acceptez la clause de non-responsabilite
3. L'outil detecte automatiquement votre installation Steam de FH5/FH6
4. Selectionnez la **Langue vocale** — la langue que vous voulez ENTENDRE (voix des personnages)
5. Selectionnez la **Langue du texte** — la langue que vous voulez VOIR (menus, sous-titres)
6. Cliquez sur **Appliquer**
7. Confirmez l'operation
8. Lancez le jeu — c'est fait !

## Fonctionnement

L'outil copie le fichier de langue de texte a la position du fichier de langue vocale dans le repertoire `StringTables` du jeu, puis definit automatiquement la langue de demarrage du jeu sur la langue vocale choisie. Le jeu charge l'audio vocal d'une langue mais affiche le texte d'une autre.

## Restauration

Cliquez sur **Restaurer la sauvegarde** a tout moment pour annuler toutes les modifications et revenir a l'etat d'origine.

## Apres les Mises a Jour du Jeu

Les mises a jour du jeu peuvent reinitialiser votre configuration linguistique. Si cela se produit, reappliquez simplement les memes parametres — cela ne prend que quelques secondes.

## Questions Frequentes

**Q : Dois-je d'abord telecharger les packs de langues dans Steam ?**
R : Oui. Dans Steam, faites un clic droit sur le jeu → Proprietes → Langue, et assurez-vous que les deux langues (voix et texte) sont telechargees.

**Q : Est-ce securise ?**
R : L'outil ne modifie que les fichiers de ressources textuelles (`.zip` dans `StringTables`). Il ne touche ni aux executables, ni aux sauvegardes, ni a l'anti-triche. Toutes les modifications sont sauvegardees et reversibles.

**Q : Vais-je etre banni ?**
R : Cet outil ne modifie ni le gameplay, ni les donnees reseau, ni les composants anti-triche. Il ne modifie que des fichiers texte locaux. Cependant, l'utilisation se fait a vos propres risques.

## Clause de Non-responsabilite

Ceci est un outil non officiel. Sans affiliation avec Playground Games, Turn 10 Studios, Xbox ou Microsoft. Utilisation a vos propres risques.
