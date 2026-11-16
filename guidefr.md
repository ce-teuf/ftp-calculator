# Corporate Actions en Prime Brokerage US
### Guide complet : Opérations, Project Management & Automatisation Dagster

> **Pour qui :** PM / dev Python qui rejoint une équipe ITO Global Markets (NYC) et qui doit automatiser la détection des écarts (*breaks*) en Corporate Actions pour les opérations Prime Brokerage.
> **Convention :** texte en français, **keywords métier en anglais en gras** (c'est le vocabulaire que tu entendras au desk).
> **Classeur d'exemples :** chaque section illustrée renvoie à un exemple chiffré **EX-xx** du fichier `CA_PB_US_Exemples.xlsx` (onglet `Index` cliquable). Toutes les données y sont fictives.
> **Avertissement :** les éléments propres à ta banque (noms de systèmes, organisation, règles internes) sont marqués **[À VÉRIFIER EN INTERNE]**. Les règles fiscales et réglementaires évoluent : valide toujours avec Tax / Compliance avant de coder une règle.

---

## Sommaire

**Partie I — Fondamentaux**
1. [Les Corporate Actions en une page](#1-les-corporate-actions-en-une-page)
2. [Classification des événements](#2-classification-des-événements)
3. [Catalogue des types d'événements (codes ISO)](#3-catalogue-des-types-dévénements-codes-iso)
4. [Les dates clés et leurs spécificités US](#4-les-dates-clés-et-leurs-spécificités-us)

**Partie II — L'écosystème US**
5. [Les acteurs du marché US](#5-les-acteurs-du-marché-us)
6. [Les sources de données : Bloomberg, Broadridge, DTC…](#6-les-sources-de-données--bloomberg-broadridge-dtc)
7. [Messages et standards (SWIFT MT, ISO 20022)](#7-messages-et-standards-swift-mt-iso-20022)

**Partie III — Le Prime Brokerage**
8. [C'est quoi un Prime Broker](#8-cest-quoi-un-prime-broker)
9. [Comment sont structurées les positions d'un PB](#9-comment-sont-structurées-les-positions-dun-pb)
10. [Ce qui rend les CA plus complexes en PB](#10-ce-qui-rend-les-ca-plus-complexes-en-pb)

**Partie III-bis — Le Prime Brokerage en profondeur (ajout)**
- 10A. [Le business PB et ses clients](#10a-le-business-pb-et-ses-clients)
- 10B. [Le modèle économique](#10b-le-modèle-économique)
- 10C. [Organisation front-to-back d'un PB](#10c-organisation-front-to-back-dun-pb)
- 10D. [Le cycle quotidien d'un trade en PB](#10d-le-cycle-quotidien-dun-trade-en-pb)
- 10E. [La marge et le risque](#10e-la-marge-et-le-risque)
- 10F. [Equity Finance et Stock Loan](#10f-equity-finance-et-stock-loan)
- 10G. [Synthetic Prime en détail](#10g-synthetic-prime-en-détail)
- 10H. [Treasury, funding et collatéral](#10h-treasury-funding-et-collatéral)
- 10I. [Reporting, NAV et expérience client](#10i-reporting-nav-et-expérience-client)
- 10J. [Réglementation propre au PB](#10j-réglementation-propre-au-pb)
- 10K. [Matrice d'impact des CA sur les fonctions PB](#10k-matrice-dimpact-des-ca-sur-les-fonctions-pb)
- 10L. [Chronologie d'un événement vu du PB](#10l-chronologie-dun-événement-vu-du-pb)
- 10M. [Scénarios d'incidents typiques en PB](#10m-scénarios-dincidents-typiques-en-pb)
- 10N. [Conséquences pour ton projet](#10n-conséquences-pour-ton-projet)

**Partie IV — Par classe d'actifs**
11. [Actions, ADR, ETF/fonds, REIT/MLP, SPAC](#11-actions-adr-etffonds-reitmlp-spac)
12. [Fixed Income : corporates, munis, Treasuries, MBS/ABS](#12-fixed-income--corporates-munis-treasuries-mbsabs)
13. [Dérivés listés et OTC](#13-dérivés-listés-et-otc)
14. [Securities Lending et Repo](#14-securities-lending-et-repo)

**Partie V — Les opérations au quotidien**
15. [Le cycle de vie détaillé](#15-le-cycle-de-vie-détaillé)
16. [Organisation d'une équipe CA PB et journée type](#16-organisation-dune-équipe-ca-pb-et-journée-type)
17. [Fiscalité US](#17-fiscalité-us)
18. [Comptabilité : accruals et income](#18-comptabilité--accruals-et-income)

**Partie VI — Les breaks (le cœur de ton projet)**
19. [Taxonomie des breaks](#19-taxonomie-des-breaks)
20. [Clés de matching et tolérances](#20-clés-de-matching-et-tolérances)
21. [Claims management](#21-claims-management)
22. [KPIs, risques et contrôles](#22-kpis-risques-et-contrôles)

**Partie VII — Cas pratiques chiffrés**
23. [Neuf cas pratiques](#23-neuf-cas-pratiques)

**Partie VIII — Project Management**
24. [Ton double rôle PM + Dev](#24-ton-double-rôle-pm--dev)
25. [Le cycle d'un projet d'automatisation ops](#25-le-cycle-dun-projet-dautomatisation-ops)
26. [Discovery : cartographier et mesurer](#26-discovery--cartographier-et-mesurer)
27. [Travailler avec le Sponsor](#27-travailler-avec-le-sponsor)
28. [Travailler avec l'IT](#28-travailler-avec-lit)
29. [Travailler avec les Ops (SMEs)](#29-travailler-avec-les-ops-smes)
30. [Gouvernance en banque : contrôles, EUC, data, audit](#30-gouvernance-en-banque--contrôles-euc-data-audit)
31. [Templates prêts à l'emploi](#31-templates-prêts-à-lemploi)
32. [Backlog de use cases priorisé](#32-backlog-de-use-cases-priorisé)
33. [Pièges classiques](#33-pièges-classiques)

**Partie IX — Implémentation Dagster**
34. [Architecture cible](#34-architecture-cible)
35. [Modèle de données canonique](#35-modèle-de-données-canonique)
36. [Squelette de code Dagster](#36-squelette-de-code-dagster)
37. [Le moteur de matching](#37-le-moteur-de-matching)
38. [Outputs : Excel et email](#38-outputs--excel-et-email)
39. [Tests, qualité, déploiement, run](#39-tests-qualité-déploiement-run)

**Annexes**
- [A. Glossaire EN → FR](#annexe-a--glossaire-en--fr)
- [B. Champs SWIFT/ISO à connaître](#annexe-b--champs-swiftiso-à-connaître)
- [C. Checklist des questions à poser en interne](#annexe-c--checklist-des-questions-à-poser-en-interne)
- [D. Plan 30-60-90 jours](#annexe-d--plan-30-60-90-jours)
- [E. Ressources pour aller plus loin](#annexe-e--ressources-pour-aller-plus-loin)
- [F. Pont terminologique US (jargon de desk NYC)](#annexe-f--pont-terminologique-us-jargon-de-desk-nyc)

---

# PARTIE I — FONDAMENTAUX

## 1. Les Corporate Actions en une page

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-01** (onglet `P1_Fondamentaux`)

Une **Corporate Action (CA)** est un événement décidé par un **émetteur** (**issuer**) qui modifie un titre ou ce que reçoivent ses détenteurs : versement d'un dividende ou d'un coupon, split, fusion, offre de rachat, remboursement d'obligation, etc.

**Le métier CA en une phrase :** pour chaque événement, savoir **qui a droit à quoi** (**entitlement**), **le collecter** auprès du marché, **le distribuer** correctement aux clients, **au bon moment**, avec **la bonne fiscalité**, et **prouver** que tout tombe juste (**reconciliation**).

**Pourquoi c'est difficile :**

| Difficulté | Explication |
|---|---|
| Volume | Des dizaines de milliers d'événements par an rien qu'aux US, et des millions de lignes de positions. |
| Données imparfaites | Les sources (Bloomberg, DTC, agents, custodians) **se contredisent** souvent, surtout au début de la vie d'un événement. |
| Événements vivants | Un événement est annoncé, puis **modifié** (taux, dates), parfois **annulé**. Chaque mise à jour peut créer des breaks. |
| Positions mouvantes | Trades non réglés, prêts de titres, shorts, rehypothéquation : la position « qui a droit » n'est pas triviale. |
| Fiscalité | Le taux de retenue dépend du statut fiscal du client, du type de revenu et du pays. |
| Délais | Les événements volontaires ont des **deadlines** strictes ; rater une élection = perte financière directe. |
| Risque | Les CA sont historiquement **l'une des premières sources de pertes opérationnelles** dans les back-offices titres. |

---

## 2. Classification des événements

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-02** (onglet `P1_Fondamentaux`)

### 2.1 Selon la participation du détenteur (champ ISO `CAMV`)

| Catégorie | Code | Description | Exemples |
|---|---|---|---|
| **Mandatory** | `MAND` | S'applique automatiquement, aucune action requise. | Dividende cash, split, changement de nom, fusion en cash |
| **Mandatory with options** (**Elective**) | `CHOS` | Le détenteur **choisit** parmi des options ; sans réponse, l'**option par défaut** (**default option**) s'applique. | Dividende cash ou actions, fusion avec choix cash/actions |
| **Voluntary** | `VOLU` | Le détenteur **peut** participer ; sans réponse, rien ne se passe (**No Action** / `NOAC`). | Tender offer, exercice de droits, rachat, consent solicitation |

### 2.2 Selon la nature économique

| Nature | Description | Exemples |
|---|---|---|
| **Distribution / Income** | Le détenteur reçoit du cash ou des titres, et garde son titre. | Dividende, coupon, spin-off, bonus issue |
| **Reorganization (Reorg)** | Le titre est transformé ou échangé. | Split, fusion, conversion, remboursement |
| **Informational / Governance** | Pas de flux, mais une information ou un vote. | AGM (proxy), class action, changement de nom |

> **Vocabulaire US à retenir :** les équipes et les systèmes sont souvent découpés entre **Income** (ou **Dividends** / **Distributions**), **Mandatory Reorg**, **Voluntary Reorg** et **Proxy**. DTC utilise aussi cette logique.

---

## 3. Catalogue des types d'événements (codes ISO)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-03** (onglet `P1_Fondamentaux`)

Les codes viennent du champ **`CAEV`** (ISO 15022/20022). Ils seront tes clés de catégorisation dans le code.

### 3.1 Distributions / Income

| Code | Nom | Description | Ce qui se passe |
|---|---|---|---|
| **DVCA** | Cash Dividend | Dividende en espèces | Cash par action |
| **DVSE** | Stock Dividend | Dividende en actions | N nouvelles actions pour M détenues |
| **DVOP** | Dividend Option | Dividende au choix | Cash et/ou actions (événement `CHOS`) |
| **DRIP** | Dividend Reinvestment | Réinvestissement | Le cash est converti en actions |
| **BONU** | Bonus Issue | Actions gratuites | Comparable à un DVSE |
| **CAPD** | Capital Distribution | Remboursement de capital | Cash, traité fiscalement comme *return of capital* |
| **CAPG** | Capital Gains Distribution | Distribution de plus-values | Fréquent sur fonds/ETF/REIT |
| **INTR** | Interest Payment | Coupon | Cash selon nominal × taux × fraction d'année |
| **PRED** | Partial Redemption (sans réduction du nominal) / **PCAL** Partial Call | Remboursement partiel | Voir Fixed Income |
| **SOFF** | Spin-off | Scission | Actions d'une nouvelle entité |
| **DVOP / SHPR** | Shares Premium Dividend | Variante | — |
| **RHDI** | Intermediate Securities Distribution | Distribution de droits | Les droits arrivent sur le compte |
| **DECR / INCR** | Decrease/Increase in Value | Réduction/augmentation du nominal | Variation du nominal |

### 3.2 Réorganisations obligatoires

| Code | Nom | Description |
|---|---|---|
| **SPLF** | Stock Split (Forward) | 1 action → N actions |
| **SPLR** | Reverse Split | N actions → 1 action (fractions fréquentes) |
| **MRGR** | Merger | Fusion : échange contre cash et/ou titres de l'acquéreur |
| **NAME** | Name Change | Nouveau nom (souvent nouveau CUSIP/ticker) |
| **CHAN** | Change | Changement de caractéristiques (ex. nouveau CUSIP) |
| **CONV** | Conversion | Obligation/préférentielle → actions (peut être volontaire) |
| **EXOF** | Exchange | Échange de titres (peut être volontaire) |
| **REDM** | Final Maturity | Remboursement à l'échéance |
| **MCAL** | Full Call | Remboursement anticipé total |
| **PCAL** | Partial Call | Remboursement anticipé partiel (avec réduction du nominal) |
| **PRED** | Partial Redemption without reduction of nominal | Amortissement via facteur (MBS/ABS) |
| **LIQU** | Liquidation | Distribution de liquidation |
| **BRUP** | Bankruptcy | Faillite |
| **DLST** | Delisting | Radiation de la cote |
| **WRTH** | Worthless | Titre déclaré sans valeur |
| **PINK** | Payment in Kind | Coupon payé en titres |

### 3.3 Événements volontaires

| Code | Nom | Description |
|---|---|---|
| **TEND** | Tender Offer | Offre publique d'achat (cash et/ou titres) |
| **BIDS** | Repurchase Offer | Rachat par l'émetteur (buyback / Dutch auction) |
| **EXRI** | Call on Intermediate Securities | Exercice des droits |
| **PRIO** | Priority Issue | Souscription prioritaire |
| **CONS** | Consent | Consent solicitation (vote des porteurs obligataires sur des modifications de contrat) |
| **BPUT** | Put Redemption | Le porteur demande le remboursement anticipé |
| **EXWA** | Warrant Exercise | Exercice de warrants |
| **ODLT** | Odd Lot Sale/Purchase | Offre aux détenteurs de « rompus » |
| **DSCL** | Disclosure | Demande d'identification des porteurs |

### 3.4 Gouvernance, information, litiges

| Code | Nom | Description |
|---|---|---|
| **MEET / XMET / OMET** | Meetings | Assemblées générales (proxy voting) |
| **CLSA** | Class Action | Action collective : les porteurs sur une période peuvent réclamer |
| **INFO** | Information | Pur informatif |
| **ACTV / SUSP** | Trading Status | Reprise ou suspension de cotation |

### 3.5 Options d'un événement (champ `CAOP`)

| Code | Signification |
|---|---|
| **CASH** | Recevoir du cash |
| **SECU** | Recevoir des titres |
| **CASE** | Cash + titres |
| **NOAC** | Ne rien faire |
| **LAPS** | Laisser expirer (droits) |
| **EXER** | Exercer |
| **SLLE** | Vendre (ex. vendre ses droits) |
| **BUYA** | Acheter des droits supplémentaires |
| **OVER** | Sur-souscrire (**oversubscription**) |
| **CONY / CONN** | Consentir / ne pas consentir |
| **MPUT** | Exercer un put |
| **PRUN** | Prorata (**proration**) |

---

## 4. Les dates clés et leurs spécificités US

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-04** (onglet `P1_Fondamentaux`)

### 4.1 Les dates (champs ISO entre parenthèses)

| Date | Champ | Définition | Pourquoi c'est critique |
|---|---|---|---|
| **Announcement Date** | `ANOU` | Annonce publique | Démarre le traitement |
| **Declaration Date** | — | Décision formelle du board (vocabulaire US) | Souvent égale à l'annonce |
| **Ex-Date** | `XDTE` | Premier jour de cotation **sans** le droit | Le prix chute ; c'est la date de bascule de l'accrual |
| **Record Date** | `RDTE` | Date à laquelle on photographie les détenteurs inscrits | Détermine l'entitlement côté marché |
| **Payable / Pay Date** | `PAYD` | Date de paiement | Date du cash/stock réel → réconciliation |
| **Effective Date** | `EFFD` | Date d'effet d'une réorg | Changement de CUSIP, de quantité |
| **Market Deadline** | `MKDT` | Deadline de l'agent | Hard stop du marché |
| **Response Deadline** | `RDDT` | Deadline du custodian/de la DTC | Plus tôt que la market deadline |
| **Client Deadline** | (interne) | Deadline imposée aux clients PB | Encore plus tôt, pour avoir le temps de traiter |
| **Protect Deadline** | `PODT` | Date limite pour livrer les titres après une *guaranteed delivery* | Spécifique aux tenders US |
| **Expiration Date** | `EXPI` | Fin de l'offre | Peut être **prolongée** (*extended*) |
| **Proration Date** | — | Calcul du prorata si l'offre est sursouscrite | Résultat ≠ instruction |

### 4.2 Règle d'or : ex-date vs record date aux US

Depuis **mai 2024**, les actions US se règlent à **T+1**.

- Pour un dividende normal : **Ex-Date = Record Date**.
- Si tu achètes la veille de l'ex-date (T), ton trade se règle à T+1 = record date. Tu es donc inscrit et tu as droit au dividende.
- Si tu achètes le jour de l'ex-date, tu te règles après la record date : pas de dividende.

> **Attention :** les autres marchés ont des cycles différents (T+2 en Europe à date, avec une migration vers T+1 prévue). Pour des titres non-US détenus par des clients US, ex-date et record date peuvent différer.

### 4.3 Qui fixe l'ex-date aux US ?

- Pour les titres **listés** : l'**exchange** (NYSE, Nasdaq…), pas l'émetteur.
- Pour les titres **OTC** : **FINRA** (règles de l'*Uniform Practice Code*).
- Les émetteurs doivent notifier les distributions à l'avance (règle SEC 10b-17 pour les titres non listés ; règles de cotation pour les exchanges). **Conséquence :** il y a parfois un délai entre l'annonce de l'émetteur et la fixation de l'ex-date, d'où des données incomplètes au début.

### 4.4 Les **Due Bills** (spécificité US, source n°1 de breaks sur les gros événements)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-05** (onglet `P1_Fondamentaux`)

Pour les distributions **égales ou supérieures à 25 %** de la valeur du titre (gros split, gros dividende exceptionnel, certains spin-offs), l'ex-date est fixée **le premier jour ouvré après la pay date**.

Conséquences :

- Entre la record date et l'ex-date, le titre se traite **encore « avec » la distribution** (*cum*).
- Le vendeur, inscrit à la record date, reçoit la distribution, mais il **la doit à l'acheteur** : c'est le **due bill**.
- DTC gère une partie de ces ajustements via l'**interim accounting**.
- **Pour toi :** tout entitlement calculé uniquement « position à la record date » sera **faux** pour les trades de la fenêtre due bill. Il faut une règle dédiée.

### 4.5 Exemple de calendrier (dividende classique)

```
Lun 03/11  Announcement   Apple déclare 0.25 $ par action
Lun 10/11  Ex-date = Record date
Jeu 13/11  Pay date       Cash crédité via DTC
Ven 14/11  Recon          Expected vs Received → breaks éventuels
```

---

# PARTIE II — L'ÉCOSYSTÈME US

## 5. Les acteurs du marché US

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-06** (onglet `P2_Ecosysteme`)

### 5.1 Infrastructure de marché

| Acteur | Rôle en CA |
|---|---|
| **DTCC** (*Depository Trust & Clearing Corporation*) | Holding de l'infrastructure post-trade US. |
| **DTC** (*Depository Trust Company*) | **Le CSD US** pour actions, corporate bonds, munis. Il collecte les paiements des agents et les distribue à ses **participants** (les brokers, dont ta banque). Il publie les annonces CA, gère les élections volontaires et les paiements. |
| **Cede & Co.** | **Nominee** de la DTC : c'est lui qui figure dans les registres des émetteurs comme détenteur légal des titres en DTC. |
| **NSCC** | Chambre de compensation des trades actions US. Elle gère aussi les ajustements CA sur les trades en cours (transformations). |
| **Fedwire Securities Service** (Federal Reserve) | Dépositaire des **US Treasuries** et des **agency MBS** (Fannie Mae, Freddie Mac, Ginnie Mae). Ce n'est **pas** la DTC. |
| **OCC** (*Options Clearing Corporation*) | Chambre de compensation des **options listées**. Elle publie les **ajustements de contrats** après les CA. |
| **Exchanges** (NYSE, Nasdaq, Cboe) | Fixent les ex-dates des titres listés. |
| **FINRA** | Autorégulateur des brokers ; ex-dates OTC ; règles de pratique. |
| **SEC** | Régulateur (règles de protection client, de notification, etc.). |
| **MSRB / EMMA** | Source officielle d'information pour les **municipal bonds**. |

### 5.2 Côté émetteur

| Acteur | Rôle |
|---|---|
| **Issuer** | Décide l'événement |
| **Transfer Agent** (Computershare, Equiniti, Broadridge…) | Tient le registre des actionnaires (très central aux US) |
| **Paying Agent** | Verse le cash (pour les obligations, c'est souvent le **trustee**) |
| **Trustee** | Représente les porteurs obligataires (BNY, US Bank, Wilmington…) |
| **Exchange / Tender / Information Agent** | Gère les offres, collecte les élections |
| **Depositary Bank** (BNY, Citi, JPMorgan, Deutsche Bank) | Émet les **ADR** et gère les CA du titre sous-jacent étranger |

### 5.3 Côté investisseur (le monde d'un Prime Broker)

| Acteur | Rôle |
|---|---|
| **Prime Broker** (ta banque) | Garde, finance et prête des titres aux hedge funds ; il est le **participant DTC** |
| **Clients** (hedge funds) | Détenteurs économiques ; ils font les élections |
| **Fund Administrators** (Citco, SS&C GlobeOp, State Street, etc.) | Tiennent la comptabilité des fonds. **Ils réconcilient eux aussi**, et ce sont eux qui remontent souvent des breaks au PB. |
| **Executing brokers** | Brokers qui exécutent les trades des clients, réglés chez le PB (*give-ups*) |
| **Agent lenders / Street lenders** | Contreparties de prêt de titres |
| **Sub-custodians** | Pour les titres non-US détenus par les clients |
| **Data vendors** | Bloomberg, LSEG, ICE, S&P, SIX… |

---

## 6. Les sources de données : Bloomberg, Broadridge, DTC…

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-07** (onglet `P2_Ecosysteme`)

### 6.1 Panorama

| Source | Ce qu'elle apporte | Forces | Faiblesses |
|---|---|---|---|
| **DTC** (flux CA, souvent ISO 20022, + portail **CA Web**) | Annonces « officielles » des événements sur titres en DTC, entitlements, paiements, élections | Source de règlement réelle : **c'est ce qui sera payé** | Couverture limitée aux titres DTC ; formats techniques |
| **Bloomberg** (terminal : `CACS <GO>`, `DVD <GO>` ; **Bloomberg Data License** en fichiers bulk via SFTP) | Annonces CA mondiales, historique, identifiant **Bloomberg Action ID** | Couverture large, rapide | Pas une source de règlement ; peut différer de la DTC |
| **Broadridge** | Très présent aux US : plateformes de back-office broker-dealer (par exemple **Impact**), **proxy voting** (Broadridge distribue la grande majorité des documents de vote aux US, outil **ProxyEdge**), services CA | Intégré au back-office | Il faut savoir **quel produit Broadridge** est utilisé en interne **[À VÉRIFIER EN INTERNE]** |
| **Custodians / sub-custodians** (MT564/566) | Annonces et paiements pour les titres hors DTC | Source de règlement locale | Formats variables |
| **Agents / Trustees** | Documents officiels (prospectus, offer docs) | Source primaire | Non structuré (PDF) |
| **EMMA (MSRB)** | Munis | Officiel | — |
| **Agences MBS** | Facteurs mensuels | Officiel | — |
| **OCC** | Info Memos d'ajustement d'options | Officiel | Non structuré |

### 6.2 La **Golden Copy**

Le processus de **scrubbing** consiste à comparer les sources, à résoudre les conflits et à publier une version validée de chaque événement : la **golden copy** (ou *validated event*). Les systèmes CA du marché proposent des règles de type « si Bloomberg et DTC concordent, l'événement est auto-validé ; sinon, revue manuelle ».

**Champs typiquement comparés :** type d'événement, ISIN/CUSIP, dates (ex, record, pay), taux brut, devise, ratio, options, option par défaut, deadlines, statut (confirmé/préliminaire/annulé).

> **Lien avec ton projet :** l'**Event Reconciliation** (vendor vs vendor vs système interne) est probablement **le premier use case** le plus simple à automatiser et à fort impact.

### 6.3 Identifiants à maîtriser

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-08** (onglet `P2_Ecosysteme`)

| Identifiant | Description |
|---|---|
| **CUSIP** | 9 caractères, **identifiant US** principal |
| **ISIN** | 12 caractères ; pour les US : `US` + CUSIP + chiffre de contrôle |
| **SEDOL** | Identifiant UK |
| **Ticker / Exchange code** | Instable : change avec les CA |
| **FIGI** (Bloomberg) | Identifiant ouvert Bloomberg |
| **COAF** | *Official Corporate Action Event Reference* : identifiant unique « officiel » de l'événement |
| **CORP** | Référence de l'événement chez le fournisseur (DTC, custodian…) |
| **Bloomberg Action ID** | Identifiant d'événement Bloomberg |
| **Internal Event ID** | Référence dans le système CA interne **[À VÉRIFIER EN INTERNE]** |

> **Piège :** après un `NAME`/`CHAN`/`MRGR`, **le CUSIP change**. Ton matching doit gérer un **mapping ancien → nouveau CUSIP**.

---

## 7. Messages et standards (SWIFT MT, ISO 20022)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-09** (onglet `P2_Ecosysteme`)

### 7.1 Les messages

| ISO 15022 | ISO 20022 | Sens | Usage |
|---|---|---|---|
| **MT564** | seev.031 (notification), seev.035 (movement preliminary advice), seev.044 (cancellation) | Custodian → Client | Annonce, mise à jour, préavis de mouvement |
| **MT565** | seev.033 | Client → Custodian | Instruction d'élection |
| **MT566** | seev.036 (confirmation), seev.037 (reversal) | Custodian → Client | Confirmation de paiement/mouvement |
| **MT567** | seev.032 / seev.034 | Custodian → Client | Statut d'une instruction ou d'un événement |
| **MT568** | seev.039 / seev.040 | Bidirectionnel | Narratif libre |

### 7.2 Fonctions d'un MT564 (champ `23G`)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-10** (onglet `P2_Ecosysteme`)

- **NEWM** : nouvelle annonce
- **REPL** : remplacement (mise à jour)
- **REPE** : *eligible balance* / préavis de paiement (avec position et montants)
- **CANC** : annulation
- **WITH** : retrait de l'événement

Plus le statut de complétude : **PREU** (préliminaire, non confirmé) vs **COMP** (complet).

> **Pour ton pipeline :** un événement est une **série de versions**. Il faut **historiser** (event versioning) et toujours comparer sur **la dernière version valide**.

### 7.3 Organismes de normalisation

- **SMPG** (*Securities Market Practice Group*) : règles d'usage des messages CA au niveau mondial.
- **ISITC** : équivalent US des market practices.

---

# PARTIE III — LE PRIME BROKERAGE

## 8. C'est quoi un Prime Broker

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-11** (onglet `P3_PB`)

Un **Prime Broker (PB)** est la banque « tout-en-un » d'un **hedge fund**. Ses services :

| Service | Description |
|---|---|
| **Custody / Clearing / Settlement** | Garde des actifs du fonds, règlement de ses trades |
| **Financing (Margin lending)** | Prêt de cash au fonds pour acheter avec effet de levier |
| **Securities Lending (Stock borrow)** | Prêt de titres au fonds pour qu'il puisse **vendre à découvert** (*short selling*) |
| **Synthetic Prime / Swaps** | Exposition via **portfolio swaps / total return swaps (TRS)** au lieu de titres physiques |
| **Capital Introduction** | Mise en relation avec des investisseurs |
| **Reporting** | Positions, P&L, marges, **CA notifications** |
| **Corporate Actions services** | Notification, élection, paiement des événements |

**Clients :** hedge funds de stratégies variées. Les stratégies **event-driven / merger arbitrage** sont **extrêmement sensibles aux CA** : elles vivent des fusions et des offres, donc les élections volontaires sont critiques.

**Revenus du PB :** spread sur le financement, frais de prêt de titres (*borrow fees*), commissions, et **réutilisation des titres clients** (rehypothéquation) pour se financer ou les prêter. Ce dernier point a un impact majeur sur les CA (voir §10).

---

## 9. Comment sont structurées les positions d'un PB

### 9.1 Types de comptes clients (vocabulaire US)

| Compte | Description |
|---|---|
| **Cash account** | Titres payés intégralement |
| **Margin account** | Titres achetés à crédit (le client a un **debit balance**) |
| **Short account** | Positions vendues à découvert |
| **Fully paid vs Margin securities** | Les titres « fully paid » ou **excess margin** doivent être protégés ; les autres peuvent être réutilisés |

### 9.2 La **rehypothéquation** et la **SEC Rule 15c3-3**

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-12** (onglet `P3_PB`)

- La **Customer Protection Rule (15c3-3)** impose au broker de garder en **possession or control** les titres **fully paid** et **excess margin** de ses clients (dans des comptes ségrégés : **box**, **seg**).
- En revanche, le PB peut **réutiliser** (rehypothéquer) les titres clients en marge, dans la limite d'un plafond (classiquement **140 % du debit balance** du client).
- Ces titres réutilisés peuvent être **prêtés** à d'autres (*street lending*) ou mis en **repo**.

**Conséquence CA majeure :** à la record date, une partie des titres « du client » n'est **pas physiquement chez le PB** ; ils sont chez un emprunteur. Le marché (DTC) verse le dividende à **l'emprunteur**. Le PB doit quand même créditer son client : c'est un **substitute payment** / **payment in lieu of dividend** (voir §10.1).

### 9.3 Les « couches » de position

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-13** (onglet `P3_PB`)

Pour un même CUSIP à la record date, le PB doit consolider :

```
Positions clients (longs)             +  1 000 000
Positions clients (shorts)            -    300 000
Titres empruntés à la rue (borrows)   +    300 000   (pour couvrir les shorts)
Titres prêtés à la rue (loans)        -    200 000   (rehypo / lending)
Positions propres de la banque        ±  …
Trades non réglés (fails, pending)    ±  …
────────────────────────────────────────────────────
= Position attendue chez DTC (depot)  =    800 000
```

**La position DTC doit « tomber juste » avec la somme des positions internes.** Si ce n'est pas le cas : **position break**, donc **entitlement break** au paiement.

### 9.4 Le **stock record**

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-14** (onglet `P3_PB`)

C'est le grand livre des titres du broker-dealer : pour chaque titre, la somme des **ownership positions** (clients, firme) doit égaler la somme des **location positions** (DTC, Fed, custodians, prêtés, en transit). La recon du stock record est la base de tout. Un break de stock record à la record date devient un break CA.

---

## 10. Ce qui rend les CA plus complexes en PB

### 10.1 Dividendes et **substitute payments**

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-15** (onglet `P3_PB`)

- Client long, titres **non prêtés** → le client reçoit un **vrai dividende** (*qualified dividend* potentiel pour un contribuable US).
- Client long, titres **rehypothéqués/prêtés** → l'emprunteur paie un **manufactured dividend** au PB, et le PB crédite le client d'un **substitute payment in lieu of dividend**.
  - **Fiscalement différent** pour un contribuable US : le substitute payment est généralement imposé comme revenu ordinaire et reporté sur le **1099-MISC** (et non sur le 1099-DIV).
  - Pour les clients non-US : retenue à la source applicable sur le substitute payment.
  - Le PB doit **allouer** quels clients ont reçu un vrai dividende et lesquels un substitute payment (méthodologie d'allocation **[À VÉRIFIER EN INTERNE]**).

### 10.2 Shorts : le client **doit** le dividende

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-16** (onglet `P3_PB`)

- Un client **short** à la record date doit payer le dividende au prêteur des titres.
- Le PB **débite** le compte short du client (souvent à la pay date, avec un accrual à l'ex-date) et paie le prêteur (*street lender*).
- **Breaks fréquents :** montant débité ≠ montant dû au prêteur, taux appliqué incorrect, shorts couverts entre l'ex-date et la record date…

### 10.3 Élections volontaires avec titres prêtés

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-17** (onglet `P3_PB`)

C'est **le** point sensible du PB. Exemple : un fonds de merger arb veut **apporter** (*tender*) 500 000 actions à une OPA, mais 300 000 sont rehypothéquées et prêtées.

Solutions :

1. **Recall** : rappeler les titres prêtés à temps.
2. **Guaranteed Delivery / Protect** : sur certaines offres US, on peut instruire avec une **Notice of Guaranteed Delivery**, puis livrer les titres plus tard (avant la **protect deadline**). Si on ne livre pas : **non-delivery** → perte potentielle.
3. **Buy-in** : racheter les titres sur le marché.
4. **Liability claim** : l'emprunteur reste redevable du résultat économique de l'élection (**voluntary CA liability**, règles des contrats de prêt).

**Breaks typiques :** élection client > position disponible ; protect non couvert ; résultat de proration ≠ attendu.

### 10.4 Le **Synthetic Prime** (swaps)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-27** (onglet `P3bis_PB_Profondeur`)

- Le client a une exposition **synthétique** via un **portfolio swap** ; c'est le PB qui détient le hedge physique.
- Les CA sont **répliquées contractuellement** sur le swap : **dividend pass-through** (souvent un pourcentage du brut défini dans la confirmation), ajustement de quantité sur les splits, traitement des fusions selon les **ISDA Equity Definitions** (*Merger Events*, *Tender Offers*, *Potential Adjustment Events*).
- **Fiscalité 871(m)** : pour les clients non-US, les **dividend equivalents** sur des dérivés « delta-one » sur actions US sont soumis à retenue à la source (voir §17).
- **Breaks typiques :** dividende payé sur le swap ≠ dividende reçu sur le hedge (*hedge vs swap*), mauvais pourcentage de pass-through, ajustement du notionnel oublié.

### 10.5 Les **Fund Administrators**

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-18** (onglet `P3_PB`)

Les fonds ont leur propre administrateur, qui réconcilie ses livres avec les **relevés du PB**. Une part non négligeable du travail des ops CA consiste à **répondre aux queries des admins et des clients** : « Pourquoi ai-je reçu 21 250 $ au lieu de 25 000 $ ? ». D'où l'intérêt d'avoir des breaks **déjà identifiés et expliqués** avant que le client n'appelle.

### 10.6 **Proxy voting**

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-19** (onglet `P3_PB`)

- Le droit de vote suit le titre : les titres **prêtés** ne votent pas chez le client.
- Aux US, la distribution des documents et la collecte des votes passent très majoritairement par **Broadridge** (ProxyEdge).
- Un client qui veut voter sur des titres prêtés doit demander un **recall** avant la record date de l'assemblée.

### 10.7 Notifications clients

Le PB doit **notifier** ses clients de chaque événement qui les concerne (portail, email, SWIFT, fichiers). Les erreurs ou retards de notification sur un événement volontaire = **risque de réclamation client**.

---

# PARTIE III-bis — LE PRIME BROKERAGE EN PROFONDEUR

> Cette partie complète les §8 à §10. L'objectif est de comprendre **comment fonctionne un PB de l'intérieur**, puis de voir **comment chaque Corporate Action se propage dans chacune de ses fonctions**.

## 10A. Le business PB et ses clients

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-20** (onglet `P3bis_PB_Profondeur`)

### Pourquoi le Prime Brokerage existe

Un hedge fund traite avec de nombreux **executing brokers** (pour trouver la meilleure exécution), mais il a besoin d'**un point central** qui :

1. **garde** tous ses actifs au même endroit (custody) ;
2. **règle** tous ses trades, quel que soit le broker exécutant (**give-up** / **clearing**) ;
3. lui **prête du cash** (levier) et **des titres** (pour vendre à découvert) ;
4. calcule **une marge consolidée** sur l'ensemble de son portefeuille (les longs compensent partiellement les shorts) ;
5. lui fournit un **reporting consolidé** (positions, P&L, financement, CA).

Ce point central, c'est le **Prime Broker**. Le modèle date des années 1980 aux US et s'est généralisé avec l'essor des hedge funds.

**Multi-prime :** depuis la crise de 2008 (faillite de Lehman, où des fonds ont vu leurs actifs gelés), la plupart des grands fonds utilisent **plusieurs PB**. Les conséquences pour les CA sont les suivantes :
- un même fonds a des positions sur le même titre **chez plusieurs PB** ;
- il doit envoyer ses élections **à chacun** ;
- c'est le **fund administrator** qui consolide, et il compare ce que chaque PB a payé.

**Ta banque en PB :** beaucoup de plateformes PB actuelles sont issues de rachats ou de fusions d'activités. Aux US, l'activité broker-dealer passe par une entité régulée SEC/FINRA **[À VÉRIFIER EN INTERNE : entités, plateformes héritées, systèmes]**. Si des clients et des systèmes hérités d'une migration coexistent encore, cela peut expliquer **une partie des écarts entre systèmes**.

### Les clients et leur sensibilité aux CA

| Stratégie du fonds | Ce qu'elle fait | Sensibilité aux CA | Pourquoi |
|---|---|---|---|
| **Long/Short Equity** | Achète des actions sous-évaluées, vend à découvert les surévaluées | **Élevée** | Beaucoup de dividendes sur les longs **et** sur les shorts, splits, spin-offs |
| **Market Neutral / Stat Arb / Quant** | Milliers de positions longues et courtes, faible net | **Élevée (volume)** | Énormément de petits événements. Les erreurs sont nombreuses et diffuses |
| **Event-Driven / Merger Arbitrage** | Parie sur la réalisation des fusions et des offres | **Critique** | Le P&L du fonds **est** l'événement : tenders, élections, prorations, protects |
| **Special Situations / Distressed** | Investit dans des sociétés en difficulté | **Critique** | Restructurations, faillites, échanges de dette, consent solicitations |
| **Convertible Arbitrage** | Long convertible / short action | **Élevée** | Ajustements des ratios de conversion, dividendes sur le short, calls |
| **Activist** | Prend des participations pour influencer la gouvernance | **Élevée (proxy)** | Votes, recalls de titres prêtés pour voter |
| **SPAC Arbitrage** | Achète des SPAC sous le trust value | **Élevée** | Redemption rights, unit splits, de-SPAC |
| **Global Macro** | Taux, devises, indices | Faible en titres | Surtout futures et dérivés |
| **Credit Long/Short** | Obligations et CDS | **Moyenne/Élevée** | Coupons, calls, tenders de dette, défauts |
| **Family offices / Asset managers** | Gestion plus classique | Moyenne | Dividendes, coupons, proxy |

> **À retenir :** en PB, les clients sont **sophistiqués**. Ils suivent leurs événements de très près, et un fonds de merger arb **connaît souvent l'événement mieux que les ops**. Une erreur se voit immédiatement, et l'attente de qualité est très élevée.

---

## 10B. Le modèle économique

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-21** (onglet `P3bis_PB_Profondeur`)

| Source de revenu | Mécanique | Lien avec les CA |
|---|---|---|
| **Margin financing** (financement des longs) | Le client paie un taux sur son **debit balance** (taux de référence USD comme le SOFR + spread) | Les dividendes reçus **réduisent** le debit balance. Un dividende non crédité à temps = intérêts facturés à tort au client |
| **Short financing / Short rebate** | Le cash de la vente à découvert reste chez le PB ; le client reçoit un **rebate** (intérêt) diminué du **borrow fee** | Le dividende débité sur les shorts **réduit** le crédit du client |
| **Stock loan fees** | Le PB emprunte des titres à la rue (ou utilise l'inventaire client) et les prête au client short. Il gagne le spread. Titres faciles = **General Collateral (GC)**, rares = **Hard-to-Borrow (HTB) / Specials** | **Les CA rendent les titres rares** : avant une fusion, un vote ou un rights issue, les prêteurs rappellent et les fees explosent |
| **Rehypothecation / Internalisation** | Le PB réutilise les titres des clients longs pour couvrir les shorts d'autres clients (**internalisation**) ou les prête à la rue | C'est ce qui crée les **substitute payments** et les problèmes d'**élections sur titres prêtés** |
| **Synthetic financing** | Spread de financement intégré au swap | Dividendes **pass-through**, 871(m), ajustements |
| **Commissions / Execution** | Sur l'exécution des ordres | Indirect |
| **Services** (Cap Intro, reporting, consulting) | Fidélisation | Indirect |

**Coût principal :** le **bilan** (*balance sheet*) et le **capital réglementaire**. Un PB veut maximiser le revenu par dollar de bilan. C'est pour cela que le **synthétique** (swaps) s'est beaucoup développé : il consomme souvent moins de bilan que le **cash prime** (titres physiques).

> **Lien business → ton projet :** chaque break CA non résolu peut avoir un **coût direct** (perte), un **coût de financement** (cash qui n'est pas là, intérêts mal facturés), un **coût en capital** (voir §10J) et un **coût relationnel** (client mécontent qui transfère ses actifs chez un autre PB).

---

## 10C. Organisation front-to-back d'un PB

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-22** (onglet `P3bis_PB_Profondeur`)

| Fonction | Rôle | Interaction avec les CA |
|---|---|---|
| **Sales / Client Coverage** | Relation commerciale avec les fonds | Escalade des plaintes clients sur les CA ; pression sur la qualité |
| **Client Onboarding / KYC** | Ouverture des comptes, documentation légale et fiscale | **Statut fiscal** (W-8/W-9), options par défaut, canaux de notification → données d'entrée des CA |
| **Legal / Documentation** | **Prime Brokerage Agreement**, **ISDA** (swaps), **MSLA / GMSLA** (prêt de titres), **Customer Account Agreement** | Définit **contractuellement** le traitement des CA (pass-through, élections, deadlines, rehypothéquation) |
| **Margin & Credit Risk** | Calcul quotidien des marges, appels de marge, limites | Les CA modifient les prix, les quantités et les identifiants → risque de **faux appels de marge** |
| **Equity Finance / Securities Lending desk** | Gère les emprunts et prêts de titres, les locates, les fees | Recalls, manufactured payments, élections sur titres prêtés, transformations des prêts |
| **Synthetic / Delta One desk** | Gère les swaps clients et leurs hedges | Pass-through, ajustements de notionnel, Extraordinary Events, 871(m) |
| **Treasury / Funding** | Finance le bilan du PB | Gros flux cash CA (dividendes spéciaux, fusions cash) |
| **Collateral Management** | Gère le collatéral reçu et donné (tri-party, CCP, contreparties) | Titres en collatéral touchés par une CA → **substitution** |
| **Operations** : Clearing & Settlement, **Corporate Actions**, Cash management, Reconciliations, Stock record | Exécution post-trade | **Ton périmètre** |
| **Client Service** | Point de contact quotidien des fonds et des admins | Reçoit les **queries CA** |
| **Tax** | Retenues, reporting, statuts QI/QDD | Taux applicables, substitute payments, 871(m) |
| **Product Control / Finance** | P&L, comptabilité | Accruals, write-offs |
| **Regulatory Reporting / Compliance** | 15c3-3, 15c3-1, Reg SHO… | Breaks anciens → **charges de capital**, conformité 14e-4 |
| **Technology (ITO)** | Systèmes et automatisation | **Toi** |

---

## 10D. Le cycle quotidien d'un trade en PB

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-23** (onglet `P3bis_PB_Profondeur`)

```
J (trade date)
 1. Le fonds exécute chez un Executing Broker (EB) : "achète 100 000 XYZ"
 2. L'EB "give-up" le trade au PB → le PB affirme/accepte le trade
 3. Si c'est une vente à découvert : le fonds a dû obtenir un LOCATE (Reg SHO)
    auprès du PB AVANT l'ordre → le desk Equity Finance confirme la disponibilité
 4. Le trade entre en compensation (NSCC) → règlement en DTC à J+1

Soir J
 5. Calcul de marge sur le portefeuille complet (prix de clôture)
 6. Accrual du financement (intérêts débiteurs/créditeurs, borrow fees)
 7. Accruals de revenus : DIVIDENDES À RECEVOIR / À PAYER pour les titres dont l'ex-date est J+1   ← CA
 8. Reporting au client et à son administrateur (fichiers de positions, cash, accruals, CA)

J+1 (settlement)
 9. Livraison/réception en DTC ; emprunt de titres pour couvrir le short
10. Fails éventuels → gérés par le settlement ; si fail sur une record date → CLAIM   ← CA
11. Le fund admin réconcilie ses livres avec les fichiers PB → queries si écart     ← CA
```

> **Pourquoi c'est important pour toi :** les fichiers envoyés chaque nuit au client et à son administrateur contiennent **les accruals CA**. Si ton pipeline détecte un break **avant** l'envoi de ces fichiers, l'erreur est corrigée avant que le client ne la voie.

---

## 10E. La marge et le risque

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-24** (onglet `P3bis_PB_Profondeur`)

### Les régimes de marge US (simplifié)

| Régime | Principe |
|---|---|
| **Reg T** (Federal Reserve) | Marge **initiale** classique : 50 % pour l'achat d'actions |
| **FINRA Rule 4210** | Marge de **maintenance** minimale (25 % sur les longs, davantage sur les shorts) |
| **Portfolio Margin** | Marge **basée sur le risque** du portefeuille global (scénarios de stress), plus favorable aux portefeuilles couverts |
| **House margin** | Règles internes du PB, plus strictes : **concentration**, **liquidité**, titres illiquides ou en difficulté |

### Comment les CA perturbent la marge

| Événement | Ce qui se passe | Risque si mal traité |
|---|---|---|
| **Dividende (ex-date)** | Le prix baisse du montant du dividende ; en compensation, un **dividend receivable** est créé | Si l'accrual n'est pas booké : la valeur du portefeuille long **baisse artificiellement** → **faux appel de marge**. Pour un short, l'inverse : l'exposition baisse mais la dette du dividende n'apparaît pas → **risque sous-estimé** |
| **Split / Reverse split** | Le prix et la quantité sont ajustés **simultanément** | Si le prix est ajusté mais pas la quantité (ou l'inverse) : exposition **multipliée ou divisée par N** → appel de marge aberrant ou risque masqué |
| **Spin-off** | Une nouvelle action arrive, parfois **sans prix** les premiers jours (*when-issued*) | Nouvelle position valorisée à **zéro** → faux appel de marge ; ou position non reconnue par le moteur de risque |
| **Fusion en titres** | Les actions de A sont remplacées par celles de B | Si le CUSIP B n'est pas créé dans le référentiel : position **orpheline** non margée |
| **Fusion en cash** | Les titres sont remplacés par du cash | Collatéral transformé ; l'**écart** entre la date d'effet et la date de paiement crée une exposition temporaire |
| **Tender (titres apportés)** | Les titres apportés sont **bloqués** (*tendered, in contra*) jusqu'au résultat | Ils ne peuvent plus être vendus, prêtés ou utilisés comme collatéral **librement** → impact de liquidité et de marge |
| **Rights issue** | Les droits ont une valeur et une **date d'expiration** | Droits non valorisés ou expirés à tort = perte de valeur du collatéral |
| **Delisting / Bankruptcy** | Le titre devient illiquide, son prix s'effondre | Le collatéral perd sa valeur ; **haircut 100 %** ; appels de marge massifs |
| **Gros dividende spécial** | Forte baisse du prix | Ajustement des options (OCC) et des dérivés ; si oublié, faux risque |

> **Idée de contrôle automatisable :** chaque jour d'ex-date ou d'effective date, vérifier que les positions ajustées (quantité, prix, CUSIP, accrual) sont **cohérentes entre le système CA, le système de marge et le stock record** (voir §10N).

---

## 10F. Equity Finance et Stock Loan

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-25** (onglet `P3bis_PB_Profondeur`)

### Les mécanismes

| Concept | Description |
|---|---|
| **Locate** | Avant une vente à découvert, le client doit obtenir du PB la confirmation qu'un titre peut être emprunté (**Reg SHO**, Rule 203) |
| **Sources de titres** | 1) **Inventaire client** (titres longs rehypothéquables des autres clients : **internalisation**) ; 2) **Inventaire firme** ; 3) **Street borrow** auprès des **agent lenders** (custodians qui prêtent les titres de fonds de pension, d'assureurs…) |
| **GC vs Specials / HTB** | Titre facile à emprunter (fee faible) vs rare (fee élevé, parfois plusieurs dizaines de % par an) |
| **Recall** | Le prêteur rappelle ses titres ; le PB doit les remplacer ou forcer le client à racheter (**buy-in**) |
| **Close-out (Reg SHO Rule 204)** | Obligation de couvrir les **fails to deliver** dans des délais stricts |
| **Rebate / Fee** | Rémunération des flux de cash collatéral et des titres prêtés |
| **Collatéral du prêt** | Le PB donne du cash ou des titres au prêteur (généralement 102 % pour l'USD), avec un **mark-to-market quotidien** |

### Comment les CA impactent l'Equity Finance

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-26** (onglet `P3bis_PB_Profondeur`)

| Situation | Impact |
|---|---|
| **Record date de dividende** | Les manufactured payments sont **dus** aux prêteurs. Certains prêteurs **rappellent** leurs titres avant la record date (raisons fiscales ou de politique interne) → recalls à gérer |
| **Assemblée générale (proxy)** | Les prêteurs (et les clients longs) qui veulent **voter** rappellent les titres → pénurie temporaire, hausse des fees |
| **Fusion / OPA annoncée** | Les fonds de merger arb **vendent à découvert l'acquéreur** → forte demande d'emprunt → le titre devient **HTB**. Les prêteurs rappellent pour participer à l'offre |
| **Événement volontaire sur titres prêtés** | Le prêteur instruit l'emprunteur, qui doit lui livrer le **résultat économique** de son choix. Si le PB ne peut pas livrer → **liability / claim** |
| **Rights issue** | Demande d'emprunt pour des arbitrages droits/actions ; ensuite, les prêts doivent être ajustés (droits manufacturés) |
| **Split** | La **quantité prêtée** doit être multipliée, et le **prix du collatéral** ajusté. Si le prêt n'est pas ajusté → break de prêt et de collatéral |
| **Fusion en titres** | Le prêt de A devient un prêt de B (**transformation du prêt**), avec un nouveau CUSIP, une nouvelle quantité, du cash en lieu des fractions |
| **Fusion en cash / Delisting** | Le prêt est **clôturé** : l'emprunteur rend du cash au lieu des titres |
| **Tender et short (Rule 14e-4)** | Aux US, il est **interdit d'apporter à une offre plus que sa position nette longue** (*short tendering*). Un client long 500 000 et short 200 000 sur le même titre ne peut apporter que **300 000**. Le PB doit contrôler les élections clients |

> **Point d'attention :** l'Equity Finance utilise souvent **son propre système** (et parfois un système de place pour la gestion des prêts). Les positions de prêt doivent être **réconciliées avec le système CA** à la record date. Sinon : substitute payments mal alloués et manufactured payments manquants.

---

## 10G. Synthetic Prime en détail

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-27** (onglet `P3bis_PB_Profondeur`)

### Pourquoi les clients utilisent le synthétique

- **Efficacité de bilan** pour le PB, donc meilleur prix pour le client.
- **Accès** à des marchés difficiles (marchés étrangers, contraintes locales).
- **Simplicité opérationnelle** pour le client : pas de règlement, pas de custody, pas de proxy.
- **Levier** intégré.
- Raisons **fiscales** (encadrées, notamment par la **871(m)** aux US).

### Mécanique d'un portfolio swap

```
Client (hedge fund)                        PB (Synthetic desk) 
   │   reçoit la performance de XYZ          │
   │   + DIVIDENDES (pass-through)  ◄────────┤  détient le HEDGE physique (100 000 XYZ)
   │   paie le financement (SOFR + spread)──►│  reçoit les vrais dividendes en DTC
   │                                          │
   └── Resets périodiques (mensuels…) : règlement du P&L, du financement et des dividendes cumulés
```

### Comment les CA impactent le synthétique

| Événement | Traitement swap (selon contrat ISDA / confirmation) | Breaks typiques |
|---|---|---|
| **Dividende cash** | **Dividend amount** = X % du brut (100 % pour certains clients, net de retenue pour d'autres) ; payé à la pay date ou au **reset** | % de pass-through incorrect ; retenue 871(m) non appliquée ; date de paiement incohérente avec le reset |
| **Dividende sur short swap** | Le client **paie** le dividende | Montant ou signe erroné |
| **Split / Stock dividend** | **Ajustement** du nombre d'actions de référence et du prix initial (notionnel inchangé) | Ajustement non appliqué → P&L client faux |
| **Spin-off** | Le swap est ajusté : soit une nouvelle ligne sur la spin-off, soit un ajustement de prix | Nouvelle ligne non créée → exposition perdue |
| **Fusion / Tender (Extraordinary Events)** | Selon le contrat : **ajustement** (*Calculation Agent Adjustment*), **modified calculation**, ou **terminaison** (*cancellation and payment*) | Traitement non aligné avec ce qui arrive au hedge physique |
| **Élections** | Le client n'élit pas « juridiquement » sur un swap, mais le contrat ou la pratique peut lui permettre d'**indiquer une préférence** ; le PB élit sur son **hedge** | Choix du client ≠ choix appliqué au hedge → mismatch économique |
| **Proxy** | Le client n'a **pas** de droit de vote via un swap | Demandes clients à gérer (question de gouvernance) |

> **Recon clé (B11) :** pour chaque événement, **ce que le PB reçoit sur le hedge** doit couvrir **ce que le PB paie au client sur le swap** (+ la retenue reversée à l'IRS). Les écarts structurels (taxe, % de pass-through) doivent être **expliqués** ; les autres sont des breaks.

---

## 10H. Treasury, funding et collatéral

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-28** (onglet `P3bis_PB_Profondeur`)

| Sujet | Impact des CA |
|---|---|
| **Prévision de trésorerie** | La Treasury a besoin de connaître les **flux CA à venir** (dividendes à recevoir et à payer, fusions cash, remboursements d'obligations). Un gros dividende spécial ou une fusion cash de plusieurs centaines de M$ change le besoin de financement du jour |
| **Cash non reçu** | Un break de paiement (B5) = du cash attendu qui n'est pas là → coût de financement, voire **découvert** sur le compte DTC |
| **Collatéral donné** (tri-party, CCP, contreparties de repo) | Si un titre donné en collatéral fait l'objet d'une CA (fusion, delisting, call d'obligation), il faut le **substituer** ou gérer le flux (manufactured) |
| **Collatéral reçu** | La valeur change avec les CA ; le collatéral doit être réévalué |
| **FX** | Événements en devises (ADR, titres étrangers) → conversions et risque de change |
| **Paiements aux clients** | Le PB crédite souvent les clients à la pay date **même si le marché n'a pas encore payé** (*contractual income*) ou seulement à réception (*actual income*) selon la politique interne **[À VÉRIFIER]**. En *contractual*, chaque paiement de marché manquant devient un **trou de trésorerie** et un risque de perte |

> **Contractual vs Actual settlement :** c'est une **question clé** à poser en interne. Elle change la nature du risque. En *contractual*, les breaks B5 représentent de l'argent **déjà avancé** au client.

---

## 10I. Reporting, NAV et expérience client

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-29** (onglet `P3bis_PB_Profondeur`)

### Ce que le client reçoit (en général)

- Fichiers quotidiens : **positions**, **trades**, **cash balances**, **financing accruals**, **income accruals** (dividendes et coupons à recevoir et à payer), **CA notifications**, **projected income**.
- Notifications d'événements (portail, email, SWIFT pour certains clients).
- Confirmations d'élections et de paiements.
- Relevés mensuels, reporting fiscal annuel (1099 / 1042-S).

### Pourquoi c'est sensible : la **NAV**

Les hedge funds calculent une **NAV** (*Net Asset Value*), souvent mensuelle, parfois plus fréquente. Leurs investisseurs souscrivent et rachètent à cette NAV.

- Un **dividende manquant** ou un **accrual erroné** dans les fichiers PB → l'administrateur trouve un écart → **query** au PB.
- Si l'écart n'est pas résolu avant la clôture de la NAV, le fonds doit **estimer**, voire **corriger sa NAV** plus tard. C'est très mal vu par les investisseurs et les auditeurs du fonds.
- **Les périodes de fin de mois sont donc critiques :** tous les breaks CA touchant les clients doivent être expliqués **avant** la clôture des NAV.

### Les queries des administrateurs

Questions typiques : « Pourquoi le dividende XYZ est-il net de 30 % ? » « Pourquoi l'accrual a-t-il disparu sans paiement ? » « Pourquoi la quantité post-split diffère-t-elle de notre calcul ? » « Pourquoi avons-nous reçu un substitute payment et non un dividende ? »

> **Idée produit :** un rapport de breaks **orienté client** (par fonds) permettrait au Client Service de répondre **de manière proactive**, avant même la query.

---

## 10J. Réglementation propre au PB

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-30** (onglet `P3bis_PB_Profondeur`)

| Règle | Contenu (simplifié) | Lien avec les CA |
|---|---|---|
| **SEC Rule 15c3-3** (Customer Protection) | Possession/contrôle des titres clients fully paid et excess margin ; **Reserve Formula** (calcul périodique des montants dus aux clients, à bloquer dans un compte spécial) | Les dividendes et revenus dus aux clients entrent dans les calculs ; les titres touchés par une CA doivent rester en possession/contrôle (attention aux réorgs et aux titres en transit) |
| **SEC Rule 15c3-1** (Net Capital) | Capital minimum du broker-dealer, avec des **déductions** pour certains éléments risqués | Des écarts de titres non résolus (**short securities differences**) et des créances anciennes peuvent entraîner des **charges de capital** au-delà de certains délais. **Un break qui vieillit peut coûter du capital** |
| **SEC Rule 17a-13** | **Comptage trimestriel** des titres et réconciliation du stock record | Les écarts de position liés aux CA doivent être expliqués |
| **Reg T / FINRA 4210** | Marge | Voir §10E |
| **Reg SHO** | Locate, close-out des fails, règles sur les ventes à découvert | Fails autour des record dates → claims, buy-ins |
| **SEC Rule 14e-4** | Interdit de **tender plus que sa position nette longue** | Contrôle obligatoire des élections clients |
| **IRS (chapitre 3 / FATCA / 871(m) / QI / QDD)** | Retenues et reporting fiscal | Voir §17 |
| **Books & Records (17a-3 / 17a-4)** | Conservation des registres | Traçabilité des CA et des élections : ton pipeline doit **archiver** ses inputs et outputs |

> **Message à porter auprès du sponsor :** la réduction de l'aging des breaks n'est pas qu'un gain de productivité. Elle a aussi un **intérêt réglementaire et de capital**. Valide les seuils exacts avec Regulatory Reporting / Finance **[À VÉRIFIER]**.

---

## 10K. Matrice d'impact des CA sur les fonctions PB

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-31** (onglet `P3bis_PB_Profondeur`)

| Événement | Stock record / Positions | Marge / Risque | Equity Finance (prêts/emprunts) | Synthetic (swaps) | Client / Admin / NAV | Tax |
|---|---|---|---|---|---|---|
| **Dividende cash (DVCA)** | Positions éligibles à la record date | Accrual nécessaire à l'ex-date | Manufactured dus/reçus ; recalls possibles | Dividend pass-through | Accruals dans les fichiers ; paiement | Retenue selon le statut ; substitute vs vrai dividende |
| **Dividende spécial important** | Idem + **due bill** possible | Forte baisse du prix ; ajustement des options OCC | Manufactured importants | Pass-through + ajustement éventuel | Impact NAV visible | Caractérisation (ROC ?) |
| **Split** | Quantités × N, **due bill** si ≥ 25 % | Prix ÷ N **synchronisé** | Prêts × N, collatéral ajusté | Nombre d'actions de référence ajusté | Quantités dans les fichiers | — |
| **Reverse split** | Quantités ÷ N, **fractions → CIL** | Idem | Prêts ajustés, fractions | Ajustement | CIL à expliquer | CIL = vente (1099-B) |
| **Spin-off** | Nouvelle position, nouveau CUSIP | Nouveau titre **à valoriser** | Nouveaux prêts sur le titre spin-off | Nouvelle ligne de swap ou ajustement | Nouvelle position + coût de revient | Allocation du coût (8937) |
| **Fusion cash** | Positions soldées, cash reçu | Collatéral transformé en cash | **Prêts clôturés** en cash | Terminaison ou ajustement (ISDA) | Fin de position | Vente imposable |
| **Fusion titres / mixte** | Ancien CUSIP → nouveau ; fractions | Nouveau titre dans le moteur de risque | **Transformation** des prêts | Ajustement ou terminaison | Nouvelle position | Selon la structure |
| **Tender / Buyback (volontaire)** | Titres **bloqués** après apport | Titres apportés moins liquides | Recalls, **liability** sur titres prêtés, **14e-4** | Choix sur le hedge ≠ préférence client | **Élection client**, deadline, proration | Vente imposable |
| **Rights issue** | Droits crédités, puis actions | Droits à valoriser, expiration | Droits manufacturés | Ajustement | Élection (exercer/vendre/laisser) | — |
| **Changement de nom / CUSIP** | Mapping ancien → nouveau | Référentiel à jour | Prêts sur le nouveau CUSIP | Mapping | Fichiers avec le nouveau CUSIP | — |
| **Delisting / Faillite** | Titre parfois retiré de la DTC | Haircut 100 %, appels de marge | Prêts difficiles à clôturer | Événement extraordinaire | Valorisation NAV délicate | Pertes (worthless) |
| **Coupon / Call obligataire** | Nominal, lottery DTC (partial call) | Valeur du collatéral obligataire | Manufactured coupon (repo) | Swaps de crédit/TRS obligataires | Accruals d'intérêts | 1099-INT / 1042-S |
| **Assemblée générale (proxy)** | Positions votantes (hors titres prêtés) | — | **Recalls** pour voter | Pas de droit de vote | Demandes de vote, activistes | — |

---

## 10L. Chronologie d'un événement vu du PB

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-32** (onglet `P3bis_PB_Profondeur`)

Exemple d'une **OPA cash volontaire** sur une action US détenue par plusieurs fonds (longs, shorts, prêts, swaps).

| Moment | Qui | Quoi |
|---|---|---|
| **J-30** Annonce | Event Management | Capture (Bloomberg, DTC), création de l'événement, golden copy |
| J-30 | Equity Finance | Anticipe la hausse de la demande d'emprunt ; surveille les recalls |
| J-29 | CA Ops / Client Service | **Notification aux clients** concernés (longs, swaps) avec la **client deadline** |
| J-29 | Risk | Revue des concentrations sur le titre et l'acquéreur |
| J-20 → J-5 | Clients | Envoient leurs élections ; le merger arb élit très tôt |
| J-5 | CA Ops | **Contrôle des élections** : élection ≤ position disponible ; respect de la **14e-4** (net long) ; titres prêtés identifiés |
| J-5 | Equity Finance | **Recalls** des titres prêtés nécessaires aux élections |
| J-3 → J-1 | CA Ops | Relances des clients sans réponse ; préparation des instructions |
| **J** Expiration (souvent 17h ou minuit NY) | CA Ops | Instructions **ATOP** en DTC ; **guaranteed delivery / protect** pour les titres non encore revenus |
| J | Synthetic desk | Décision d'élection sur le **hedge** physique |
| J+1 → J+2 | Equity Finance / Ops | **Couverture des protects** (titres rappelés livrés à temps ?) |
| J+2 → J+5 | Agent | Résultat, **proration** éventuelle |
| J+5 | CA Ops | Calcul des allocations clients selon la proration ; mise à jour des positions |
| J+5 → J+7 | DTC | **Paiement** cash et retour des titres non acceptés |
| J+7 | CA Ops / Recon | **Recon expected vs received** (cash et titres rendus) |
| J+7 | Synthetic | Règlement/ajustement des swaps |
| J+7 | Equity Finance | Clôture ou transformation des prêts ; règlement des **liabilities** |
| J+8 | Client / Admin | Recon côté fonds → queries éventuelles |
| J+30 | Claims | Suivi des claims ouverts (contreparties de prêt, fails) |

---

## 10M. Scénarios d'incidents typiques en PB

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-33** (onglet `P3bis_PB_Profondeur`)

| # | Scénario | Cause | Conséquence |
|---|---|---|---|
| 1 | **Élection manquée** d'un fonds de merger arb | Notification non reçue, deadline interne mal calculée | Le fonds ne participe pas à l'offre → **le PB indemnise** la différence de valeur (peut atteindre des millions) |
| 2 | **Over-election** | Élection sur une position déjà vendue ou prêtée | Instruction rejetée ou protect non couvert → indemnisation |
| 3 | **Protect non couvert** | Titres prêtés non revenus à temps | Le client perd le bénéfice de l'offre → indemnisation |
| 4 | **Short tendering** | Élection > position nette longue | **Violation réglementaire** (14e-4) |
| 5 | **Split non synchronisé** | Prix ajusté dans le système de marge mais pas la quantité | Appel de marge erroné → plainte client, voire liquidation forcée à tort |
| 6 | **Dividende non crédité** au client | Accrual manquant ou paiement non alloué | Query admin ; erreur de NAV ; intérêts facturés à tort |
| 7 | **Mauvais taux de retenue** | Statut fiscal non à jour | Sous-retenue (risque IRS, pénalités) ou sur-retenue (plainte client, reclaim) |
| 8 | **Manufactured payment non réclamé** | Break de prêt non détecté | Perte sèche après un certain délai |
| 9 | **Swap non ajusté** après un spin-off | Ligne de swap non créée | P&L client faux ; dispute avec le client |
| 10 | **Recall non anticipé** avant un vote | Activiste qui veut voter | Fail, buy-in, tension avec le client |
| 11 | **Événement non capté** sur un titre peu liquide | Absent d'une source | Rien n'est fait → perte découverte tardivement |

> Ces scénarios constituent **ton argumentaire** auprès du sponsor : chaque contrôle automatisé doit être relié à un ou plusieurs de ces risques.

---

## 10N. Conséquences pour ton projet

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-34** (onglet `P3bis_PB_Profondeur`)

Use cases **propres au PB**, qui s'ajoutent au backlog du §32 (sans le remplacer) :

| # | Use case PB | Sources | Ce qu'on détecte | Scénarios couverts (§10M) |
|---|---|---|---|---|
| UC11 | **Election availability check** : élections clients vs position disponible (longs non prêtés + titres rappelables) vs net long (14e-4) | Élections, stock record, Equity Finance | Over-election, short tendering, protects à risque | 2, 3, 4 |
| UC12 | **Recall planner** : titres prêtés nécessaires aux élections ou aux votes à venir | Événements volontaires et assemblées, prêts, élections | Recalls à lancer à J-N | 3, 10 |
| UC13 | **CA consistency check cross-systems** à l'ex-date / effective date : quantités, prix, CUSIP, accruals cohérents entre système CA, marge, stock record, Equity Finance et swaps | Tous systèmes | Splits non synchronisés, spin-offs non valorisés, CUSIP non mappés | 5, 9 |
| UC14 | **Income allocation check** : vrai dividende vs substitute payment vs short charges vs manufactured | Positions, prêts, paiements | Allocations incohérentes | 6, 8 |
| UC15 | **Swap vs hedge dividend recon** (déjà UC7, version PB détaillée : % pass-through par client, 871(m)) | Système swaps, paiements DTC, contrats | Écarts non expliqués | 9 |
| UC16 | **Client-facing break report** par fonds, avant la clôture des NAV (fin de mois) | Breaks + mapping comptes/fonds | Breaks à expliquer proactivement aux clients | 6, 7 |
| UC17 | **Event coverage check** : chaque titre détenu ou prêté a-t-il ses événements captés dans toutes les sources ? | Positions, Bloomberg, DTC | Événement manquant | 11 |

**Questions PB supplémentaires à poser en interne :**
- [ ] Paiement des revenus aux clients : **contractual** ou **actual** ?
- [ ] Comment l'Equity Finance est-elle informée des élections et des votes à venir ?
- [ ] Où vit le contrôle 14e-4 aujourd'hui ?
- [ ] Quels clients sont en **synthetic** et avec quels % de pass-through ?
- [ ] Les clients et systèmes hérités d'autres plateformes sont-ils entièrement migrés ?
- [ ] Quels breaks CA entrent dans les calculs de capital ou de réserve (15c3-1 / 15c3-3) ?
- [ ] Calendrier des NAV des principaux clients (fin de mois) et volume des queries des administrateurs ?

---

# PARTIE IV — PAR CLASSE D'ACTIFS

## 11. Actions, ADR, ETF/fonds, REIT/MLP, SPAC

### 11.1 Actions US

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-35** (onglet `P4_Classes_Actifs`), **EX-36** (onglet `P4_Classes_Actifs`)

- Le volume est dominé par **DVCA**.
- Événements fréquents : **splits**, **spin-offs**, **mergers**, **tender offers**, **buybacks**.
- **Fractions :** un reverse split 1:10 sur 1 005 actions donne 100,5. L'émetteur paie la fraction en **cash in lieu (CIL)**, souvent au prix de vente agrégé des fractions. Fiscalement, c'est une **vente** (reportée sur le 1099-B).
- **Spin-off :** il faut aussi gérer la **répartition du coût de revient** (*cost basis allocation*) entre parent et spin-off (formulaire IRS **8937** publié par l'émetteur).

### 11.2 **ADR** (*American Depositary Receipts*)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-37** (onglet `P4_Classes_Actifs`)

- Certificats US représentant des actions étrangères, émis par une **depositary bank**.
- Un dividende ADR = dividende étranger, converti en USD (**FX**), moins la **retenue fiscale étrangère**, moins les **ADR fees** (frais de dépositaire par ADR).
- **Breaks typiques :** taux de change, taux de retenue étrangère, fees, **ratio** ADR/action ordinaire. Les dates ADR peuvent différer de celles du titre local.

### 11.3 ETF et fonds

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-38** (onglet `P4_Classes_Actifs`)

- Distributions de dividendes, de **capital gains** (souvent en fin d'année), de **return of capital**.
- **Reclassification de fin d'année :** la nature fiscale (dividende ordinaire / ROC / capital gain) est parfois **connue seulement en début d'année suivante** → corrections du reporting 1099.

### 11.4 REIT et MLP

- **REIT :** distributions souvent requalifiées en fin d'année (ROC, capital gain), même logique que les fonds.
- **MLP** (*Master Limited Partnerships*) : ce sont des partnerships, pas des sociétés. Les distributions sont largement du *return of capital*, le reporting se fait via **K-1**, et il existe des **régimes de retenue spécifiques pour les non-US**. C'est un sujet Tax pur **[À VÉRIFIER AVEC TAX]**.

### 11.5 **SPAC**

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-39** (onglet `P4_Classes_Actifs`)

- **Units** = action + fraction de warrant, qui se **séparent** (*unit split*).
- **Redemption rights** : avant le *business combination*, les porteurs peuvent demander le remboursement de leurs actions au prorata du trust (événement volontaire avec deadline).
- Ensuite fusion (**de-SPAC**) : changement de CUSIP et de nom, warrants convertis.
- Les fonds d'arbitrage SPAC sont des clients PB typiques.

### 11.6 Droits et warrants

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-40** (onglet `P4_Classes_Actifs`)

- **Rights offering :** distribution de droits (RHDI) → exercice (EXRI), vente (SLLE) ou expiration (LAPS). Parfois avec **oversubscription**.
- **Warrants :** exercice (EXWA), ajustements de ratio après d'autres CA.

---

## 12. Fixed Income : corporates, munis, Treasuries, MBS/ABS

### 12.1 Corporate bonds (en DTC)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-41** (onglet `P4_Classes_Actifs`), **EX-42** (onglet `P4_Classes_Actifs`)

- **INTR** : coupons. Montant = nominal × taux × fraction selon la **day count convention** (30/360, ACT/360, ACT/365…).
- **Floating rate notes** : taux refixé à chaque période (aujourd'hui sur **SOFR** pour l'USD) → **rate breaks** fréquents.
- **REDM** (échéance), **MCAL** (call total), **PCAL** (call partiel).
  - **Partial call en DTC :** la DTC alloue les titres remboursés entre participants par **tirage au sort** (*lottery*), puis le PB doit **réallouer** entre ses clients selon une méthode équitable. **Breaks fréquents** sur cette allocation.
- **BPUT** : put à l'initiative du porteur.
- **CONS** : consent solicitations (souvent avec un **consent fee**).
- **TEND / EXOF** : offres de rachat ou d'échange de dette, parfois avec **early tender premium** et date de fin anticipée.
- **Default / Bankruptcy** : coupons non payés, distributions de restructuration (nouveaux titres, cash, *equity*). Très manuel.
- **PIK** (*Payment in Kind*) : coupon payé en titres.

### 12.2 Municipal bonds

- Source officielle : **EMMA (MSRB)**.
- Spécificités : **pre-refunding / escrow** (CUSIP changé), **mandatory puts / tenders**, **sinking funds**.
- La fiscalité des intérêts munis (souvent exonérés fédéralement) a un impact sur le reporting.

### 12.3 US Treasuries

- Détenus via **Fedwire**, pas en DTC.
- Peu d'événements : coupons et échéances. Mais attention aux **TIPS** (principal indexé sur l'inflation) et aux **STRIPS**.

### 12.4 **MBS / ABS** (*Mortgage / Asset-Backed Securities*)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-43** (onglet `P4_Classes_Actifs`)

- Le principal est remboursé **chaque mois** selon un **factor** (facteur de pool) publié mensuellement.
  - `Current face = Original face × Factor`
  - `Principal payment = Original face × (Factor_prev − Factor_new)`
- **Agency MBS** (Fannie Mae, Freddie Mac, Ginnie Mae) : en Fedwire. **Non-agency** (private label) et ABS : généralement en DTC, avec des données de **trustee reports**.
- Record date généralement en fin de mois précédent, avec un **délai de paiement** (*payment delay*) spécifique à chaque type de programme.
- **Breaks typiques :** facteur différent entre sources, facteur publié en retard, *interest shortfall*, ajustements rétroactifs.
- **TBA** (*To Be Announced*) : trades forward sur MBS ; les CA impactent le moment de l'allocation.

---

## 13. Dérivés listés et OTC

### 13.1 Options listées (OCC)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-44** (onglet `P4_Classes_Actifs`)

- L'**OCC** ajuste les contrats après certaines CA et publie des **Information Memos**.
- **Ajustés :** splits, reverse splits, spin-offs, fusions, **dividendes spéciaux** (au-dessus d'un seuil défini par l'OCC). En général, **les dividendes ordinaires ne sont pas ajustés**.
- Après ajustement, on obtient un **adjusted option** : nouveau symbole, nouveau **deliverable** (par exemple 100 actions + cash + actions du spin-off), multiplicateur modifié.
- Pour un PB : il faut mettre à jour les positions d'options des clients, leurs marges et leur valorisation.

### 13.2 Futures

- Futures sur actions individuelles : ajustements similaires.
- Futures sur indices : l'**index provider** gère l'impact des CA dans l'indice (*index divisor*).

### 13.3 OTC (swaps, TRS, CFD)

- Ils sont régis par les **ISDA Equity Derivatives Definitions** (ou par la confirmation) : *Potential Adjustment Events*, *Extraordinary Events* (**Merger Event**, **Tender Offer**, **Nationalization**, **Insolvency**, **Delisting**).
- La **Calculation Agent** (souvent la banque) détermine l'ajustement.
- **Dividend pass-through** sur les swaps : *dividend amount* = % du dividende brut ou net, selon le contrat.
- **Recon clé :** dividendes reçus sur le **hedge** physique vs dividendes payés sur le **swap**.

### 13.4 Convertibles

- Ajustement du **conversion ratio** après des CA sur le sous-jacent (dividendes au-dessus d'un seuil, splits…).
- **Deemed distributions** (section 305(c)) : l'ajustement du ratio peut être fiscalement traité comme un dividende réputé reçu, avec une retenue pour les non-US **[À VÉRIFIER AVEC TAX]**.

---

## 14. Securities Lending et Repo

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-45** (onglet `P4_Classes_Actifs`)

| Concept | Description |
|---|---|
| **Lender / Borrower** | Le prêteur transfère la propriété légale, mais garde le **bénéfice économique** |
| **Manufactured payment** | L'emprunteur verse au prêteur l'équivalent de chaque distribution |
| **Recall** | Le prêteur rappelle les titres (pour voter, élire ou vendre) |
| **Voluntary CA on loaned securities** | Le prêteur instruit l'emprunteur, qui doit lui livrer le résultat économique |
| **Income tracking** | Suivi des manufactured payments dus et reçus. **Source majeure de breaks et de claims** |
| **Repo** | Même logique : le cédant temporaire garde le bénéfice économique (*manufactured coupon*) |
| **Mark-to-market** | Après une CA, la valeur du prêt et du collatéral change (split, spin-off) → réajustement |

Aux US, DTC propose un service de **Claim Connect** pour faciliter le règlement de certaines réclamations de revenus entre participants.

---

# PARTIE V — LES OPÉRATIONS AU QUOTIDIEN

## 15. Le cycle de vie détaillé

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-46** (onglet `P5_Operations`)

| # | Étape | Inputs | Outputs | Contrôles | Breaks possibles |
|---|---|---|---|---|---|
| 1 | **Event capture** | Bloomberg, DTC, Broadridge, custodians, agents | Événements bruts | Complétude (aucun titre détenu sans événement connu) | Événement manquant |
| 2 | **Scrubbing / Validation** | Événements bruts multi-sources | **Golden copy** | Comparaison des sources, 4-eyes | **Event data breaks** |
| 3 | **Position identification** | Stock record, positions clients, lending, swaps | Positions éligibles à la record date | Recon stock record vs DTC | **Position breaks** |
| 4 | **Entitlement calculation** | Golden copy + positions + statut fiscal | Entitlements attendus (gross, tax, net) par compte | Recalcul, comparaison avec l'entitlement DTC/custodian | **Entitlement breaks** |
| 5 | **Client notification** | Golden copy + positions | Notifications (portail, email, SWIFT) | Exhaustivité, délais | Client non notifié |
| 6 | **Election collection** | Réponses clients | Élections agrégées | Élection ≤ position disponible ; deadline | **Over-election**, élection tardive |
| 7 | **Instruction** | Élections agrégées | Instructions à la DTC (ex. via **ATOP** pour les tenders) / au custodian | Statuts MT567 | Instruction rejetée |
| 8 | **Accrual booking** | Entitlements | Écritures *income receivable / payable* à l'ex-date | Recon comptable | **Accrual breaks** |
| 9 | **Payment / Settlement** | Paiements DTC/custodian, MT566 | Cash/titres reçus et alloués aux clients | Recon **expected vs received** | **Cash / stock breaks** |
| 10 | **Client allocation** | Montants reçus | Crédits/débits clients | Somme des allocations = montant reçu | **Allocation breaks** |
| 11 | **Claims** | Breaks avec contreparties | Claims émis et reçus | Suivi de l'aging | Claims non réglés |
| 12 | **Tax reporting** | Paiements, statuts fiscaux | 1099, 1042-S | — | Reclassifications |
| 13 | **Close-out** | — | Événement clôturé | Plus aucun break ouvert | — |

---

## 16. Organisation d'une équipe CA PB et journée type

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-47** (onglet `P5_Operations`)

### 16.1 Équipes typiques (l'organisation réelle de ta banque est **[À VÉRIFIER EN INTERNE]**)

| Équipe | Périmètre |
|---|---|
| **Announcements / Event Management** | Capture et validation (golden copy) |
| **Income / Dividends** | Événements cash obligatoires, accruals, paiements |
| **Mandatory Reorg** | Splits, fusions, changements de CUSIP |
| **Voluntary Reorg / Elections** | Tenders, droits, élections, deadlines, protects |
| **Claims** | Market claims, manufactured payments, compensations |
| **Tax** | Retenues, documentation, reclaims, reporting |
| **Proxy** | Votes |
| **Client Service** | Queries clients et fund admins |
| **Offshore** | Souvent une partie du traitement est faite dans des centres offshore (Inde, Portugal…) **[À VÉRIFIER]** |

### 16.2 Journée type (NYC)

| Heure | Activité |
|---|---|
| 7h00–9h00 | Lecture des flux de la nuit (Bloomberg, DTC), nouvelles annonces, mises à jour |
| 9h00–10h00 | Revue des **paiements du jour** (pay date) : expected vs received |
| 10h00–12h00 | Investigation des breaks, emails aux custodians, aux agents et aux contreparties de prêt |
| 12h00–14h00 | **Deadlines volontaires du jour** : relance clients, agrégation des élections, instructions ATOP |
| 14h00–16h00 | Queries des fund admins et des clients, claims |
| 16h00–18h00 | Préparation de la record date du lendemain, checks de positions, reporting management |

**Là où les ops perdent du temps (= tes opportunités) :** extractions de plusieurs systèmes, copier-coller dans Excel, VLOOKUP, recherche manuelle d'écarts, emails répétitifs, suivi de l'aging dans des fichiers partagés.

---

## 17. Fiscalité US

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-48** (onglet `P5_Operations`)

### 17.1 Principes

| Bénéficiaire | Régime de base sur dividendes US |
|---|---|
| **US person** (W-9 valide) | Pas de retenue (sauf **backup withholding** à 24 % si la documentation manque ou est invalide). Reporting **1099-DIV** / 1099-INT / 1099-MISC / 1099-B |
| **Non-US person** (W-8BEN, W-8BEN-E, W-8IMY…) | **NRA withholding** à **30 %** par défaut, réduit si une **convention fiscale** s'applique (*treaty rate*, souvent 15 % pour les dividendes de portefeuille). Reporting **1042-S** |
| **Intérêts** de portefeuille (*portfolio interest exemption*) | Souvent exonérés pour les non-US sous conditions |

### 17.2 Concepts clés

- **QI** (*Qualified Intermediary*) : statut d'intermédiaire auprès de l'IRS, qui permet d'appliquer les taux conventionnels sans transmettre l'identité des bénéficiaires.
- **QDD** (*Qualified Derivatives Dealer*) : statut lié aux dérivés et à la 871(m).
- **FATCA** (chapitre 4) et **NRA** (chapitre 3) : deux régimes de retenue qui coexistent.
- **Section 871(m)** : retenue sur les **dividend equivalents** payés à des non-US via des dérivés sur actions US (swaps delta-one notamment, avec des règles de phase-in pour les autres). **Très important en PB synthétique.**
- **Section 305(c)** : distributions réputées (convertibles, ajustements de ratio).
- **Tax reclaim** : récupération du trop-perçu (surtout pour des titres étrangers).
- **Income characterization** : un même paiement peut être dividende, ROC, capital gain, intérêt ou substitute payment. Chacun a sa fiscalité.

### 17.3 Breaks fiscaux typiques

| Symptôme | Cause probable |
|---|---|
| Net reçu = gross × 70 % au lieu de 85 % | Documentation fiscale non prise en compte, mauvais taux de convention |
| Retenue sur un client US | W-9 manquant → backup withholding |
| Retenue sur un ROC | Reclassification non encore connue |
| Écart sur un swap | Calcul 871(m) |

---

## 18. Comptabilité : accruals et income

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-49** (onglet `P5_Operations`)

- À l'**ex-date** : on comptabilise un **income receivable** (créance) pour les longs et un **income payable** (dette) pour les shorts.
- À la **pay date** : on solde la créance contre le cash reçu.
- Si le cash ne vient pas, la créance reste ouverte : c'est un **aged receivable**, un indicateur de break.
- **Recon clé :** *open accruals* vs *expected payments*. Tout accrual ouvert après la pay date + N jours est un break à investiguer.
- Impact P&L : un break non résolu peut finir en **write-off** (perte) si la banque a payé le client sans encaisser du marché.

---

# PARTIE VI — LES BREAKS (le cœur de ton projet)

## 19. Taxonomie des breaks

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-50** (onglet `P6_Breaks`)

| # | Famille | Ce qu'on compare | Granularité | Root causes typiques | Timing |
|---|---|---|---|---|---|
| B1 | **Event data break** | Golden copy vs Bloomberg vs DTC vs Broadridge | Événement | Taux, dates, ratio, devise, options, statut différents ; événement manquant | Dès l'annonce, puis à chaque mise à jour |
| B2 | **Missing event** | Positions détenues vs événements connus | Titre | Titre détenu sans événement capté (ou l'inverse) | Avant la record date |
| B3 | **Position break** | Positions internes (stock record) vs position DTC/custodian à la record date | Titre × lieu | Trades non réglés, fails, booking, lending non reflété | Record date |
| B4 | **Entitlement break** | Entitlement interne vs entitlement DTC/custodian | Événement × compte de dépôt | B1 ou B3, arrondis, fractions | Entre record date et pay date |
| B5 | **Cash break** | Expected vs received (MT566, nostro, DTC) | Événement × compte | Paiement manquant, partiel, en retard, mauvais taux, taxe, devise | Pay date et après |
| B6 | **Stock break** | Titres attendus vs reçus (splits, spin-offs, stock dividends) | Événement × compte | Ratio, fractions, CUSIP | Pay/effective date |
| B7 | **Tax break** | Taxe attendue (selon le statut client) vs taxe retenue | Événement × client | Documentation, taux de convention, caractérisation | Pay date |
| B8 | **Allocation break** | Σ allocations clients vs montant reçu | Événement | Rehypo/substitute payment, arrondis, shorts | Après paiement |
| B9 | **Short / Borrow break** | Dividendes débités sur les shorts clients vs payés aux prêteurs | Événement | Taux, positions, timing de couverture | Pay date |
| B10 | **Lending / Manufactured break** | Manufactured dus vs reçus (prêts/emprunts) | Événement × contrepartie | Position de prêt, taux, timing | Pay date + |
| B11 | **Swap vs Hedge break** | Dividendes swap (pass-through) vs dividendes reçus sur le hedge | Événement × swap | % pass-through, 871(m), notionnel | Pay date |
| B12 | **Election break** | Élections clients vs instructions envoyées vs position disponible | Événement volontaire | Over-election, instruction manquante, protect non couvert | Avant la deadline (**critique**) |
| B13 | **Deadline risk** | Événements volontaires avec deadline proche et clients sans réponse | Événement × client | Pas de réponse client | J-5 → J |
| B14 | **Accrual break** | Accruals ouverts vs paiements | Événement | Paiement non reçu, accrual incorrect | Après la pay date |
| B15 | **Claim break** | Claims ouverts (émis/reçus) | Claim | Contrepartie ne paie pas | Aging |
| B16 | **Reference data break** | CUSIP/ticker/nom après une réorg vs static data | Titre | Mise à jour manquée | Effective date |
| B17 | **Derivative adjustment break** | Ajustements OCC vs positions d'options | Contrat | Ajustement non appliqué | Effective date |

> **Priorisation suggérée** (à valider avec le sponsor) : **B12/B13** (risque de perte élevé, temps critique), **B5** (gros volume, simple), **B1** (en amont, il prévient le reste), puis B4, B7, B9, B10, B11.

---

## 20. Clés de matching et tolérances

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-51** (onglet `P6_Breaks`)

### 20.1 Matcher un **événement** entre sources

Ordre de préférence :

1. **COAF** (s'il est présent dans les deux sources)
2. **CUSIP/ISIN + CAEV + Pay date** (+ ex-date)
3. **CUSIP + CAEV + Record date**
4. Matching approximatif (**fuzzy**) : CUSIP + type + dates à ±N jours, pour les cas où une source a une date erronée

Pièges :

- Plusieurs événements le même jour sur le même titre (dividende ordinaire + spécial).
- Changement de CUSIP (utiliser une table de mapping).
- Différences de typologie entre vendors (un vendor dit `CAPD`, un autre `DVCA` avec un flag).

### 20.2 Matcher un **paiement**

- Clé : **Event key + compte (account) + devise**
- Puis comparaison des montants : **gross**, **tax**, **net**.
- Gérer le **many-to-one** (plusieurs paiements partiels pour un entitlement) et les **reversals** (paiement annulé puis refait).

### 20.3 Tolérances (**[À VALIDER AVEC LES OPS]**)

| Type | Tolérance typique |
|---|---|
| Arrondi cash | ±0,01 $ par ligne, ou ±1 $ par événement |
| Taux | Comparaison à 6–8 décimales |
| Quantité de titres | 0 (exact), sauf fractions traitées en CIL |
| Dates | 0 jour, sauf tolérance explicite (ex. jour férié) |

### 20.4 Classification automatique des causes (*auto root-cause*)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-52** (onglet `P6_Breaks`)

Règles heuristiques simples, très appréciées des ops :

```
si received == 0                         → "Unpaid"
si received / expected ≈ 0.70 / 0.85     → "Tax rate difference"
si received == expected_on_other_qty     → "Position difference"
si |diff| < tolérance                     → "Rounding" (auto-close)
si pay_date > today                       → "Not yet due" (pas un break)
si event status == CANC                  → "Cancelled event"
```

---

## 21. Claims management

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-53** (onglet `P6_Breaks`)

| Type de claim | Situation |
|---|---|
| **Market claim** | Trade acheté avant l'ex-date mais réglé après la record date : le vendeur a reçu la distribution, l'acheteur la réclame |
| **Reverse claim** | Inverse : le PB a reçu à tort et doit rendre |
| **Due bill claim** | Fenêtre due bill (§4.4) |
| **Manufactured claim** | Lending / repo |
| **Fail claim** | Trade en échec (*fail*) sur la record date |
| **Transformation** | Pour les réorgs : le trade en attente sur l'ancien titre devient un trade sur le nouveau |

**Métriques :** nombre et montant des claims ouverts, **aging** (0–30, 31–60, 61–90, 90+ jours), contreparties les plus en retard. Les claims anciens finissent en **write-off**.

---

## 22. KPIs, risques et contrôles

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-54** (onglet `P6_Breaks`)

### 22.1 KPIs opérationnels

- **STP rate** (% d'événements traités sans intervention manuelle)
- Nombre et montant de **breaks ouverts**, par famille
- **Aging** des breaks
- **Time to detect** et **time to resolve**
- **Missed elections / late instructions** (cible : 0)
- **Client queries** (volume, délai de réponse)
- **Losses / write-offs** liés aux CA
- **FTE hours** passées par tâche (base du business case)

### 22.2 Risques

- **Missed election** (perte directe, réclamation client)
- **Incorrect entitlement** (surpaiement non récupérable)
- **Tax errors** (risque réglementaire IRS, pénalités)
- **Client communication failure**
- **Concentration du savoir** (une personne connaît le fichier Excel)
- **EUC risk** : macros Excel non contrôlées (voir §30)

---

# PARTIE VII — CAS PRATIQUES CHIFFRÉS

## 23. Neuf cas pratiques

### Cas 1 — Dividende avec long, short, rehypothéquation et client non-US

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-55** (onglet `P7_Cas_Pratiques`)

**Événement :** XYZ `DVCA` 0,50 $/action. Record date = ex-date = 10/11. Pay date = 20/11.

| Compte | Position | Statut |
|---|---|---|
| Fund A (US, W-9) | +200 000 long, dont 80 000 rehypothéquées et prêtées | US person |
| Fund B (Cayman, W-8BEN-E, pas de convention) | +100 000 long | Non-US |
| Fund C | −50 000 short | — |
| Street borrow (pour couvrir C) | +50 000 | Contrepartie : Lender L |
| Street loan (rehypo de A) | −80 000 | Contrepartie : Borrower K |

**Position attendue en DTC :** 200 000 + 100 000 + 50 000 − 80 000 − 50 000 (vendues par C, déjà livrées) = **220 000**

**Cash attendu de la DTC :** 220 000 × 0,50 = **110 000 $**

**Allocations :**

| Flux | Calcul | Montant |
|---|---|---|
| Fund A, vrai dividende | 120 000 × 0,50 | 60 000 $ |
| Fund A, substitute payment | 80 000 × 0,50 | 40 000 $ (payé par K au PB) |
| Fund B, gross | 100 000 × 0,50 = 50 000 $, retenue 30 % = 15 000 $ | **35 000 $ net** |
| Fund C, débit short | 50 000 × 0,50 | −25 000 $ |
| PB → Lender L (manufactured) | 50 000 × 0,50 | −25 000 $ |
| Borrower K → PB (manufactured) | 80 000 × 0,50 | +40 000 $ |

**Breaks possibles :** DTC paie 105 000 $ (position DTC à 210 000 → B3/B5) ; K ne paie pas (B10) ; Fund B est retenu à 15 % au lieu de 30 % alors qu'il n'y a pas de convention (B7).

### Cas 2 — Split 4:1 avec due bill

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-56** (onglet `P7_Cas_Pratiques`)

**Événement :** split 4-for-1 (`SPLF`). Record date 01/12, pay date 08/12, ex-date 09/12 (distribution ≥ 25 % → ex-date après la pay date).

- Un client vend 10 000 actions le 03/12 (règlement le 04/12, après la record date, mais avant l'ex-date).
- Le titre se traite « cum » : l'acheteur a droit aux 30 000 actions supplémentaires.
- Le vendeur, inscrit à la record date, reçoit les 30 000 actions le 08/12, mais **les doit** → **due bill**.
- **Break si** le système ne génère pas le due bill : le client vendeur garde 30 000 actions en trop et l'acheteur réclame.

### Cas 3 — Fusion cash + stock avec fractions

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-57** (onglet `P7_Cas_Pratiques`)

**Événement :** A est rachetée par B. Pour chaque action A : **25 $ cash + 0,3375 action B**. Prix de B = 80 $.

Client : 1 001 actions A.

- Cash = 1 001 × 25 = **25 025 $**
- Actions B = 1 001 × 0,3375 = 337,8375 → **337 actions B** + **0,8375 fraction**
- Cash in lieu = 0,8375 × prix de cession des fractions (ex. 80 $) = **67,00 $**
- **Breaks typiques :** arrondi des fractions au niveau du compte de dépôt vs au niveau du client (le PB agrège : 337,8375 + d'autres fractions de clients peuvent faire des actions entières chez la DTC !), nouveau CUSIP B non mappé.

> **Point subtil PB :** la DTC calcule les fractions **sur la position agrégée du PB**, mais le PB doit allouer **client par client**. La somme des fractions clients ≠ la fraction DTC → petite différence structurelle à expliquer (règle d'allocation **[À VÉRIFIER]**).

### Cas 4 — Tender offer avec guaranteed delivery et proration

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-58** (onglet `P7_Cas_Pratiques`)

**Événement :** OPA partielle à 50 $ sur **60 %** des actions de XYZ (`TEND`, `VOLU`). Expiration le 15/12 à 17h00 NY. Guaranteed delivery possible, protect deadline à T+1.

- Fund M (merger arb) détient 1 000 000 actions, dont 400 000 rehypothéquées.
- Il **élit** de tout apporter.
- Le PB instruit 600 000 via **ATOP** (titres disponibles) + **400 000 en protect** (guaranteed delivery).
- Le PB **rappelle** les 400 000 prêtés. S'ils reviennent avant la protect deadline, ils sont livrés. Sinon : **failed protect**, le client perd la participation sur ces titres et le PB peut devoir compenser.
- Offre sursouscrite : **proration** à 72 %. Fund M reçoit 720 000 × 50 $ = **36 M$** ; 280 000 actions sont rendues.
- **Breaks :** élections > position (B12), protect non couvert (B12), résultat de proration différent entre l'agent et le calcul interne (B5/B6).

### Cas 5 — Rights issue

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-59** (onglet `P7_Cas_Pratiques`)

**Événement :** 1 droit (RHDI) par action ; 5 droits permettent de souscrire 1 action nouvelle à 20 $. Le droit est cotable.

Client : 10 000 actions → 10 000 droits.

- Option EXER : souscrit 2 000 actions, paie 40 000 $.
- Option SLLE : vend ses droits.
- Option LAPS : laisse expirer (perte de valeur !). Pour les clients sans réponse, l'**option par défaut** du PB est cruciale.
- Option OVER : demande plus d'actions ; proration possible.
- **Breaks :** droits crédités ≠ attendus, cash de souscription débité mais actions non reçues.

### Cas 6 — Partial call d'obligation en DTC

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-60** (onglet `P7_Cas_Pratiques`)

**Événement :** `PCAL` sur 20 % de l'émission, à 101.

- Le PB détient 10 M$ de nominal (agrégé, pour 5 clients).
- La **lottery DTC** sélectionne 2,5 M$ chez le PB (pas forcément 20 % !).
- Le PB doit allouer 2,5 M$ entre ses 5 clients selon une méthode **impartiale** (lottery interne ou pro rata, selon la politique **[À VÉRIFIER]**).
- **Breaks :** allocation interne ≠ montant DTC, client informé avec un mauvais montant.

### Cas 7 — Paiement MBS (facteur)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-61** (onglet `P7_Cas_Pratiques`)

Original face 10 000 000 $. Factor précédent 0,85, nouveau facteur 0,83. Coupon 5 %, intérêts mensuels.

- Principal = 10 000 000 × (0,85 − 0,83) = **200 000 $**
- Intérêts = 10 000 000 × 0,85 × 5 % / 12 = **35 416,67 $**
- **Break typique :** le facteur utilisé en interne (vendor) est 0,8301 au lieu de 0,83 → écart de 1 000 $ de principal.

### Cas 8 — Dividende ADR

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-62** (onglet `P7_Cas_Pratiques`)

ADR ratio 1 ADR = 2 actions ordinaires. Dividende local 1,00 EUR/action. EUR/USD = 1,08. Retenue étrangère 25 %. Fee dépositaire 0,02 $/ADR.

- Gross par ADR = 2 × 1,00 × 1,08 = 2,16 $
- Retenue = 0,54 $
- Fee = 0,02 $
- **Net = 1,60 $/ADR**
- **Breaks :** taux de change différent (date de conversion), retenue à 15 % (si documentation de convention) vs 25 %.

### Cas 9 — Equity swap (synthetic prime)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-63** (onglet `P7_Cas_Pratiques`)

Client non-US long 100 000 XYZ via swap ; pass-through contractuel de 100 % du gross ; dividende 0,50 $. Dividend equivalent soumis à la 871(m) au taux applicable (ici 30 %, sans convention).

- Dividend amount dû au client = 50 000 $, moins la retenue 871(m) (15 000 $) = **35 000 $**
- Hedge : le PB détient 100 000 actions physiques et reçoit 50 000 $ de la DTC.
- **Recon swap vs hedge :** 50 000 $ reçus, 35 000 $ payés au client et 15 000 $ remis à l'IRS → OK.
- **Break si** le pass-through est mal paramétré (ex. 85 % au lieu de 100 %) ou si la retenue 871(m) n'est pas appliquée.

---

# PARTIE VIII — PROJECT MANAGEMENT

## 24. Ton double rôle PM + Dev

| Casquette | Tu es responsable de… | Piège |
|---|---|---|
| **PM** | Périmètre, priorités, planning, risques, communication, adhésion des ops, mesure des bénéfices | Passer tout ton temps à coder et ne plus piloter |
| **Dev** | Pipelines Dagster fiables, testés, maintenables, sécurisés | Coder ce qui est demandé sans challenger le besoin |

**Règle pratique :** bloque des créneaux fixes « PM » (par exemple lundi matin pour la planification, vendredi après-midi pour le reporting) et protège des plages de **deep work** pour le dev.

**Ta valeur unique :** tu comprends à la fois le métier (après ce guide) et la technique. Tu peux **traduire** les besoins ops en règles codables, et les contraintes IT en options compréhensibles pour le sponsor.

---

## 25. Le cycle d'un projet d'automatisation ops

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-64** (onglet `P8_Project_Mgmt`)

```
 0. Cadrage         → Charter, sponsor, objectifs, périmètre, parties prenantes
 1. Discovery       → Cartographie as-is, mesure du temps, inventaire des sources
 2. Priorisation    → Backlog de use cases, valeur × effort, choix du MVP
 3. Design          → Règles métier, data mapping, architecture, maquette de l'output
 4. Build (sprints) → Dev itératif, démos toutes les 2 semaines aux ops
 5. Test / UAT      → Tests avec données réelles, validation des ops
 6. Parallel run    → Ancien process et nouveau process en parallèle (2–4 semaines)
 7. Go-live         → Change management IT, bascule
 8. Hypercare       → Support renforcé 2–4 semaines
 9. BAU / Handover  → Runbook, support model, monitoring
10. Benefits        → Mesure des gains vs baseline, communication des résultats
```

**Méthode :** un **agile pragmatique** fonctionne bien. Itérations de 2 semaines, démo systématique aux utilisateurs, backlog dans Jira (ou l'outil interne), documentation dans Confluence (ou équivalent).

**Le principe le plus important :** livrer **un premier use case utile en 6–8 semaines** plutôt qu'une plateforme complète en 9 mois. La crédibilité vient du premier rapport qui fait gagner du temps aux ops.

---

## 26. Discovery : cartographier et mesurer

### 26.1 Cartographie as-is

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-65** (onglet `P8_Project_Mgmt`)

Pour chaque tâche manuelle, remplis une fiche :

| Champ | Exemple |
|---|---|
| Nom de la tâche | Recon des paiements de dividendes US du jour |
| Équipe / personne | Income team, 2 analystes |
| Fréquence | Quotidienne |
| Déclencheur | Réception du rapport de paiements DTC (heure ?) |
| Inputs (systèmes/fichiers) | Export système CA, rapport DTC, extraction cash nostro |
| Étapes | 1. Extraire… 2. Copier dans Excel… 3. VLOOKUP… 4. Filtrer… 5. Investiguer… 6. Emails… |
| Output | Excel des breaks, emails aux custodians |
| Durée | 2 h/jour/personne |
| Volume | ~300 lignes/jour, ~20 breaks |
| Douleurs | Formats instables, doublons, données manquantes |
| Risques | Break raté = argent non réclamé |
| Règles implicites | « On ignore les écarts < 5 $ », « Pour les ADR, on regarde le rapport du dépositaire » |

### 26.2 Mesurer le temps (*time & motion study*)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-66** (onglet `P8_Project_Mgmt`)

Puisque tu ne sais pas encore combien de temps ça prend :

1. Demande aux ops de **noter leur temps pendant 2 semaines** (tableau simple : tâche, début, fin, volume).
2. Ou bien **observe-les** (*shadowing*) : assieds-toi avec un analyste pendant une demi-journée. C'est **la meilleure chose que tu puisses faire** dans tes premières semaines.
3. Calcule une **baseline** : heures/jour × jours/an × nombre de personnes → **FTE**.

```
Exemple : 2 h/jour × 250 jours × 3 personnes = 1 500 h/an ≈ 0,8 FTE
```

Cette baseline est **la base du business case** et de la mesure des bénéfices. Sans elle, tu ne pourras jamais prouver la valeur du projet.

### 26.3 Inventaire des données

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-67** (onglet `P8_Project_Mgmt`)

Pour chaque source : propriétaire, format, mode d'accès (API, fichier, base de données), fréquence, heure de disponibilité, historique disponible, qualité, **contact IT**.

---

## 27. Travailler avec le Sponsor

### 27.1 Qui est le sponsor ?

Probablement un responsable des opérations CA / PB, ou de l'ITO **[À VÉRIFIER]**. Il **finance**, **arbitre** et **porte** le projet. Il parle en **risques, coûts, FTE, incidents, clients, audit**, pas en technologie.

### 27.2 Questions à poser au sponsor (première réunion)

1. Quel est le **problème n°1** que tu veux résoudre ? Qu'est-ce qui t'empêche de dormir (pertes, élections ratées, plaintes clients, audit) ?
2. À quoi ressemble le **succès dans 3 mois** ? Et dans 12 mois ?
3. Comment mesures-tu la réussite : FTE libérés, réduction des breaks, délai de détection, pertes évitées ?
4. Qui sont les **utilisateurs clés** et les **experts** (SMEs) que je peux mobiliser ? Combien de leur temps ?
5. Quel est le **périmètre** prioritaire (types d'événements, classes d'actifs, régions) ?
6. Y a-t-il des **projets en cours** qui recoupent celui-ci (nouveau système CA, migration Broadridge, programme stratégique) ?
7. Quelles sont les **contraintes** : budget, deadlines réglementaires, audit, gel des changements (*freeze periods*) ?
8. Comment veux-tu être tenu informé (fréquence, format) ? Faut-il un **steering committee** ?
9. Qui décide en cas d'arbitrage entre les besoins de plusieurs équipes ?
10. Quels sont les **risques politiques** (équipes inquiètes de l'automatisation, IT centrale qui préfère une autre solution) ?

### 27.3 Le **Project Charter** (1 page, à faire valider)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-68** (onglet `P8_Project_Mgmt`)

Voir le template au §31.1.

### 27.4 Communication avec le sponsor

- **Weekly status** court (template au §31.4) : RAG, avancement, prochaines étapes, **décisions requises**, risques.
- **Steerco mensuel** : bénéfices obtenus, roadmap, arbitrages.
- **Règle d'or :** **pas de surprise**. Une mauvaise nouvelle annoncée tôt avec un plan d'action renforce la confiance ; découverte tard, elle la détruit.
- Parle en **impact** : « Le rapport de breaks paiements est en prod : il fait gagner 1h30/jour à l'équipe Income et a détecté 3 dividendes impayés pour 45 000 $ la première semaine. »

---

## 28. Travailler avec l'IT

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-69** (onglet `P8_Project_Mgmt`)

### 28.1 Les interlocuteurs IT probables

| Rôle | Pourquoi tu en as besoin |
|---|---|
| Owners des systèmes source (système CA, stock record, lending, swaps) | Accès API/DB/fichiers, documentation des données |
| Équipe plateforme Dagster / data | Environnements, déploiement, standards |
| Infrastructure / Middleware | Partages réseau, SFTP, MQ, comptes de service |
| Sécurité / IAM | Habilitations, secrets, chiffrement |
| Production support / Run | Monitoring, astreintes, incidents |
| Architecture | Validation de la solution (*design review*) |
| Release / Change management | Mise en production (tickets, CAB) |

### 28.2 Questions à poser à l'IT

**Données**
1. Quelles sont les sources disponibles pour : événements CA, positions (stock record), paiements, lending, swaps, statut fiscal client ?
2. Mode d'accès pour chacune : API REST, base de données (lecture via vue/réplica ?), fichiers déposés (où, quand, quel format) ?
3. Existe-t-il un **data lake / data warehouse** qui expose déjà ces données ?
4. Y a-t-il un **dictionnaire de données** ? Qui sont les data owners ?
5. Quelle est la latence des données (temps réel, J-1, intraday) ?
6. Y a-t-il des **environnements de test** avec des données réalistes (anonymisées ?) ?

**Plateforme Dagster**
7. Dagster OSS ou **Dagster+** ? Quelle version ? Qui administre ?
8. Environnements DEV / UAT / PROD ? Comment se fait le déploiement (CI/CD, Git, revue de code) ?
9. Où tournent les jobs (Kubernetes, VM, on-prem) ? Accès réseau aux sources ?
10. Comment sont gérés les **secrets** (vault, variables d'environnement) ?
11. Standards existants : structure de code, librairies autorisées, logging, tests ?

**Exploitation**
12. Qui supporte en production (L1/L2/L3) ? Faut-il un **runbook** ?
13. Monitoring et alerting : quel outil, quels seuils ?
14. Processus de **change** : délai, CAB, périodes de gel ?

**Outputs**
15. Envoi d'emails : relais SMTP interne, adresses autorisées, pièces jointes autorisées ?
16. Où déposer les fichiers Excel (SharePoint, partage réseau) ?

### 28.3 Conseils

- **Demande les accès dès la semaine 1.** Les habilitations en banque peuvent prendre **plusieurs semaines** : c'est souvent le chemin critique.
- Prévois des **comptes de service** (pas tes identifiants personnels) pour la prod.
- Documente les **interfaces** (contrat de données : schéma, fréquence, heure, propriétaire).
- Fais valider ton architecture tôt (*design review*) : tu évites un refus au moment du go-live.

---

## 29. Travailler avec les Ops (SMEs)

- Les ops sont **experts** et **occupés**. Respecte leur temps : sessions courtes, préparées, à des heures calmes (pas autour des deadlines du jour).
- Fais-les **co-concevoir** l'output : montre une maquette Excel dès la première semaine et itère.
- **Peur de l'automatisation :** présente l'outil comme un moyen de **supprimer les tâches à faible valeur** pour qu'ils se concentrent sur l'investigation et les clients.
- Demande-leur des **exemples réels de breaks** (avec leur résolution). Ils deviendront tes **cas de test**.
- Identifie un **champion** dans chaque équipe : il testera, donnera son feedback et défendra l'outil auprès de ses collègues.
- **Parallel run** obligatoire : l'outil doit trouver **au moins** les mêmes breaks que le process manuel avant de remplacer quoi que ce soit.

---

## 30. Gouvernance en banque : contrôles, EUC, data, audit

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-70** (onglet `P8_Project_Mgmt`)

| Sujet | Ce que ça implique pour toi |
|---|---|
| **EUC** (*End-User Computing*) | Les macros et fichiers Excel critiques sont un risque identifié (inventaire EUC). **Remplacer un EUC par un pipeline contrôlé** est un argument fort pour ton business case. Il faudra aussi **décommissionner** l'ancien EUC. |
| **SDLC controls** | Code versionné (Git), revue de code, tests, séparation DEV/PROD, traçabilité des déploiements |
| **Data confidentiality** | Les données de positions des hedge funds sont **très confidentielles**. Diffusion des rapports sur la base du *need-to-know*. Attention aux emails externes. |
| **Accès** | Principe du moindre privilège, revues d'accès périodiques |
| **Audit trail** | Conserve les inputs, les outputs et les versions des règles de chaque run (Dagster aide : métadonnées, historique des runs) |
| **Contrôles** | Si ton rapport devient un **contrôle clé** (*key control*), il sera testé par l'audit/le risque ops : documentation, preuves d'exécution, gestion des exceptions |
| **Rétention** | Durée de conservation des rapports et des données (politique interne) |
| **Modèles / règles métier** | Documente chaque règle (tolérances, mapping) avec son **owner métier** et sa date de validation |

---

## 31. Templates prêts à l'emploi

### 31.1 Project Charter (1 page)

```markdown
# Project Charter — CA Breaks Automation (Prime Brokerage US)

**Sponsor :** [Nom, titre]
**PM / Lead Dev :** [Toi]
**Date :** [ ]  **Version :** 0.1

## Problème
Les équipes CA PB US détectent manuellement les écarts (événements, positions,
paiements, taxes, élections) via des extractions et des fichiers Excel.
Ce processus prend [X h/jour], est sujet aux erreurs et détecte les breaks tardivement.

## Objectifs (SMART)
1. Automatiser la détection des breaks de paiement (cash) sur les dividendes US d'ici [date].
2. Réduire le temps de recon manuel de [X]% d'ici [date].
3. Alerter sur 100 % des événements volontaires avec élection manquante à J-3.

## Périmètre
- IN : US, événements DVCA/INTR/…, comptes PB, outputs Excel + email
- OUT (phase 1) : non-US, proxy, reporting fiscal

## Livrables
- Pipelines Dagster en production, rapports quotidiens, runbook, documentation des règles

## Parties prenantes
Sponsor, Ops CA (Income, Reorg, Claims), IT (owners des sources, plateforme Dagster), Tax, Risk/Control

## Jalons
M1 Discovery terminée | M2 MVP en UAT | M3 Go-live UC1 | M4 UC2…

## Bénéfices attendus
FTE libérés, breaks détectés plus tôt (J0 vs J+3), réduction des pertes, suppression d'EUC

## Risques principaux
Accès aux données, qualité des données, disponibilité des SMEs, priorités IT

## Hypothèses / Dépendances
[ ]
```

### 31.2 RACI (exemple)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-71** (onglet `P8_Project_Mgmt`)

| Activité | Sponsor | PM/Dev (toi) | Ops SMEs | IT Sources | IT Plateforme | Risk/Control |
|---|---|---|---|---|---|---|
| Priorisation des use cases | **A** | R | C | I | I | I |
| Règles métier | I | R | **A** | I | — | C |
| Accès aux données | I | R | I | **A** | C | — |
| Développement | I | **A/R** | C | C | C | — |
| UAT | I | R | **A** | — | — | I |
| Mise en production | I | R | I | C | **A** | I |
| Run / Support | I | C | I | — | **A** | — |
| Validation des contrôles | I | C | C | — | — | **A** |

*R = Responsible, A = Accountable, C = Consulted, I = Informed*

### 31.3 RAID Log

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-72** (onglet `P8_Project_Mgmt`)

| ID | Type | Description | Impact | Proba | Owner | Action | Statut |
|---|---|---|---|---|---|---|---|
| R1 | **Risk** | Accès à la base de positions retardé | Élevé | Moyenne | Toi | Demande d'accès en S1, escalade sponsor si > 3 sem. | Ouvert |
| A1 | **Assumption** | Le flux Bloomberg Data License CA est déjà disponible | — | — | IT | À confirmer | Ouvert |
| I1 | **Issue** | Format du fichier DTC différent entre UAT et PROD | Moyen | — | IT | — | Ouvert |
| D1 | **Dependency** | Projet de migration du système CA | Élevé | — | Sponsor | Synchroniser les roadmaps | Ouvert |

### 31.4 Weekly Status Report

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-73** (onglet `P8_Project_Mgmt`)

```markdown
# Status — CA Breaks Automation — Semaine du [ ]

**Statut global : 🟢 / 🟡 / 🔴**   (Périmètre 🟢 | Planning 🟡 | Ressources 🟢)

## Faits marquants
- UC1 (cash breaks DVCA) : en UAT avec l'équipe Income, 2 bugs mineurs corrigés.
- Accès API positions obtenu.

## Prochaines étapes
- Parallel run UC1 du [ ] au [ ].
- Atelier règles métier UC2 (élections) avec la Voluntary Reorg.

## Décisions requises
- Valider la tolérance d'arrondi à 1 $ par événement (Ops Income).

## Risques / Points d'attention
- Disponibilité des SMEs réduite en décembre (fin d'année, volume élevé).

## Bénéfices à date
- [X] breaks détectés automatiquement, [Y] h économisées.
```

### 31.5 User story (format)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-74** (onglet `P8_Project_Mgmt`)

```
En tant qu'analyste Income CA,
je veux recevoir chaque jour à 10h un fichier Excel listant les dividendes US
dont le paiement reçu diffère du montant attendu au-delà de 1 $,
afin de lancer les investigations avant que les clients ne nous contactent.

Critères d'acceptation :
- Tous les événements avec pay date = J-1 ou J sont couverts.
- Chaque ligne contient : CUSIP, nom, type, pay date, compte, attendu, reçu, écart, catégorie, aging.
- Les écarts < 1 $ sont exclus mais comptés dans un onglet de synthèse.
- Si les données sources ne sont pas arrivées à 9h45, un email d'alerte est envoyé.
```

### 31.6 Fiche « règle métier » (à faire signer par les ops)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-75** (onglet `P8_Project_Mgmt`)

```
Règle : CASH_TOLERANCE_USD
Description : écart cash ignoré si |attendu - reçu| ≤ 1.00 USD par événement × compte
Owner métier : [Nom, équipe]
Validée le : [date]
Raison : arrondis DTC sur grosses positions
Revue : annuelle
```

---

## 32. Backlog de use cases priorisé

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-76** (onglet `P8_Project_Mgmt`)

| # | Use case | Famille de breaks | Valeur | Complexité | Proposition |
|---|---|---|---|---|---|
| UC1 | **Cash payment recon** (expected vs received) sur dividendes et coupons US | B5, B14 | Élevée | Faible/Moyenne | **MVP** |
| UC2 | **Election & deadline monitor** (événements volontaires, réponses clients, positions disponibles, protects) | B12, B13 | **Très élevée** (risque) | Moyenne | **MVP ou Phase 2** |
| UC3 | **Event data recon** (Bloomberg vs DTC vs système interne) | B1, B2 | Élevée (prévention) | Moyenne | Phase 2 |
| UC4 | **Position recon à la record date** (stock record vs DTC) | B3 | Élevée | Moyenne/Élevée | Phase 2 |
| UC5 | **Tax rate check** (taux appliqué vs statut fiscal) | B7 | Moyenne/Élevée | Moyenne | Phase 3 |
| UC6 | **Short dividend & lending manufactured payments recon** | B9, B10 | Élevée en PB | Élevée | Phase 3 |
| UC7 | **Swap vs hedge dividend recon** | B11 | Élevée en PB synthétique | Élevée | Phase 3 |
| UC8 | **Claims aging dashboard** | B15 | Moyenne | Faible | Quick win |
| UC9 | **Reference data / CUSIP change check** | B16 | Moyenne | Faible | Quick win |
| UC10 | **MBS factor recon** | B5 (FI) | Moyenne | Moyenne | Phase 3 |

**Méthode de priorisation :** score = (heures économisées + risque évité + visibilité) / effort. Fais valider la matrice par le sponsor.

---

## 33. Pièges classiques

1. **Automatiser un processus mal compris.** Fais du shadowing avant de coder.
2. **Commencer par une plateforme générique.** Livre d'abord un use case concret.
3. **Sous-estimer l'accès aux données.** C'est souvent 50 % du délai.
4. **Ignorer les règles implicites des ops** (« on ignore toujours ce custodian le lundi »). Fais-les **expliciter et documenter**.
5. **Trop de faux positifs.** Un rapport avec 500 lignes dont 480 non pertinentes sera abandonné en 2 semaines. Travaille les tolérances et l'auto-classification.
6. **Pas de baseline.** Tu ne pourras pas prouver les gains.
7. **Pas de parallel run.** Risque de rater des breaks à la bascule.
8. **Pas de plan de run.** Qui regarde quand le job échoue à 7h ?
9. **Ne pas historiser.** Tu dois pouvoir dire : « le break existait le 12/11, il a été résolu le 15/11 ».
10. **Oublier les jours fériés US** et les fins de mois (volumes, calendrier des paiements).

---

# PARTIE IX — IMPLÉMENTATION DAGSTER

## 34. Architecture cible

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-77** (onglet `P9_Dagster`)

```
        SOURCES                         DAGSTER                                  OUTPUTS
┌──────────────────────┐   ┌──────────────────────────────────────────┐   ┌──────────────────┐
│ Bloomberg DL (SFTP)  │──▶│ RAW (ingestion, 1 asset par source)      │   │ Excel breaks     │
│ DTC (fichiers/API)   │──▶│   - sensors fichiers / schedules API      │   │ Email récap      │
│ Broadridge (extract) │──▶│                                           │   │ Table historique │
│ Système CA interne   │──▶│ STAGING (normalisation → modèle canonique)│──▶│ (DB / parquet)   │
│ Positions / stock rec│──▶│   - asset checks qualité                  │   │ Dashboard (plus  │
│ Cash / nostro        │──▶│                                           │   │ tard)            │
│ Lending / Swaps      │──▶│ CORE (golden events, expected entitlements│   └──────────────────┘
│ Statut fiscal client │──▶│        received payments)                │
└──────────────────────┘   │ RECON (matching, breaks, catégorisation)  │
                           │ REPORT (excel, email, historisation)      │
                           └──────────────────────────────────────────┘
```

**Principes :**

- **1 asset = 1 jeu de données** ; découpage en couches `raw → staging → core → recon → report`.
- **Partitions journalières** (business date) : relancer un jour précis, faire un backfill.
- **Idempotence** : relancer un run donne le même résultat.
- **Règles métier en configuration** (YAML/table) et non en dur dans le code.
- **Historisation des breaks** (statut, première détection, résolution) pour l'aging.
- **Asset checks** pour bloquer un rapport construit sur des données incomplètes.

---

## 35. Modèle de données canonique

### 35.1 `ca_event` (golden copy simplifiée)

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-78** (onglet `P9_Dagster`)

| Champ | Type | Exemple |
|---|---|---|
| event_key | str | `US0378331005|DVCA|2026-11-13` |
| source | str | `BBG` / `DTC` / `INTERNAL` |
| source_event_id | str | ID Bloomberg / DTC / interne |
| coaf | str | Référence officielle si disponible |
| cusip / isin | str | |
| caev | str | `DVCA` |
| camv | str | `MAND` |
| ex_date / record_date / pay_date | date | |
| currency | str | `USD` |
| gross_rate | decimal | 0.25 |
| ratio_new / ratio_old | decimal | 4 / 1 |
| status | str | `PREU` / `COMP` / `CANC` |
| version_ts | timestamp | horodatage de la version |

### 35.2 `position_snapshot`

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-79** (onglet `P9_Dagster`)

| Champ | Exemple |
|---|---|
| business_date, cusip, account_id, account_type (`LONG`/`SHORT`/`LOAN`/`BORROW`/`SWAP_HEDGE`/`FIRM`), quantity, location (`DTC`, `FED`, custodian), tax_status, treaty_rate |

### 35.3 `expected_entitlement`

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-80** (onglet `P9_Dagster`)

| Champ | Exemple |
|---|---|
| event_key, account_id, quantity, gross_amount, tax_rate, tax_amount, net_amount, currency, payment_type (`DIVIDEND`/`SUBSTITUTE`/`SHORT_CHARGE`/`MANUFACTURED`) |

### 35.4 `received_payment`

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-81** (onglet `P9_Dagster`)

| Champ | Exemple |
|---|---|
| source (`DTC`/`MT566`/`NOSTRO`), source_ref, event_key (ou cusip + date), account_id, gross, tax, net, currency, value_date |

### 35.5 `ca_break`

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-82** (onglet `P9_Dagster`)

| Champ | Exemple |
|---|---|
| break_id (hash stable), break_type (`B5_CASH`…), event_key, account_id, expected, actual, diff, diff_usd, category (`UNPAID`/`TAX_RATE`/`POSITION`/`ROUNDING`/`UNKNOWN`), first_seen_date, last_seen_date, aging_days, status (`OPEN`/`RESOLVED`), comment |

> Le **break_id stable** (hash de `break_type + event_key + account_id`) permet de suivre un break d'un jour à l'autre et de calculer l'aging.

---

## 36. Squelette de code Dagster

> Code illustratif, à adapter aux sources et aux standards internes. Les noms de systèmes sont fictifs.

### 36.1 Structure du projet

```
ca_breaks/
├── pyproject.toml
├── ca_breaks/
│   ├── definitions.py
│   ├── resources.py          # connexions API / DB / SMTP / file system
│   ├── partitions.py
│   ├── config/
│   │   └── rules.yaml        # tolérances, mappings, destinataires
│   ├── assets/
│   │   ├── raw.py            # ingestion brute
│   │   ├── staging.py        # normalisation
│   │   ├── core.py           # golden events, expected, received
│   │   ├── recon.py          # matching + breaks
│   │   └── report.py         # excel + email + historique
│   ├── checks.py             # asset checks
│   ├── sensors.py            # sensors fichiers
│   └── lib/
│       ├── matching.py       # moteur de matching (pur Python, testable)
│       ├── categorize.py
│       └── calendar.py       # jours ouvrés US
└── tests/
    ├── fixtures/             # fichiers anonymisés de vrais cas
    └── test_matching.py
```

### 36.2 Partitions et ressources

```python
# partitions.py
import dagster as dg

business_days = dg.DailyPartitionsDefinition(
    start_date="2026-01-01",
    timezone="America/New_York",
)
```

```python
# resources.py
import dagster as dg
import pandas as pd
import requests


class PositionsApi(dg.ConfigurableResource):
    base_url: str
    token: str  # injecté depuis le gestionnaire de secrets, jamais en dur

    def get_positions(self, as_of: str) -> pd.DataFrame:
        r = requests.get(
            f"{self.base_url}/positions",
            params={"asOf": as_of},
            headers={"Authorization": f"Bearer {self.token}"},
            timeout=60,
        )
        r.raise_for_status()
        return pd.DataFrame(r.json()["items"])


class DropFolder(dg.ConfigurableResource):
    root: str  # ex: /mnt/shares/ca/inbound

    def path_for(self, source: str, date: str) -> str:
        return f"{self.root}/{source}/{date}"


class Mailer(dg.ConfigurableResource):
    smtp_host: str
    sender: str

    def send(self, to: list[str], subject: str, html: str, attachments: list[str]):
        import smtplib
        from email.message import EmailMessage
        from pathlib import Path

        msg = EmailMessage()
        msg["From"], msg["To"], msg["Subject"] = self.sender, ", ".join(to), subject
        msg.set_content("Voir la version HTML.")
        msg.add_alternative(html, subtype="html")
        for p in attachments:
            msg.add_attachment(
                Path(p).read_bytes(),
                maintype="application",
                subtype="vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                filename=Path(p).name,
            )
        with smtplib.SMTP(self.smtp_host) as s:
            s.send_message(msg)
```

### 36.3 Ingestion et normalisation

```python
# assets/raw.py + staging.py (condensé)
import dagster as dg
import pandas as pd
from ..partitions import business_days
from ..resources import DropFolder, PositionsApi


@dg.asset(partitions_def=business_days, group_name="raw")
def raw_dtc_payments(context: dg.AssetExecutionContext, drop: DropFolder) -> pd.DataFrame:
    date = context.partition_key
    df = pd.read_csv(f"{drop.path_for('dtc', date)}/payments.csv", dtype=str)
    context.add_output_metadata({"rows": len(df)})
    return df


@dg.asset(partitions_def=business_days, group_name="raw")
def raw_positions(context: dg.AssetExecutionContext, positions: PositionsApi) -> pd.DataFrame:
    return positions.get_positions(as_of=context.partition_key)


@dg.asset(partitions_def=business_days, group_name="staging")
def stg_received_payments(raw_dtc_payments: pd.DataFrame) -> pd.DataFrame:
    """Mapping des colonnes source vers le modèle canonique."""
    df = raw_dtc_payments.rename(columns={
        "CUSIP": "cusip", "EVENT_TYPE": "caev", "PAY_DT": "pay_date",
        "PARTICIPANT_ACCT": "account_id", "GROSS_AMT": "gross",
        "TAX_AMT": "tax", "NET_AMT": "net",
    })
    for c in ["gross", "tax", "net"]:
        df[c] = pd.to_numeric(df[c], errors="coerce").fillna(0.0)
    df["pay_date"] = pd.to_datetime(df["pay_date"]).dt.date.astype(str)
    df["event_key"] = df["cusip"] + "|" + df["caev"] + "|" + df["pay_date"]
    return df[["event_key", "cusip", "caev", "pay_date", "account_id", "gross", "tax", "net"]]
```

### 36.4 Expected entitlements

```python
# assets/core.py (condensé)
import dagster as dg
import pandas as pd
from ..partitions import business_days


@dg.asset(partitions_def=business_days, group_name="core")
def expected_entitlements(
    golden_events: pd.DataFrame,          # événements validés (pay_date = partition)
    stg_positions_at_record: pd.DataFrame # positions à la record date de chaque event
) -> pd.DataFrame:
    df = golden_events.merge(stg_positions_at_record, on=["cusip", "record_date"], how="inner")
    df["gross_expected"] = (df["quantity"] * df["gross_rate"]).round(2)
    df["tax_expected"] = (df["gross_expected"] * df["tax_rate"]).round(2)
    df["net_expected"] = df["gross_expected"] - df["tax_expected"]
    return (
        df.groupby(["event_key", "account_id"], as_index=False)
          [["quantity", "gross_expected", "tax_expected", "net_expected"]].sum()
    )
```

### 36.5 Recon et asset checks

```python
# assets/recon.py
import dagster as dg
import pandas as pd
from ..partitions import business_days
from ..lib.matching import reconcile_cash


class ReconConfig(dg.Config):
    tolerance_usd: float = 1.0


@dg.asset(partitions_def=business_days, group_name="recon")
def cash_breaks(
    context: dg.AssetExecutionContext,
    config: ReconConfig,
    expected_entitlements: pd.DataFrame,
    stg_received_payments: pd.DataFrame,
) -> pd.DataFrame:
    breaks = reconcile_cash(
        expected_entitlements, stg_received_payments,
        tolerance=config.tolerance_usd, business_date=context.partition_key,
    )
    context.add_output_metadata({
        "breaks": len(breaks),
        "total_abs_diff_usd": float(breaks["diff"].abs().sum()) if len(breaks) else 0.0,
    })
    return breaks


# checks.py
@dg.asset_check(asset="stg_received_payments", blocking=True)
def payments_not_empty(stg_received_payments: pd.DataFrame) -> dg.AssetCheckResult:
    return dg.AssetCheckResult(
        passed=len(stg_received_payments) > 0,
        description="Le fichier de paiements DTC ne doit pas être vide un jour ouvré",
    )


@dg.asset_check(asset="stg_received_payments")
def no_null_cusip(stg_received_payments: pd.DataFrame) -> dg.AssetCheckResult:
    n = int(stg_received_payments["cusip"].isna().sum())
    return dg.AssetCheckResult(passed=n == 0, metadata={"null_cusips": n})
```

### 36.6 Sensor sur fichier déposé + schedule

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-83** (onglet `P9_Dagster`)

```python
# sensors.py
import os
import dagster as dg

daily_recon_job = dg.define_asset_job(
    "daily_cash_recon",
    selection=dg.AssetSelection.groups("raw", "staging", "core", "recon", "report"),
    partitions_def=business_days,
)


@dg.sensor(job=daily_recon_job, minimum_interval_seconds=120)
def dtc_payments_file_sensor(context: dg.SensorEvaluationContext):
    from datetime import date
    today = date.today().isoformat()
    path = f"/mnt/shares/ca/inbound/dtc/{today}/payments.csv"
    done_flag = path + ".done"  # convention : le producteur dépose un fichier .done
    if os.path.exists(done_flag):
        yield dg.RunRequest(run_key=f"dtc-{today}", partition_key=today)
    else:
        yield dg.SkipReason(f"En attente de {done_flag}")
```

> **Bonnes pratiques fichiers :** attendre un **flag de fin d'écriture** (`.done`, ou taille stable) avant de lire ; archiver les fichiers traités ; ne jamais supprimer un input.

### 36.7 Definitions

```python
# definitions.py
import dagster as dg
from .assets import raw, staging, core, recon, report
from . import checks, sensors
from .resources import PositionsApi, DropFolder, Mailer

defs = dg.Definitions(
    assets=dg.load_assets_from_modules([raw, staging, core, recon, report]),
    asset_checks=[checks.payments_not_empty, checks.no_null_cusip],
    jobs=[sensors.daily_recon_job],
    sensors=[sensors.dtc_payments_file_sensor],
    resources={
        "positions": PositionsApi(
            base_url=dg.EnvVar("POSITIONS_API_URL"), token=dg.EnvVar("POSITIONS_API_TOKEN")
        ),
        "drop": DropFolder(root=dg.EnvVar("CA_DROP_ROOT")),
        "mailer": Mailer(smtp_host=dg.EnvVar("SMTP_HOST"), sender=dg.EnvVar("MAIL_SENDER")),
    },
)
```

---

## 37. Le moteur de matching

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-84** (onglet `P9_Dagster`)

À garder en **pur Python/pandas** (sans Dagster) pour le tester facilement.

```python
# lib/matching.py
import hashlib
import pandas as pd

KEYS = ["event_key", "account_id"]


def _break_id(row) -> str:
    raw = f"CASH|{row['event_key']}|{row['account_id']}"
    return hashlib.sha1(raw.encode()).hexdigest()[:16]


def categorize(row) -> str:
    exp, rec = row["net_expected"], row["net"]
    if row["_merge"] == "left_only" or rec == 0:
        return "UNPAID"
    if row["_merge"] == "right_only":
        return "UNEXPECTED_PAYMENT"
    if row["gross_expected"] and abs(row["gross"] - row["gross_expected"]) <= 0.01:
        return "TAX_DIFFERENCE"          # même brut, net différent → taxe
    if row["gross_expected"]:
        ratio = row["gross"] / row["gross_expected"]
        if 0 < ratio < 1:
            return "PARTIAL_OR_POSITION"
        if ratio > 1:
            return "OVERPAID_OR_POSITION"
    return "UNKNOWN"


def reconcile_cash(expected: pd.DataFrame, received: pd.DataFrame,
                   tolerance: float, business_date: str) -> pd.DataFrame:
    rec = received.groupby(KEYS, as_index=False)[["gross", "tax", "net"]].sum()  # gère les paiements partiels
    m = expected.merge(rec, on=KEYS, how="outer", indicator=True)
    for c in ["gross_expected", "tax_expected", "net_expected", "gross", "tax", "net"]:
        m[c] = m[c].fillna(0.0)
    m["diff"] = (m["net"] - m["net_expected"]).round(2)
    breaks = m[m["diff"].abs() > tolerance].copy()
    breaks["category"] = breaks.apply(categorize, axis=1)
    breaks["break_id"] = breaks.apply(_break_id, axis=1)
    breaks["business_date"] = business_date
    return breaks.drop(columns="_merge")
```

### 37.1 Historisation et aging

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-85** (onglet `P9_Dagster`)

Chaque jour, on fusionne les breaks du jour avec la table historique :

- Break présent hier **et** aujourd'hui → `last_seen_date = today`, l'aging augmente.
- Break présent hier mais **pas** aujourd'hui → `status = RESOLVED`, `resolved_date = today`.
- Nouveau break → `first_seen_date = today`.

```python
def update_history(history: pd.DataFrame, today_breaks: pd.DataFrame, today: str) -> pd.DataFrame:
    open_hist = history[history["status"] == "OPEN"]
    still_open = open_hist["break_id"].isin(today_breaks["break_id"])
    history.loc[open_hist[~still_open].index, ["status", "resolved_date"]] = ["RESOLVED", today]

    new = today_breaks[~today_breaks["break_id"].isin(history["break_id"])].assign(
        first_seen_date=today, status="OPEN", resolved_date=None
    )
    history = pd.concat([history, new], ignore_index=True)
    history.loc[history["break_id"].isin(today_breaks["break_id"]), "last_seen_date"] = today
    history["aging_days"] = (
        pd.to_datetime(history["last_seen_date"]) - pd.to_datetime(history["first_seen_date"])
    ).dt.days
    return history
```

> Utilise idéalement une **base de données** (ou des fichiers parquet versionnés) pour l'historique, pas un Excel.

---

## 38. Outputs : Excel et email

### 38.1 Structure recommandée du fichier Excel

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-86** (onglet `P9_Dagster`)

| Onglet | Contenu |
|---|---|
| **Summary** | Nombre et montant des breaks par catégorie et par aging, comparaison avec la veille |
| **New breaks** | Nouveaux breaks du jour (priorité d'investigation) |
| **Open breaks** | Tous les breaks ouverts, triés par montant USD décroissant |
| **Resolved** | Breaks résolus depuis hier (preuve du suivi) |
| **Data quality** | Résultats des asset checks, fichiers manquants, lignes rejetées |
| **Parameters** | Tolérances, sources, horodatages des données (traçabilité audit) |

### 38.2 Génération (openpyxl via pandas)

```python
# assets/report.py (condensé)
import dagster as dg
import pandas as pd


@dg.asset(partitions_def=business_days, group_name="report")
def cash_breaks_report(context, cash_breaks_history: pd.DataFrame, mailer: Mailer) -> str:
    d = context.partition_key
    path = f"/mnt/shares/ca/outbound/cash_breaks_{d}.xlsx"
    open_b = cash_breaks_history[cash_breaks_history.status == "OPEN"]
    new_b = open_b[open_b.first_seen_date == d]
    summary = open_b.groupby("category").agg(n=("break_id", "count"), usd=("diff", "sum")).reset_index()

    with pd.ExcelWriter(path, engine="openpyxl") as xw:
        summary.to_excel(xw, "Summary", index=False)
        new_b.to_excel(xw, "New breaks", index=False)
        open_b.sort_values("diff", key=abs, ascending=False).to_excel(xw, "Open breaks", index=False)
        for ws in xw.book.worksheets:                 # mise en forme minimale
            ws.freeze_panes = "A2"
            ws.auto_filter.ref = ws.dimensions

    html = f"""
    <p>CA Cash Breaks — {d}</p>
    <ul>
      <li>Nouveaux breaks : <b>{len(new_b)}</b></li>
      <li>Breaks ouverts : <b>{len(open_b)}</b> ({open_b['diff'].abs().sum():,.2f} USD)</li>
      <li>Ouverts &gt; 30 jours : <b>{(open_b.aging_days > 30).sum()}</b></li>
    </ul>"""
    mailer.send(to=["ca-income-ny@…"], subject=f"[CA] Cash breaks {d}", html=html, attachments=[path])
    return path
```

> **Confidentialité :** destinataires internes uniquement, liste gérée en configuration et validée par le sponsor. Si la politique interne l'exige, mettre un **lien** vers le partage plutôt qu'une pièce jointe **[À VÉRIFIER]**.

---

## 39. Tests, qualité, déploiement, run

### 39.1 Tests

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-87** (onglet `P9_Dagster`)

- **Unit tests** sur `lib/` (matching, catégorisation, calcul d'entitlement) avec les **cas pratiques de la Partie VII** comme fixtures.
- **Golden tests** : un jour réel (anonymisé) → la sortie attendue est validée par les ops et figée.
- **Tests de régression** à chaque changement de règle.
- Tests Dagster : `dg.materialize([...], resources={...})` avec des ressources mockées.

```python
def test_tax_difference_detected():
    exp = pd.DataFrame([{"event_key": "E1", "account_id": "A", "quantity": 100_000,
                         "gross_expected": 50_000, "tax_expected": 7_500, "net_expected": 42_500}])
    rec = pd.DataFrame([{"event_key": "E1", "account_id": "A",
                         "gross": 50_000, "tax": 15_000, "net": 35_000}])
    b = reconcile_cash(exp, rec, tolerance=1.0, business_date="2026-11-13")
    assert b.iloc[0]["category"] == "TAX_DIFFERENCE"
    assert b.iloc[0]["diff"] == -7_500
```

### 39.2 Qualité des données (asset checks recommandés)

- Fichier présent et non vide un jour ouvré
- Nombre de lignes dans une fourchette habituelle (± X % vs moyenne des 20 derniers jours)
- Aucun CUSIP nul, aucun montant non numérique
- Toutes les positions ont un statut fiscal
- **Freshness** : données du bon jour (pas le fichier d'hier)

### 39.3 Déploiement

- Git + revue de code + CI (lint `ruff`, tests `pytest`) + déploiement par environnement.
- Configuration par environnement (DEV/UAT/PROD) via variables d'environnement et secrets.
- **Versionner les règles** (`rules.yaml`) et tracer leur version dans les métadonnées des runs.

### 39.4 Run

- **Alerting** sur l'échec d'un run ou d'un asset check bloquant (email/Teams vers le support et vers toi).
- **Runbook :** que faire si le fichier DTC n'arrive pas, comment relancer une partition, qui contacter.
- **SLA :** rapport disponible avant [heure] ; en cas d'échec, email de « rapport retardé » aux ops.
- **Backfills :** les partitions permettent de régénérer un historique ou de rejouer après correction.

---

# ANNEXES

## Annexe A — Glossaire EN → FR

| Terme | Signification |
|---|---|
| **Accrual** | Comptabilisation d'un revenu à recevoir/payer avant le paiement |
| **ADR** | American Depositary Receipt : certificat US sur une action étrangère |
| **Agent (paying/transfer/exchange)** | Intermédiaire de l'émetteur |
| **Aging** | Ancienneté d'un break ou d'un claim |
| **ATOP** | Automated Tender Offer Program : système DTC pour les tenders et offres d'échange |
| **Backup withholding** | Retenue US (24 %) en cas de documentation fiscale manquante |
| **Break** | Écart de réconciliation |
| **Buy-in** | Rachat forcé de titres non livrés |
| **Cash in lieu (CIL)** | Paiement cash à la place des fractions d'actions |
| **Cede & Co.** | Nominee de la DTC |
| **Claim** | Réclamation auprès d'une contrepartie |
| **COAF** | Official Corporate Action Event Reference |
| **Cum / Ex** | Avec / sans le droit |
| **CUSIP** | Identifiant de titre US |
| **Default option** | Option appliquée sans instruction |
| **Due bill** | Obligation de reverser une distribution à l'acheteur (fenêtre US) |
| **DTC / DTCC** | Dépositaire central US / sa holding |
| **Election / Instruction** | Choix du détenteur sur un événement à options |
| **Entitlement** | Ce à quoi un détenteur a droit |
| **EUC** | End-User Computing (Excel, macros…) |
| **Ex-date** | Premier jour de cotation sans le droit |
| **Factor** | Proportion restante du principal (MBS/ABS) |
| **Fail** | Trade non réglé à la date prévue |
| **Golden copy** | Version validée d'un événement |
| **Guaranteed delivery / Protect** | Instruction de tender avec livraison différée des titres |
| **Manufactured payment** | Paiement compensatoire dans un prêt/repo |
| **Market claim** | Réclamation de distribution suite à un trade chevauchant la record date |
| **Merger arbitrage** | Stratégie de hedge fund sur les fusions |
| **NRA** | Non-Resident Alien (régime de retenue US) |
| **OCC** | Options Clearing Corporation |
| **Over-election** | Élection supérieure à la position disponible |
| **Pay date** | Date de paiement |
| **Prime Broker** | Banque prestataire de services pour hedge funds |
| **Proration** | Réduction proportionnelle si une offre est sursouscrite |
| **Proxy** | Vote en assemblée |
| **QI / QDD** | Statuts fiscaux d'intermédiaire / de dealer de dérivés |
| **Recall** | Rappel de titres prêtés |
| **Record date** | Date de photographie des détenteurs |
| **Rehypothecation** | Réutilisation des titres clients par le broker |
| **Reorg** | Réorganisation (mandatory/voluntary) |
| **Scrubbing** | Nettoyage/validation des données d'événement |
| **Stock record** | Grand livre des titres du broker-dealer |
| **STP** | Straight-Through Processing |
| **Substitute payment** | Paiement « en lieu » d'un dividende (titres prêtés) |
| **Tender offer** | Offre publique d'achat |
| **TRS / Portfolio swap** | Swap de performance (synthetic prime) |
| **Treaty rate** | Taux de retenue réduit par une convention fiscale |
| **Withholding tax (WHT)** | Retenue à la source |

## Annexe B — Champs SWIFT/ISO à connaître

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-88** (onglet `Annexes`)

| Champ (MT564/566) | Signification |
|---|---|
| `20C::CORP` | Référence de l'événement (fournisseur) |
| `20C::COAF` | Référence officielle |
| `23G` | Fonction du message (NEWM, REPL, REPE, CANC, WITH) |
| `22F::CAEV` | Type d'événement |
| `22F::CAMV` | MAND / CHOS / VOLU |
| `25D::PROC` | Statut de traitement (PREU, COMP…) |
| `35B` | Identifiant du titre (ISIN + description) |
| `93a::ELIG` / `SETT` | Solde éligible / réglé |
| `98a::XDTE / RDTE / PAYD / EFFD` | Ex / Record / Pay / Effective date |
| `98a::MKDT / RDDT` | Market / Response deadline |
| `92a::GRSS / NETT / WITF / TAXR` | Taux brut / net / taux de retenue |
| `92D::NEWO / ADEX` | Ratio nouveau/ancien / ratio additionnel |
| `13A::CAON` | Numéro d'option |
| `22F::CAOP` | Type d'option |
| `17B::DFLT` | Option par défaut (Y/N) |
| `19B::ENTL / GRSS / NETT / TAXR` | Montants (entitlement, brut, net, taxe) |
| `90a::PRPP / OFFR` | Prix (offre, souscription) |

## Annexe C — Checklist des questions à poser en interne

> 📊 **Exemple(s) Excel** — `CA_PB_US_Exemples.xlsx` : **EX-89** (onglet `Annexes`)

**Organisation**
- [ ] Organigramme des équipes CA PB (NYC, offshore), qui fait quoi ?
- [ ] Qui est le sponsor exact, et qui sont les SMEs par type d'événement ?

**Systèmes**
- [ ] Quel est le système CA principal ? (Broadridge ? Lequel ? Interne ?)
- [ ] D'où viennent les annonces : Bloomberg Data License, DTC ISO 20022, autres ?
- [ ] Quel système porte le stock record ? Le lending ? Les swaps ? Les statuts fiscaux ?
- [ ] Quels outils de réconciliation existent déjà (SmartStream TLM, Duco, maison…) ?

**Processus**
- [ ] Quelles réconciliations sont faites aujourd'hui, à quelle heure, avec quels fichiers ?
- [ ] Quels sont les 10 types de breaks les plus fréquents ? Les plus coûteux ?
- [ ] Quels incidents ou pertes CA ont eu lieu ces 2 dernières années ?
- [ ] Comment sont gérées les élections des clients PB (canal, deadline interne, option par défaut) ?
- [ ] Comment les substitute payments sont-ils alloués ?
- [ ] Comment les queries des fund admins arrivent-elles et sont-elles suivies ?

**Data & IT**
- [ ] Accès aux données (API, DB, fichiers), environnements de test, comptes de service
- [ ] Plateforme Dagster : version, déploiement, standards, support
- [ ] Politique email/partage des données clients

**Contrôles**
- [ ] Les fichiers Excel actuels sont-ils dans l'inventaire EUC ?
- [ ] Le futur rapport sera-t-il un contrôle clé ?

## Annexe D — Plan 30-60-90 jours

| Période | Objectifs PM | Objectifs Dev | Objectifs Apprentissage |
|---|---|---|---|
| **J1–30** | Rencontrer sponsor et équipes, shadowing (≥ 3 demi-journées), mesurer la baseline, rédiger le charter, construire le backlog, demander **tous les accès** | Setup environnement, premier pipeline « hello data » sur une source réelle, prototype de matching UC1 sur données exportées | Maîtriser les Parties I–VI ; vocabulaire ; lire 20 MT564/566 réels |
| **J31–60** | Valider la priorisation, steerco #1, règles métier UC1 signées | UC1 en UAT, Excel co-conçu avec les ops, asset checks, historisation | Approfondir le PB (rehypo, shorts, swaps), la fiscalité |
| **J61–90** | Parallel run UC1, go-live, mesure des premiers bénéfices, lancement UC2 | UC1 en prod (runbook, alerting), design UC2 (élections/deadlines) | Cas complexes : reorgs volontaires, protects, MBS |

## Annexe E — Ressources pour aller plus loin

- **DTCC** : documentation publique sur les services Asset Services / Corporate Actions (CA Web, ISO 20022, ATOP, Claim Connect) — site dtcc.com
- **SMPG** : market practices CA (smpg.info)
- **ISITC** : market practices US (isitc.org)
- **SWIFT** : documentation des catégories MT5xx et des messages ISO 20022 `seev`
- **OCC** : Information Memos sur les ajustements d'options (theocc.com)
- **SEC** : Rule 15c3-3 (Customer Protection), Rule 10b-17
- **FINRA** : Uniform Practice Code (ex-dates, due bills)
- **IRS** : publications sur NRA withholding (Pub. 515), section 871(m), formulaires W-8/W-9, 1042-S, 1099
- **MSRB EMMA** : données munis
- **Dagster** : documentation officielle (assets, partitions, sensors, asset checks, resources) — docs.dagster.io

---

*Document vivant : complète les sections marquées **[À VÉRIFIER EN INTERNE]** au fil de tes rencontres et transforme-le en documentation projet.*

---

## Annexe F — Pont terminologique US (jargon de desk NYC)

> Ajout : le vocabulaire que tu entendras **réellement** au desk et dans les emails à New York, avec sa définition et son équivalent français. Organisé par thème pour faire le pont avec le reste du guide.

### F.1 Élections et événements volontaires

| Terme US | Définition | Équivalent / remarque |
|---|---|---|
| **Voluntary Reorg** (*Vol Reorg*) | Équipe et famille des événements où le porteur choisit (tenders, exchange offers, rights, consents, puts) | Événements `VOLU` / `CHOS` |
| **Mandatory Reorg** (*Mand Reorg*) | Réorganisations sans choix : splits, mergers cash, name/CUSIP changes, calls | Événements `MAND` hors distributions |
| **Election** / **Instruction** | Le choix communiqué par le client (option + quantité) | Côté marché : MT565 / ATOP |
| **Non-election** / **No response** | Le client n'a pas répondu avant la *cutoff* | Déclenche l'option par défaut (CHOS) ou *no action* (VOLU) |
| **Non-electing holders** | Porteurs qui n'ont pas élu. Dans un merger avec choix, ils reçoivent ce qui reste après proration (souvent un mix cash/actions) | À expliquer aux clients : le résultat peut surprendre |
| **Default option** / **Default election** | Option appliquée automatiquement en cas de non-élection | Champ `17B::DFLT//Y` |
| **"We'll default them"** | « On lui applique l'option par défaut » | Phrase de desk |
| **Take no action** / **NOAC** | Choix explicite de ne rien faire | Préférable à un silence : on sait que le client a vu l'événement |
| **Cutoff** / **Client cutoff** / **Internal deadline** | Deadline fixée par le PB à ses clients, avant celle de la DTC | *Client deadline* du guide |
| **DTC cutoff** / **Market deadline** / **Agent deadline** | Deadline de la DTC ou de l'agent | `RDDT` / `MKDT` |
| **Expiration** / **Expiration date** | Fin de l'offre, souvent 17h00 ou 23h59 heure de NY (*"5 p.m. New York City time"*, *"one minute after 11:59 p.m."*) | Toujours vérifier l'heure exacte dans l'*Offer to Purchase* |
| **Extended** / **Extension** | L'offre est prolongée. La nouvelle deadline s'applique et les élections déjà reçues restent généralement valables | Source classique de confusion : relancer les non-répondants |
| **Withdrawal rights** / **Withdraw** | Droit de **retirer** des titres déjà apportés tant que l'offre n'a pas expiré | Le client peut changer d'avis : il faut annuler l'instruction ATOP |
| **Tender** / **Tendered shares** / **Contra CUSIP** | Apporter à l'offre. Les titres apportés sont bloqués à la DTC, souvent sous un **CUSIP « contra »** temporaire | Ils ne peuvent plus être vendus, prêtés ni utilisés comme collatéral |
| **ATOP** | *Automated Tender Offer Program* : système DTC pour instruire les tenders et exchange offers | — |
| **Offer to Purchase** / **Letter of Transmittal** | Documents officiels de l'offre (conditions, prix, deadlines) | La « source primaire » à consulter en cas de doute |
| **Protect** / **NGD** (*Notice of Guaranteed Delivery*) | Instruction d'apport **sans** livrer les titres tout de suite, avec engagement de les livrer avant la **protect deadline** (souvent 1 à 2 jours ouvrés après l'expiration) | Utilisé quand les titres sont prêtés, en fail ou en transit |
| **Cover the protect** / **Protect cover** | Livrer effectivement les titres promis par le protect | — |
| **"We're short the protect"** / **Failed protect** | On n'a pas les titres pour couvrir le protect | Risque d'indemnisation client |
| **Over-election** / **Over-instructed** | Élection supérieure à la position disponible | Break B12 |
| **Short tender** / **Rule 14e-4** | Apporter plus que sa position nette longue (interdit) | Contrôle obligatoire (EX-25) |
| **Proration** / **Prorated** / **Proration factor** | Acceptation partielle quand l'offre est sursouscrite (ex. 72 % accepté) | Le reste est rendu au porteur |
| **Odd lot** / **Odd-lot priority** | Détenteur de moins de 100 actions, parfois exempté de proration | Rare mais existe dans les buybacks |
| **Dutch auction** | Rachat où les porteurs indiquent un prix dans une fourchette, l'émetteur fixe un prix unique de clearing | Événement `BIDS` |
| **Early tender premium** / **Early tender date** | Prime versée aux porteurs obligataires qui apportent avant une date anticipée | Fréquent sur les *debt tenders* |
| **Consent solicitation** / **Consent fee** | Demande de vote aux porteurs obligataires pour modifier le contrat (*indenture*), avec parfois un fee | Événement `CONS` |
| **Rights** / **Subscription** / **Oversubscription privilege** | Droits de souscription ; possibilité de demander des actions en plus | `RHDI` / `EXRI` / `OVER` |
| **Lapse** / **Lapsed rights** | Droits expirés sans exercice : leur valeur est perdue | `LAPS` |
| **Buyer protection** | Mécanisme par lequel l'**acheteur** d'un trade non encore réglé transmet son élection au **vendeur**, qui doit livrer le résultat choisi | Surtout utilisé sur les élections avec des trades en cours |
| **Recall** | Rappel des titres prêtés (pour élire, voter ou vendre) | Equity Finance |
| **Liability** / **Voluntary liability** | Obligation de l'emprunteur de livrer au prêteur le résultat économique de son élection | Prêt de titres |

### F.2 Income (dividendes, coupons, distributions)

| Terme US | Définition | Équivalent / remarque |
|---|---|---|
| **Income** / **Divs** / **Distributions team** | Équipe qui traite les paiements cash et titres sans choix | — |
| **Declared** / **Declaration date** | Date de décision du board | ≈ Announcement date |
| **Ex date** / **"Goes ex"** | « Le titre passe ex-dividende » | `XDTE` |
| **Record date** / **"Holders of record"** | Porteurs inscrits à la record date | `RDTE` |
| **Payable date** / **Pay date** | Date de paiement | `PAYD` |
| **Regular dividend** / **Special dividend** / **Extra dividend** | Dividende récurrent / exceptionnel / complément exceptionnel | Les *specials* peuvent déclencher des ajustements OCC |
| **Due bill** / **Due bill period** / **Interim accounting** | Voir §4.4 : distribution due à l'acheteur quand l'ex-date est après la pay date | Spécifique US |
| **Accrual** / **Receivable** / **Payable** | Revenu à recevoir / à payer comptabilisé à l'ex-date | §18 |
| **Contractual income** / **Contractual settlement** | Le PB crédite le client à la pay date **même si** le marché n'a pas encore payé | Risque : avance de trésorerie |
| **Actual income** / **Actual settlement** | Le client est crédité **à réception** du paiement de marché | — |
| **Unpaid** / **Outstanding** / **Aged** | Paiement attendu non reçu / break qui vieillit | Catégorie UNPAID |
| **Short dividend** / **Short charge** | Dividende débité sur le compte d'un client short | §10.2 |
| **Manufactured dividend** / **Substitute payment** / **Payment in lieu (PIL)** | Paiement compensatoire versé à la place d'un vrai dividende (titres prêtés) | Fiscalité différente |
| **Cash in lieu (CIL)** | Cash à la place des fractions d'actions | — |
| **Paydown** / **Factor** / **P&I** | Remboursement mensuel de principal (MBS/ABS) / facteur de pool / *Principal & Interest* | §12.4 |
| **Day count** / **30/360** / **ACT/360** | Convention de calcul des intérêts | EX-41 |
| **Reset** / **Refix** | Recalcul du taux d'une FRN ou d'un swap | Cause classique de rate break |

### F.3 Réconciliation et gestion des écarts

| Terme US | Définition | Équivalent / remarque |
|---|---|---|
| **Break** | Écart de réconciliation | — |
| **Rec** / **Recon** | Réconciliation | « *Run the rec* » = lancer la recon |
| **Expected vs Actual** / **Expected vs Received** | Montant attendu vs montant réel | Cœur de UC1 |
| **Matched** / **Unmatched** / **Partially matched** | Rapproché / non rapproché / rapproché en partie | — |
| **Aging** / **"Aged break"** / **Aging buckets** | Ancienneté d'un écart et tranches (0-30, 31-60, 61-90, 90+) | EX-53 |
| **Root cause** / **RCA** | Cause racine / *Root Cause Analysis* | Catégorisation automatique |
| **Tolerance** / **Threshold** | Seuil sous lequel un écart est ignoré | À faire valider (EX-75) |
| **Write-off** | Passer un écart en perte | Dernier recours |
| **Claim** / **Market claim** / **Reverse claim** | Réclamation à une contrepartie / inverse | §21 |
| **Fail** / **Fail to deliver (FTD)** / **Fail to receive (FTR)** | Trade non réglé côté livraison / réception | Cause de claims |
| **Stock record** / **Box** / **Seg** | Grand livre des titres / titres en possession / titres ségrégués | §9.4 |
| **Short securities difference** / **Long securities difference** | Écart de titres en moins / en plus au stock record | Coût en capital possible (§10J) |
| **Nostro** / **DDA** (*Demand Deposit Account*) | Compte cash de la banque | Côté cash de la recon |
| **STP** (*Straight-Through Processing*) | Traitement sans intervention manuelle | KPI |
| **Exception** / **Exception queue** | Élément à traiter manuellement | Ce que ton outil doit réduire |
| **Escalation** | Remontée d'un problème au management ou au client | — |

### F.4 Prime Brokerage et financement

| Terme US | Définition | Équivalent / remarque |
|---|---|---|
| **PB** / **Prime** | Prime brokerage | — |
| **Cash prime** vs **Synthetic prime** | Titres physiques vs exposition via swaps | §10G |
| **Delta One** | Desk ou produits qui répliquent 1:1 la performance d'un sous-jacent (swaps, TRS) | Synthétique |
| **Debit balance** / **Credit balance** | Cash emprunté par le client / cash du client chez le PB | Base du financement |
| **Rebate** | Intérêt reversé au client sur le cash de ses ventes à découvert | — |
| **Locate** | Confirmation préalable qu'un titre peut être emprunté avant une vente à découvert | Reg SHO |
| **GC** / **HTB** / **Special** | Titre facile / difficile / très difficile à emprunter | Les CA rendent souvent les titres HTB |
| **Borrow** / **Loan** / **Street side** | Emprunt / prêt de titres avec des contreparties externes | Equity Finance |
| **Internalization** | Couvrir le short d'un client avec les longs d'un autre | Rehypo |
| **Rehypo** (*Rehypothecation*) | Réutilisation des titres clients en marge | §9.2 |
| **Fully paid** / **Excess margin** | Titres payés / au-delà du besoin de marge, à protéger | Rule 15c3-3 |
| **Buy-in** | Rachat forcé de titres non livrés | — |
| **Margin call** / **House call** / **Maintenance call** | Appel de marge (règles internes / maintenance) | §10E |
| **Haircut** | Décote appliquée à la valeur d'un collatéral | 100 % sur un titre non valorisé |
| **Give-up** / **Executing broker (EB)** | Trade exécuté chez un autre broker puis transféré au PB | §10D |
| **Fund admin** / **Administrator** | Prestataire qui tient la comptabilité et calcule la NAV du fonds | Source de queries |
| **NAV strike** | Calcul officiel de la NAV (souvent mensuel) | Les breaks doivent être expliqués avant |
| **Multi-prime** | Fonds utilisant plusieurs PB | — |

### F.5 Fiscalité et documentation

| Terme US | Définition | Équivalent / remarque |
|---|---|---|
| **NRA withholding** | Retenue sur les revenus US versés aux non-résidents (30 % par défaut) | Chapitre 3 |
| **Treaty rate** / **Treaty benefits** | Taux réduit par une convention fiscale | Nécessite une documentation valide |
| **W-9** / **W-8BEN** / **W-8BEN-E** / **W-8IMY** | Formulaires de statut fiscal (US / individu non-US / entité non-US / intermédiaire) | Expirations = breaks fiscaux |
| **Backup withholding** | Retenue de 24 % si un US person n'a pas de W-9 valide | — |
| **1099-DIV / -INT / -B / -MISC** | Reporting fiscal annuel des US persons | Substitute payments en 1099-MISC |
| **1042-S** | Reporting des revenus versés aux non-US | — |
| **QI** / **QDD** | Qualified Intermediary / Qualified Derivatives Dealer | Statuts IRS |
| **871(m)** / **Dividend equivalent** | Retenue sur les équivalents de dividendes des dérivés (non-US) | Synthétique |
| **Reclass** / **Year-end reclassification** | Requalification fiscale de fin d'année (ROC, capital gains) | ETF, REIT, fonds |
| **ROC** (*Return of Capital*) | Remboursement de capital, non imposable comme un dividende | — |
| **Form 8937** | Formulaire de l'émetteur qui décrit l'impact d'une CA sur le coût de revient | Spin-offs, réorgs |
| **Cost basis** | Coût de revient fiscal | EX-36 |

### F.6 Acteurs, systèmes et sigles

| Terme US | Définition | Équivalent / remarque |
|---|---|---|
| **DTC** / **DTCC** / **"the Street"** | Dépositaire central US / sa holding / l'ensemble des autres banques et brokers | — |
| **Cede** / **Cede & Co.** | Nominee de la DTC dans les registres des émetteurs | — |
| **Participant** / **Participant number** | Banque ou broker membre de la DTC, identifié par un numéro | Le PB est participant |
| **CA Web** | Portail DTC des corporate actions | — |
| **Transfer agent** / **TA** | Tient le registre des actionnaires | Computershare, Equiniti… |
| **Paying agent** / **Trustee** | Verse les paiements / représente les porteurs obligataires | — |
| **Info agent** / **Tender agent** / **Exchange agent** | Agents d'une offre | — |
| **OCC** / **Info Memo** | Chambre de compensation des options / note d'ajustement | §13.1 |
| **CUSIP** / **"New CUSIP"** / **"CUSIP change"** | Identifiant US / changement d'identifiant après une réorg | EX-08 |
| **Golden copy** / **Scrub** / **Scrubbing** | Version validée d'un événement / nettoyage des données | §6.2 |
| **Ops** / **Middle office** / **Back office** | Opérations ; contrôle post-trade ; traitement et règlement | — |
| **BAU** (*Business As Usual*) | Fonctionnement courant (vs projet) | Cible après go-live |
| **EUC** | *End-User Computing* : Excel, macros, outils hors IT | §30 |
| **SME** (*Subject Matter Expert*) | Expert métier | — |
| **UAT** / **Parallel run** / **Go-live** / **Hypercare** | Phases de test et de mise en production | §25 |
| **CAB** (*Change Advisory Board*) | Comité qui autorise les mises en production | IT |
| **SteerCo** | Comité de pilotage | §27 |
| **RAG** (*Red / Amber / Green*) | Code couleur des statuts projet | Status report |

### F.7 Phrases types entendues au desk

| Phrase | Ce que ça veut dire |
|---|---|
| *"The client didn't respond, we'll default them."* | Pas de réponse : on applique l'option par défaut. |
| *"The offer got extended to the 23rd."* | L'offre est prolongée : nouvelle deadline, relancer les non-répondants. |
| *"We're short the protect, call Equity Finance."* | On n'a pas les titres pour couvrir le protect : il faut accélérer le recall. |
| *"They over-instructed by 50k."* | L'élection dépasse la position de 50 000 titres. |
| *"DTC hasn't paid yet, it's sitting in unpaid."* | Le paiement n'est pas arrivé, le break est en catégorie UNPAID. |
| *"It's a tax break, the W-8 expired."* | Écart de taxe causé par une documentation fiscale expirée. |
| *"The admin is chasing us on the Granite special."* | L'administrateur du fonds relance sur le dividende spécial Granite. |
| *"Raise a claim against the street."* | Ouvrir une réclamation contre la contrepartie externe. |
| *"It's aged, we need to escalate before month-end."* | Le break est ancien, il faut escalader avant la clôture (NAV, capital). |
| *"Is it contractual or actual?"* | Le client est-il crédité à la pay date ou à réception du paiement ? |
| *"That's a due bill situation."* | Trade dans la fenêtre due bill : la distribution est due à l'acheteur. |
| *"Check the Info Memo."* | Vérifier l'ajustement publié par l'OCC pour les options. |
