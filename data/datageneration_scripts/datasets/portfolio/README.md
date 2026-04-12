Voici la documentation en texte brut (plain text) pour les trois scripts, décrivant tous les paramètres possibles.

---

DOCUMENTATION DES SCRIPTS DE GÉNÉRATION DE DONNÉES FINANCIÈRES

1. amortization_matrix.py
2. outstanding_generator.py
3. run_generators_zip.py (orchestrateur)

================================================================================

I. PARAMÈTRES COMMUNS (fichier YAML)

- global (section optionnelle)
    start : string (MM-YYYY) – date de début commune aux deux générateurs
    end   : string (MM-YYYY) – date de fin commune

Si global est présent, les sections amortization et outstanding n'ont pas besoin de redéfinir start/end.

================================================================================

II. SECTION AMORTIZATION (matrice d'amortissement)

Exemple minimal :
amortization:
  start: "01-2020"
  end: "12-2024"
  periods:
    - maturity: 60
      curvature: 1.5
      curvature_std: 0.2

Paramètres détaillés :

- start : string (MM-YYYY) – obligatoire si global absent
- end   : string (MM-YYYY) – obligatoire si global absent
- output : string – nom du fichier CSV de sortie (défaut "amortization_matrix.csv")
- periods : liste d'objets (obligatoire, au moins un élément)
    Chaque objet period peut contenir :
        - start_period : string (MM-YYYY) – optionnel pour la première période,
          obligatoire pour les suivantes. La période s'applique à toutes les dates
          de début >= start_period et < start_period de la période suivante.
          Si omis pour la première période, elle s'applique à toutes les dates
          avant le start_period de la deuxième période (ou à tout l'intervalle).
        - maturity : entier (>=1) – nombre de mois d'amortissement (colonnes)
        - curvature : float – paramètre de forme (défaut 1.0)
            * curvature = 1 -> linéaire
            * curvature > 1 -> convexe (amortissement plus rapide au début)
            * 0 < curvature < 1 -> concave (amortissement plus lent au début)
        - curvature_std : float – écart-type pour randomiser la courbure
          (défaut 0.0). Si >0, chaque ligne reçoit une courbure aléatoire
          selon une loi normale N(curvature, curvature_std). La valeur est
          tronquée à 0.01 minimum.

La matrice générée contient une colonne "date" (MM-YYYY) et M colonnes numérotées
de 1 à M (M = maturity maximale parmi toutes les périodes). Les colonnes au-delà
de la maturity de chaque ligne sont remplies de 0.

================================================================================

III. SECTION OUTSTANDING (générateur de montants outstanding)

Exemple minimal :
outstanding:
  start: "01-2020"
  end: "12-2024"
  trend:
    type: "linear"
    start_value: 1000
    end_value: 5000
  noise:
    type: "absolute"
    std: 50
  random_seed: 42

Paramètres détaillés :

- start : string (MM-YYYY) – obligatoire si global absent
- end   : string (MM-YYYY) – obligatoire si global absent
- output : string – nom du fichier CSV de sortie (défaut "outstanding.csv")
- random_seed : entier – optionnel, fixe la graine aléatoire pour reproductibilité
- trend : objet (obligatoire)
    - type : string – parmi "linear", "exponential", "convex", "concave", "logistic"
    - start_value : float – valeur au premier mois
    - end_value   : float – valeur au dernier mois
    - curvature : float – requis pour "convex" et "concave" (exposant de la loi puissance)
        * convex : curvature > 1 (accélération)
        * concave : 0 < curvature < 1 (décélération)
    - logistic_midpoint : float – requis pour "logistic", entre 0 et 1 (défaut 0.5)
        Position du point d'inflexion (progression où la croissance est la plus rapide)
    - logistic_steepness : float – requis pour "logistic" (défaut 10)
        Pente de la courbe logistique (plus élevé = transition plus brutale)
- noise : objet (optionnel, absence = pas de bruit)
    - type : string – "absolute" ou "relative" (défaut "absolute")
    - std : float – requis si type="absolute", écart-type absolu du bruit gaussien
    - rel : float – requis si type="relative", fraction relative du bruit
        (ex: rel=0.05 -> bruit = 5% de la valeur courante)

La série générée suit d'abord la tendance déterministe, puis on ajoute un bruit
gaussien (additif). Les valeurs sont ensuite tronquées à zéro (pas de négatif).

================================================================================

IV. SCRIPT ORCHESTRATEUR (run_generators_zip.py)

Ce script appelle les deux générateurs à partir d'un seul YAML et produit un ZIP
contenant les deux CSV nommés avec un UUIDv7.

Utilisation :
python run_generators_zip.py --config config.yaml [--zip-output archive.zip]

Arguments :
--config      : chemin vers le fichier YAML (obligatoire)
--zip-output  : nom du fichier ZIP de sortie (optionnel). Par défaut : <uuid>.zip

Le YAML doit contenir les sections "amortization" et "outstanding" (et optionnellement "global").

Le script génère deux CSV temporaires nommés :
    <uuid>_amortization.csv
    <uuid>_outstanding.csv
Les compresse dans le ZIP, puis supprime les fichiers temporaires.

Note : l'UUID utilisé est un UUID version 7 (basé sur timestamp + aléatoire),
assurant l'ordre chronologique et l'unicité.

================================================================================

V. FORMAT DES FICHIERS DE SORTIE

1. Matrice d'amortissement (CSV)
   - Première colonne : date (MM-YYYY)
   - Colonnes suivantes : 1, 2, ..., M (M = maturité maximale)
   - Chaque cellule contient le ratio restant (entre 0 et 1) arrondi à 6 décimales.

2. Outstanding (CSV)
   - Deux colonnes : date, outstanding
   - Les outstanding sont arrondis à 2 décimales.

3. ZIP
   - Contient les deux fichiers CSV avec le préfixe UUID commun.

================================================================================

VI. EXEMPLES COMPLETS

Exemple 1 : génération simple avec global

global:
  start: "01-2020"
  end: "12-2024"

amortization:
  output: "amort.csv"
  periods:
    - maturity: 60
      curvature: 1.2
      curvature_std: 0.1

outstanding:
  output: "out.csv"
  trend:
    type: "linear"
    start_value: 1000
    end_value: 2000
  noise:
    type: "relative"
    rel: 0.03
  random_seed: 123

Exemple 2 : plusieurs périodes d'amortissement

amortization:
  start: "01-2020"
  end: "12-2025"
  periods:
    - maturity: 48
      curvature: 1.0
    - start_period: "01-2022"
      maturity: 36
      curvature: 2.0
      curvature_std: 0.2
    - start_period: "01-2024"
      maturity: 24
      curvature: 0.8

Exemple 3 : outstanding avec tendance logistique

outstanding:
  start: "01-2020"
  end: "12-2026"
  trend:
    type: "logistic"
    start_value: 500
    end_value: 5000
    logistic_midpoint: 0.6
    logistic_steepness: 8
  noise:
    type: "absolute"
    std: 100

================================================================================

FIN DE LA DOCUMENTATION