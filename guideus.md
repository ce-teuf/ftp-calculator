# Corporate Actions in US Prime Brokerage
### Complete Guide: Operations, Project Management & Dagster Automation

> **Who this is for:** a PM / Python developer joining a Global Markets ITO team (NYC) who needs to automate break detection in Corporate Actions for Prime Brokerage operations.
> **Convention:** **key business terms in bold** (this is the vocabulary you will hear on the desk).
> **Example workbook:** each illustrated section points to a worked example **EX-xx** in the file `CA_PB_US_Exemples.xlsx` (clickable `Index` tab). All data in it is fictitious. The workbook itself is in French (tab names and labels).
> **Disclaimer:** Firm-specific items (system names, organization, internal rules) are flagged **[TO BE CONFIRMED INTERNALLY]**. Tax and regulatory rules change: always validate with Tax / Compliance before coding a rule.

---

## Table of Contents

**Part I — Fundamentals**
1. [Corporate Actions on one page](#1-corporate-actions-on-one-page)
2. [Event classification](#2-event-classification)
3. [Event type catalog (ISO codes)](#3-event-type-catalog-iso-codes)
4. [Key dates and US specifics](#4-key-dates-and-us-specifics)

**Part II — The US Ecosystem**
5. [US market participants](#5-us-market-participants)
6. [Data sources: Bloomberg, Broadridge, DTC…](#6-data-sources-bloomberg-broadridge-dtc)
7. [Messages and standards (SWIFT MT, ISO 20022)](#7-messages-and-standards-swift-mt-iso-20022)

**Part III — Prime Brokerage**
8. [What is a Prime Broker](#8-what-is-a-prime-broker)
9. [How a PB's positions are structured](#9-how-a-pbs-positions-are-structured)
10. [What makes CAs more complex in PB](#10-what-makes-cas-more-complex-in-pb)

**Part III-bis — Prime Brokerage in depth (addition)**
- 10A. [The PB business and its clients](#10a-the-pb-business-and-its-clients)
- 10B. [The business model](#10b-the-business-model)
- 10C. [Front-to-back organization of a PB](#10c-front-to-back-organization-of-a-pb)
- 10D. [The daily trade cycle in PB](#10d-the-daily-trade-cycle-in-pb)
- 10E. [Margin and risk](#10e-margin-and-risk)
- 10F. [Equity Finance and Stock Loan](#10f-equity-finance-and-stock-loan)
- 10G. [Synthetic Prime in detail](#10g-synthetic-prime-in-detail)
- 10H. [Treasury, funding and collateral](#10h-treasury-funding-and-collateral)
- 10I. [Reporting, NAV and client experience](#10i-reporting-nav-and-client-experience)
- 10J. [PB-specific regulation](#10j-pb-specific-regulation)
- 10K. [CA impact matrix across PB functions](#10k-ca-impact-matrix-across-pb-functions)
- 10L. [Timeline of an event from the PB's perspective](#10l-timeline-of-an-event-from-the-pbs-perspective)
- 10M. [Typical PB incident scenarios](#10m-typical-pb-incident-scenarios)
- 10N. [Implications for your project](#10n-implications-for-your-project)

**Part IV — By asset class**
11. [Equities, ADRs, ETFs/funds, REITs/MLPs, SPACs](#11-equities-adrs-etfsfunds-reitsmlps-spacs)
12. [Fixed Income: corporates, munis, Treasuries, MBS/ABS](#12-fixed-income-corporates-munis-treasuries-mbsabs)
13. [Listed and OTC derivatives](#13-listed-and-otc-derivatives)
14. [Securities Lending and Repo](#14-securities-lending-and-repo)

**Part V — Day-to-day operations**
15. [The detailed lifecycle](#15-the-detailed-lifecycle)
16. [How a PB CA team is organized, and a typical day](#16-how-a-pb-ca-team-is-organized-and-a-typical-day)
17. [US taxation](#17-us-taxation)
18. [Accounting: accruals and income](#18-accounting-accruals-and-income)

**Part VI — Breaks (the core of your project)**
19. [Break taxonomy](#19-break-taxonomy)
20. [Matching keys and tolerances](#20-matching-keys-and-tolerances)
21. [Claims management](#21-claims-management)
22. [KPIs, risks and controls](#22-kpis-risks-and-controls)

**Part VII — Worked examples**
23. [Nine worked cases](#23-nine-worked-cases)

**Part VIII — Project Management**
24. [Your dual PM + Dev role](#24-your-dual-pm--dev-role)
25. [The lifecycle of an ops automation project](#25-the-lifecycle-of-an-ops-automation-project)
26. [Discovery: mapping and measuring](#26-discovery-mapping-and-measuring)
27. [Working with the Sponsor](#27-working-with-the-sponsor)
28. [Working with IT](#28-working-with-it)
29. [Working with Ops (SMEs)](#29-working-with-ops-smes)
30. [Governance in a bank: controls, EUC, data, audit](#30-governance-in-a-bank-controls-euc-data-audit)
31. [Ready-to-use templates](#31-ready-to-use-templates)
32. [Prioritized use case backlog](#32-prioritized-use-case-backlog)
33. [Classic pitfalls](#33-classic-pitfalls)

**Part IX — Dagster implementation**
34. [Target architecture](#34-target-architecture)
35. [Canonical data model](#35-canonical-data-model)
36. [Dagster code skeleton](#36-dagster-code-skeleton)
37. [The matching engine](#37-the-matching-engine)
38. [Outputs: Excel and email](#38-outputs-excel-and-email)
39. [Testing, quality, deployment, run](#39-testing-quality-deployment-run)

**Appendices**
- [A. Glossary](#appendix-a--glossary)
- [B. SWIFT/ISO fields to know](#appendix-b--swiftiso-fields-to-know)
- [C. Checklist of questions to ask internally](#appendix-c--checklist-of-questions-to-ask-internally)
- [D. 30-60-90 day plan](#appendix-d--30-60-90-day-plan)
- [E. Further resources](#appendix-e--further-resources)
- [F. US terminology bridge (NYC desk jargon)](#appendix-f--us-terminology-bridge-nyc-desk-jargon)

---

# PART I — FUNDAMENTALS

## 1. Corporate Actions on one page

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-01** (sheet `P1_Fondamentaux`)

A **Corporate Action (CA)** is an event decided by an **issuer** that changes a security or what its holders receive: paying a dividend or coupon, a split, a merger, a buyback offer, a bond redemption, etc.

**The CA business in one sentence:** for each event, figure out **who is entitled to what** (**entitlement**), **collect** it from the market, **distribute** it correctly to clients, **on time**, with **the right tax treatment**, and **prove** that everything ties out (**reconciliation**).

**Why it's hard:**

| Difficulty | Explanation |
|---|---|
| Volume | Tens of thousands of events a year in the US alone, and millions of position lines. |
| Imperfect data | Sources (Bloomberg, DTC, agents, custodians) often **contradict each other**, especially early in an event's life. |
| Living events | An event is announced, then **amended** (rates, dates), sometimes **cancelled**. Every update can create breaks. |
| Moving positions | Unsettled trades, securities lending, shorts, rehypothecation: the position "entitled" to the event is not trivial. |
| Tax | The withholding rate depends on the client's tax status, the type of income and the country. |
| Deadlines | Voluntary events have strict **deadlines**; a missed election means a direct financial loss. |
| Risk | CAs have historically been **one of the top sources of operational losses** in securities back offices. |

---

## 2. Event classification

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-02** (sheet `P1_Fondamentaux`)

### 2.1 By holder participation (ISO field `CAMV`)

| Category | Code | Description | Examples |
|---|---|---|---|
| **Mandatory** | `MAND` | Applies automatically, no action required. | Cash dividend, split, name change, cash merger |
| **Mandatory with options** (**Elective**) | `CHOS` | The holder **chooses** among options; with no response, the **default option** applies. | Cash or stock dividend, merger with cash/stock election |
| **Voluntary** | `VOLU` | The holder **may** participate; with no response, nothing happens (**No Action** / `NOAC`). | Tender offer, rights exercise, buyback, consent solicitation |

### 2.2 By economic nature

| Nature | Description | Examples |
|---|---|---|
| **Distribution / Income** | The holder receives cash or securities and keeps the original security. | Dividend, coupon, spin-off, bonus issue |
| **Reorganization (Reorg)** | The security is transformed or exchanged. | Split, merger, conversion, redemption |
| **Informational / Governance** | No flow, but information or a vote. | AGM (proxy), class action, name change |

> **US vocabulary to remember:** teams and systems are often split into **Income** (or **Dividends** / **Distributions**), **Mandatory Reorg**, **Voluntary Reorg** and **Proxy**. DTC uses the same logic.

---

## 3. Event type catalog (ISO codes)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-03** (sheet `P1_Fondamentaux`)

The codes come from the **`CAEV`** field (ISO 15022/20022). They will be your categorization keys in the code.

### 3.1 Distributions / Income

| Code | Name | Description | What happens |
|---|---|---|---|
| **DVCA** | Cash Dividend | Cash dividend | Cash per share |
| **DVSE** | Stock Dividend | Dividend paid in shares | N new shares for M held |
| **DVOP** | Dividend Option | Dividend with a choice | Cash and/or shares (a `CHOS` event) |
| **DRIP** | Dividend Reinvestment | Reinvestment | Cash is converted into shares |
| **BONU** | Bonus Issue | Free shares | Comparable to a DVSE |
| **CAPD** | Capital Distribution | Return of capital | Cash, taxed as *return of capital* |
| **CAPG** | Capital Gains Distribution | Capital gains distribution | Common on funds/ETFs/REITs |
| **INTR** | Interest Payment | Coupon | Cash based on face × rate × year fraction |
| **PRED** | Partial Redemption (without reduction of nominal) / **PCAL** Partial Call | Partial redemption | See Fixed Income |
| **SOFF** | Spin-off | Spin-off | Shares of a new entity |
| **DVOP / SHPR** | Shares Premium Dividend | Variant | — |
| **RHDI** | Intermediate Securities Distribution | Rights distribution | Rights are credited to the account |
| **DECR / INCR** | Decrease/Increase in Value | Reduction/increase in nominal | Change in nominal value |

### 3.2 Mandatory reorganizations

| Code | Name | Description |
|---|---|---|
| **SPLF** | Stock Split (Forward) | 1 share → N shares |
| **SPLR** | Reverse Split | N shares → 1 share (fractions are common) |
| **MRGR** | Merger | Exchange for cash and/or acquirer shares |
| **NAME** | Name Change | New name (often a new CUSIP/ticker) |
| **CHAN** | Change | Change of terms (e.g., new CUSIP) |
| **CONV** | Conversion | Bond/preferred → shares (can be voluntary) |
| **EXOF** | Exchange | Exchange of securities (can be voluntary) |
| **REDM** | Final Maturity | Redemption at maturity |
| **MCAL** | Full Call | Full early redemption |
| **PCAL** | Partial Call | Partial early redemption (with reduction of nominal) |
| **PRED** | Partial Redemption without reduction of nominal | Amortization via factor (MBS/ABS) |
| **LIQU** | Liquidation | Liquidating distribution |
| **BRUP** | Bankruptcy | Bankruptcy |
| **DLST** | Delisting | Delisting |
| **WRTH** | Worthless | Security declared worthless |
| **PINK** | Payment in Kind | Coupon paid in securities |

### 3.3 Voluntary events

| Code | Name | Description |
|---|---|---|
| **TEND** | Tender Offer | Tender offer (cash and/or securities) |
| **BIDS** | Repurchase Offer | Issuer buyback (buyback / Dutch auction) |
| **EXRI** | Call on Intermediate Securities | Rights exercise |
| **PRIO** | Priority Issue | Priority subscription |
| **CONS** | Consent | Consent solicitation (bondholder vote on amendments to the indenture) |
| **BPUT** | Put Redemption | Holder requests early redemption |
| **EXWA** | Warrant Exercise | Warrant exercise |
| **ODLT** | Odd Lot Sale/Purchase | Offer to odd-lot holders |
| **DSCL** | Disclosure | Request to disclose holders |

### 3.4 Governance, information, litigation

| Code | Name | Description |
|---|---|---|
| **MEET / XMET / OMET** | Meetings | Shareholder meetings (proxy voting) |
| **CLSA** | Class Action | Class action: holders over a period may file claims |
| **INFO** | Information | Purely informational |
| **ACTV / SUSP** | Trading Status | Trading resumed or suspended |

### 3.5 Event options (field `CAOP`)

| Code | Meaning |
|---|---|
| **CASH** | Receive cash |
| **SECU** | Receive securities |
| **CASE** | Cash + securities |
| **NOAC** | Take no action |
| **LAPS** | Let lapse (rights) |
| **EXER** | Exercise |
| **SLLE** | Sell (e.g., sell your rights) |
| **BUYA** | Buy additional rights |
| **OVER** | Oversubscribe (**oversubscription**) |
| **CONY / CONN** | Consent / do not consent |
| **MPUT** | Exercise a put |
| **PRUN** | Pro rata (**proration**) |

---

## 4. Key dates and US specifics

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-04** (sheet `P1_Fondamentaux`)

### 4.1 The dates (ISO fields in parentheses)

| Date | Field | Definition | Why it matters |
|---|---|---|---|
| **Announcement Date** | `ANOU` | Public announcement | Processing starts |
| **Declaration Date** | — | Formal board decision (US term) | Often the same as announcement |
| **Ex-Date** | `XDTE` | First trading day **without** the entitlement | Price drops; this is when the accrual flips |
| **Record Date** | `RDTE` | Date on which registered holders are captured | Determines market-side entitlement |
| **Payable / Pay Date** | `PAYD` | Payment date | Date of actual cash/stock → reconciliation |
| **Effective Date** | `EFFD` | Date a reorg takes effect | CUSIP and quantity change |
| **Market Deadline** | `MKDT` | Agent's deadline | The market's hard stop |
| **Response Deadline** | `RDDT` | Custodian/DTC deadline | Earlier than the market deadline |
| **Client Deadline** | (internal) | Deadline set for PB clients | Even earlier, to leave time to process |
| **Protect Deadline** | `PODT` | Deadline to deliver securities after a *guaranteed delivery* | Specific to US tenders |
| **Expiration Date** | `EXPI` | End of the offer | Can be **extended** |
| **Proration Date** | — | Pro rata calculation if the offer is oversubscribed | Result ≠ instruction |

### 4.2 The golden rule: ex-date vs record date in the US

Since **May 2024**, US equities settle **T+1**.

- For a regular dividend: **Ex-Date = Record Date**.
- If you buy the day before the ex-date (T), your trade settles at T+1 = record date. You are on the register and entitled to the dividend.
- If you buy on the ex-date, you settle after the record date: no dividend.

> **Watch out:** other markets have different cycles (T+2 in Europe as of today, with a planned move to T+1). For non-US securities held by US clients, ex-date and record date can differ.

### 4.3 Who sets the ex-date in the US?

- For **listed** securities: the **exchange** (NYSE, Nasdaq…), not the issuer.
- For **OTC** securities: **FINRA** (*Uniform Practice Code* rules).
- Issuers must notify distributions in advance (SEC Rule 10b-17 for non-listed securities; listing rules for exchanges). **Consequence:** there is sometimes a lag between the issuer's announcement and the setting of the ex-date, hence incomplete data early on.

### 4.4 **Due Bills** (a US specific, the #1 source of breaks on large events)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-05** (sheet `P1_Fondamentaux`)

For distributions **equal to or greater than 25%** of the security's value (large split, large special dividend, some spin-offs), the ex-date is set to **the first business day after the pay date**.

Consequences:

- Between the record date and the ex-date, the security still trades **"with"** the distribution (*cum*).
- The seller, registered on the record date, receives the distribution but **owes it to the buyer**: this is the **due bill**.
- DTC handles some of these adjustments through **interim accounting**.
- **For you:** any entitlement computed purely on "position at record date" will be **wrong** for trades in the due bill window. You need a dedicated rule.

### 4.5 Sample calendar (regular dividend)

```
Mon 11/03  Announcement   Apple declares $0.25 per share
Mon 11/10  Ex-date = Record date
Thu 11/13  Pay date       Cash credited via DTC
Fri 11/14  Recon          Expected vs Received → possible breaks
```

---

# PART II — THE US ECOSYSTEM

## 5. US market participants

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-06** (sheet `P2_Ecosysteme`)

### 5.1 Market infrastructure

| Participant | Role in CAs |
|---|---|
| **DTCC** (*Depository Trust & Clearing Corporation*) | Holding company of the US post-trade infrastructure. |
| **DTC** (*Depository Trust Company*) | **The US CSD** for equities, corporate bonds and munis. It collects payments from agents and distributes them to its **participants** (brokers, including your firm). It publishes CA announcements, handles voluntary elections and payments. |
| **Cede & Co.** | DTC's **nominee**: it appears on issuers' registers as the legal holder of securities held at DTC. |
| **NSCC** | Clearing house for US equity trades. It also handles CA adjustments on open trades (transformations). |
| **Fedwire Securities Service** (Federal Reserve) | Depository for **US Treasuries** and **agency MBS** (Fannie Mae, Freddie Mac, Ginnie Mae). This is **not** DTC. |
| **OCC** (*Options Clearing Corporation*) | Clearing house for **listed options**. It publishes **contract adjustments** after CAs. |
| **Exchanges** (NYSE, Nasdaq, Cboe) | Set ex-dates for listed securities. |
| **FINRA** | Broker self-regulatory organization; OTC ex-dates; practice rules. |
| **SEC** | Regulator (client protection rules, notification rules, etc.). |
| **MSRB / EMMA** | Official information source for **municipal bonds**. |

### 5.2 Issuer side

| Participant | Role |
|---|---|
| **Issuer** | Decides the event |
| **Transfer Agent** (Computershare, Equiniti, Broadridge…) | Keeps the shareholder register (very central in the US) |
| **Paying Agent** | Pays out cash (for bonds, often the **trustee**) |
| **Trustee** | Represents bondholders (BNY, US Bank, Wilmington…) |
| **Exchange / Tender / Information Agent** | Runs offers, collects elections |
| **Depositary Bank** (BNY, Citi, JPMorgan, Deutsche Bank) | Issues **ADRs** and handles CAs on the underlying foreign security |

### 5.3 Investor side (a Prime Broker's world)

| Participant | Role |
|---|---|
| **Prime Broker** (your firm) | Holds, finances and lends securities to hedge funds; it is the **DTC participant** |
| **Clients** (hedge funds) | Economic holders; they make the elections |
| **Fund Administrators** (Citco, SS&C GlobeOp, State Street, etc.) | Keep the funds' books. **They reconcile too**, and they are often the ones raising breaks to the PB. |
| **Executing brokers** | Brokers that execute client trades, settled at the PB (*give-ups*) |
| **Agent lenders / Street lenders** | Securities lending counterparties |
| **Sub-custodians** | For non-US securities held by clients |
| **Data vendors** | Bloomberg, LSEG, ICE, S&P, SIX… |

---

## 6. Data sources: Bloomberg, Broadridge, DTC…

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-07** (sheet `P2_Ecosysteme`)

### 6.1 Overview

| Source | What it provides | Strengths | Weaknesses |
|---|---|---|---|
| **DTC** (CA feed, often ISO 20022, + **CA Web** portal) | "Official" announcements for DTC-eligible securities, entitlements, payments, elections | The actual settlement source: **this is what will be paid** | Limited to DTC securities; technical formats |
| **Bloomberg** (terminal: `CACS <GO>`, `DVD <GO>`; **Bloomberg Data License** bulk files via SFTP) | Global CA announcements, history, **Bloomberg Action ID** | Broad coverage, fast | Not a settlement source; can differ from DTC |
| **Broadridge** | Very present in the US: broker-dealer back-office platforms (e.g., **Impact**), **proxy voting** (Broadridge distributes the vast majority of US voting materials, **ProxyEdge** tool), CA services | Integrated with the back office | You need to know **which Broadridge product** your firm uses **[TO BE CONFIRMED INTERNALLY]** |
| **Custodians / sub-custodians** (MT564/566) | Announcements and payments for non-DTC securities | Local settlement source | Variable formats |
| **Agents / Trustees** | Official documents (prospectus, offer docs) | Primary source | Unstructured (PDF) |
| **EMMA (MSRB)** | Munis | Official | — |
| **MBS agencies** | Monthly factors | Official | — |
| **OCC** | Option adjustment Info Memos | Official | Unstructured |

### 6.2 The **Golden Copy**

**Scrubbing** means comparing sources, resolving conflicts and publishing a validated version of each event: the **golden copy** (or *validated event*). Market CA systems offer rules such as "if Bloomberg and DTC agree, the event is auto-validated; otherwise, manual review."

**Fields typically compared:** event type, ISIN/CUSIP, dates (ex, record, pay), gross rate, currency, ratio, options, default option, deadlines, status (confirmed/preliminary/cancelled).

> **Link to your project:** **Event Reconciliation** (vendor vs vendor vs internal system) is probably **the first and simplest high-impact use case** to automate.

### 6.3 Identifiers you need to know

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-08** (sheet `P2_Ecosysteme`)

| Identifier | Description |
|---|---|
| **CUSIP** | 9 characters, the main **US identifier** |
| **ISIN** | 12 characters; for the US: `US` + CUSIP + check digit |
| **SEDOL** | UK identifier |
| **Ticker / Exchange code** | Unstable: changes with CAs |
| **FIGI** (Bloomberg) | Bloomberg open identifier |
| **COAF** | *Official Corporate Action Event Reference*: the unique "official" event identifier |
| **CORP** | The provider's event reference (DTC, custodian…) |
| **Bloomberg Action ID** | Bloomberg event identifier |
| **Internal Event ID** | Reference in the internal CA system **[TO BE CONFIRMED INTERNALLY]** |

> **Trap:** after a `NAME`/`CHAN`/`MRGR`, **the CUSIP changes**. Your matching must handle an **old → new CUSIP mapping**.

---

## 7. Messages and standards (SWIFT MT, ISO 20022)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-09** (sheet `P2_Ecosysteme`)

### 7.1 The messages

| ISO 15022 | ISO 20022 | Direction | Use |
|---|---|---|---|
| **MT564** | seev.031 (notification), seev.035 (movement preliminary advice), seev.044 (cancellation) | Custodian → Client | Announcement, update, preliminary advice of movement |
| **MT565** | seev.033 | Client → Custodian | Election instruction |
| **MT566** | seev.036 (confirmation), seev.037 (reversal) | Custodian → Client | Payment/movement confirmation |
| **MT567** | seev.032 / seev.034 | Custodian → Client | Status of an instruction or event |
| **MT568** | seev.039 / seev.040 | Both ways | Free-format narrative |

### 7.2 MT564 functions (field `23G`)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-10** (sheet `P2_Ecosysteme`)

- **NEWM**: new announcement
- **REPL**: replacement (update)
- **REPE**: *eligible balance* / preliminary payment advice (with position and amounts)
- **CANC**: cancellation
- **WITH**: event withdrawn

Plus the completeness status: **PREU** (preliminary, unconfirmed) vs **COMP** (complete).

> **For your pipeline:** an event is a **series of versions**. You need to **keep history** (event versioning) and always compare against **the latest valid version**.

### 7.3 Standards bodies

- **SMPG** (*Securities Market Practice Group*): global usage rules for CA messages.
- **ISITC**: the US equivalent for market practices.

---

# PART III — PRIME BROKERAGE

## 8. What is a Prime Broker

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-11** (sheet `P3_PB`)

A **Prime Broker (PB)** is the "all-in-one" bank of a **hedge fund**. Its services:

| Service | Description |
|---|---|
| **Custody / Clearing / Settlement** | Safekeeping of the fund's assets, settlement of its trades |
| **Financing (Margin lending)** | Lending cash to the fund so it can buy with leverage |
| **Securities Lending (Stock borrow)** | Lending securities to the fund so it can **sell short** (*short selling*) |
| **Synthetic Prime / Swaps** | Exposure via **portfolio swaps / total return swaps (TRS)** instead of physical securities |
| **Capital Introduction** | Introductions to investors |
| **Reporting** | Positions, P&L, margin, **CA notifications** |
| **Corporate Actions services** | Notification, election and payment of events |

**Clients:** hedge funds running a wide range of strategies. **Event-driven / merger arbitrage** strategies are **extremely sensitive to CAs**: they live off mergers and offers, so voluntary elections are critical.

**PB revenue:** spread on financing, securities lending fees (*borrow fees*), commissions, and **reuse of client securities** (rehypothecation) for funding or lending. That last point has a major impact on CAs (see §10).

---

## 9. How a PB's positions are structured

### 9.1 Client account types (US vocabulary)

| Account | Description |
|---|---|
| **Cash account** | Securities paid in full |
| **Margin account** | Securities bought on credit (the client has a **debit balance**) |
| **Short account** | Short positions |
| **Fully paid vs Margin securities** | "Fully paid" or **excess margin** securities must be protected; the rest can be reused |

### 9.2 **Rehypothecation** and **SEC Rule 15c3-3**

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-12** (sheet `P3_PB`)

- The **Customer Protection Rule (15c3-3)** requires the broker to keep **fully paid** and **excess margin** client securities in its **possession or control** (in segregated accounts: **box**, **seg**).
- On the other hand, the PB may **reuse** (rehypothecate) client margin securities, up to a cap (classically **140% of the client's debit balance**).
- These reused securities can be **lent** to others (*street lending*) or put into **repo**.

**Major CA consequence:** on the record date, part of "the client's" securities are **not physically at the PB**; they sit with a borrower. The market (DTC) pays the dividend to **the borrower**. The PB still has to credit its client: this is a **substitute payment** / **payment in lieu of dividend** (see §10.1).

### 9.3 Position "layers"

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-13** (sheet `P3_PB`)

For a given CUSIP on the record date, the PB must consolidate:

```
Client positions (longs)              +  1,000,000
Client positions (shorts)             -    300,000
Securities borrowed from the street   +    300,000   (to cover the shorts)
Securities lent to the street (loans) -    200,000   (rehypo / lending)
Firm proprietary positions            ±  …
Unsettled trades (fails, pending)     ±  …
────────────────────────────────────────────────────
= Expected position at DTC (depot)    =    800,000
```

**The DTC position must tie out to the sum of internal positions.** If it doesn't: **position break**, hence an **entitlement break** at payment.

### 9.4 The **stock record**

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-14** (sheet `P3_PB`)

This is the broker-dealer's securities ledger: for each security, the sum of **ownership positions** (clients, firm) must equal the sum of **location positions** (DTC, Fed, custodians, on loan, in transit). The stock record recon is the foundation of everything. A stock record break on the record date becomes a CA break.

---

## 10. What makes CAs more complex in PB

### 10.1 Dividends and **substitute payments**

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-15** (sheet `P3_PB`)

- Client long, securities **not lent** → the client receives a **real dividend** (potentially a *qualified dividend* for a US taxpayer).
- Client long, securities **rehypothecated/lent** → the borrower pays the PB a **manufactured dividend**, and the PB credits the client with a **substitute payment in lieu of dividend**.
  - **Taxed differently** for a US taxpayer: the substitute payment is generally taxed as ordinary income and reported on **Form 1099-MISC** (not 1099-DIV).
  - For non-US clients: withholding applies to the substitute payment.
  - The PB must **allocate** which clients received a real dividend and which received a substitute payment (allocation methodology **[TO BE CONFIRMED INTERNALLY]**).

### 10.2 Shorts: the client **owes** the dividend

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-16** (sheet `P3_PB`)

- A client **short** on the record date must pay the dividend to the securities lender.
- The PB **debits** the client's short account (often on the pay date, with an accrual on the ex-date) and pays the lender (*street lender*).
- **Common breaks:** amount debited ≠ amount owed to the lender, wrong rate applied, shorts covered between ex-date and record date…

### 10.3 Voluntary elections with lent securities

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-17** (sheet `P3_PB`)

This is **the** sensitive point in PB. Example: a merger arb fund wants to **tender** 500,000 shares into an offer, but 300,000 are rehypothecated and on loan.

Options:

1. **Recall**: recall the lent securities in time.
2. **Guaranteed Delivery / Protect**: on some US offers, you can instruct with a **Notice of Guaranteed Delivery** and deliver the securities later (before the **protect deadline**). If you don't deliver: **non-delivery** → potential loss.
3. **Buy-in**: buy the securities in the market.
4. **Liability claim**: the borrower remains liable for the economic outcome of the election (**voluntary CA liability**, under the lending agreement).

**Common breaks:** client election > available position; uncovered protect; proration result ≠ expected.

### 10.4 **Synthetic Prime** (swaps)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-27** (sheet `P3bis_PB_Profondeur`)

- The client has **synthetic** exposure via a **portfolio swap**; the PB holds the physical hedge.
- CAs are **replicated contractually** on the swap: **dividend pass-through** (often a percentage of the gross amount defined in the confirmation), quantity adjustment on splits, treatment of mergers under the **ISDA Equity Definitions** (*Merger Events*, *Tender Offers*, *Potential Adjustment Events*).
- **871(m) tax:** for non-US clients, **dividend equivalents** on "delta-one" derivatives on US equities are subject to withholding (see §17).
- **Common breaks:** dividend paid on the swap ≠ dividend received on the hedge (*hedge vs swap*), wrong pass-through percentage, forgotten notional adjustment.

### 10.5 **Fund Administrators**

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-18** (sheet `P3_PB`)

Funds have their own administrator, who reconciles its books against the **PB's statements**. A significant share of CA ops work is **answering admin and client queries**: "Why did I receive $21,250 instead of $25,000?" Hence the value of having breaks **already identified and explained** before the client calls.

### 10.6 **Proxy voting**

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-19** (sheet `P3_PB`)

- The voting right follows the security: **lent** securities do not vote for the client.
- In the US, distribution of voting materials and vote collection go overwhelmingly through **Broadridge** (ProxyEdge).
- A client who wants to vote lent securities must request a **recall** before the meeting's record date.

### 10.7 Client notifications

The PB must **notify** its clients of every event that affects them (portal, email, SWIFT, files). Errors or delays in notifying a voluntary event = **risk of client claims**.

---

# PART III-bis — PRIME BROKERAGE IN DEPTH

> This part complements §8 to §10. The goal is to understand **how a PB works from the inside**, then see **how each Corporate Action propagates through each of its functions**.

## 10A. The PB business and its clients

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-20** (sheet `P3bis_PB_Profondeur`)

### Why Prime Brokerage exists

A hedge fund trades with many **executing brokers** (to get best execution), but it needs **a central point** that:

1. **holds** all of its assets in one place (custody);
2. **settles** all of its trades, regardless of the executing broker (**give-up** / **clearing**);
3. **lends it cash** (leverage) and **securities** (to sell short);
4. computes **consolidated margin** across its whole portfolio (longs partly offset shorts);
5. provides **consolidated reporting** (positions, P&L, financing, CAs).

That central point is the **Prime Broker**. The model dates back to the 1980s in the US and spread as hedge funds grew.

**Multi-prime:** since the 2008 crisis (the Lehman failure, where funds saw their assets frozen), most large funds use **several PBs**. The consequences for CAs:
- the same fund holds positions in the same security **at several PBs**;
- it must send its elections **to each of them**;
- the **fund administrator** consolidates and compares what each PB paid.

**Your firm in PB:** many of today's PB platforms are the product of acquisitions or business combinations. In the US, the broker-dealer business runs through an SEC/FINRA-regulated entity **[TO BE CONFIRMED INTERNALLY: entities, legacy platforms, systems]**. If legacy clients and systems from a migration still coexist, that may explain **some of the breaks between systems**.

### Clients and their sensitivity to CAs

| Fund strategy | What it does | CA sensitivity | Why |
|---|---|---|---|
| **Long/Short Equity** | Buys undervalued stocks, shorts overvalued ones | **High** | Lots of dividends on longs **and** shorts, splits, spin-offs |
| **Market Neutral / Stat Arb / Quant** | Thousands of long and short positions, low net | **High (volume)** | Huge number of small events. Errors are many and diffuse |
| **Event-Driven / Merger Arbitrage** | Bets on mergers and offers closing | **Critical** | The fund's P&L **is** the event: tenders, elections, prorations, protects |
| **Special Situations / Distressed** | Invests in companies in distress | **Critical** | Restructurings, bankruptcies, debt exchanges, consent solicitations |
| **Convertible Arbitrage** | Long convertible / short stock | **High** | Conversion ratio adjustments, dividends on the short, calls |
| **Activist** | Takes stakes to influence governance | **High (proxy)** | Votes, recalls of lent securities to vote |
| **SPAC Arbitrage** | Buys SPACs below trust value | **High** | Redemption rights, unit splits, de-SPAC |
| **Global Macro** | Rates, FX, indices | Low on securities | Mostly futures and derivatives |
| **Credit Long/Short** | Bonds and CDS | **Medium/High** | Coupons, calls, debt tenders, defaults |
| **Family offices / Asset managers** | More traditional management | Medium | Dividends, coupons, proxy |

> **Key takeaway:** in PB, clients are **sophisticated**. They follow their events very closely, and a merger arb fund **often knows the event better than ops do**. Any error is spotted immediately, and the quality bar is very high.

---

## 10B. The business model

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-21** (sheet `P3bis_PB_Profondeur`)

| Revenue source | Mechanics | Link with CAs |
|---|---|---|
| **Margin financing** (financing longs) | The client pays a rate on its **debit balance** (USD reference rate such as SOFR + spread) | Dividends received **reduce** the debit balance. A dividend not credited on time = interest wrongly charged to the client |
| **Short financing / Short rebate** | Short-sale proceeds stay at the PB; the client receives a **rebate** (interest) minus the **borrow fee** | The dividend debited on shorts **reduces** the client's credit |
| **Stock loan fees** | The PB borrows securities from the street (or uses client inventory) and lends them to the short client, earning the spread. Easy securities = **General Collateral (GC)**, scarce ones = **Hard-to-Borrow (HTB) / Specials** | **CAs make securities scarce**: ahead of a merger, a vote or a rights issue, lenders recall and fees spike |
| **Rehypothecation / Internalization** | The PB reuses long clients' securities to cover other clients' shorts (**internalization**) or lends them to the street | This is what creates **substitute payments** and the **elections on lent securities** problems |
| **Synthetic financing** | Financing spread built into the swap | **Pass-through** dividends, 871(m), adjustments |
| **Commissions / Execution** | On order execution | Indirect |
| **Services** (Cap Intro, reporting, consulting) | Client retention | Indirect |

**Main cost:** the **balance sheet** and **regulatory capital**. A PB wants to maximize revenue per dollar of balance sheet. That is why **synthetic** (swaps) has grown so much: it often consumes less balance sheet than **cash prime** (physical securities).

> **Business → your project:** every unresolved CA break can carry a **direct cost** (loss), a **funding cost** (cash that isn't there, interest charged incorrectly), a **capital cost** (see §10J) and a **relationship cost** (an unhappy client who moves its assets to another PB).

---

## 10C. Front-to-back organization of a PB

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-22** (sheet `P3bis_PB_Profondeur`)

| Function | Role | Interaction with CAs |
|---|---|---|
| **Sales / Client Coverage** | Commercial relationship with funds | Escalation of client complaints about CAs; pressure on quality |
| **Client Onboarding / KYC** | Account opening, legal and tax documentation | **Tax status** (W-8/W-9), default options, notification channels → CA input data |
| **Legal / Documentation** | **Prime Brokerage Agreement**, **ISDA** (swaps), **MSLA / GMSLA** (securities lending), **Customer Account Agreement** | Defines **contractually** how CAs are handled (pass-through, elections, deadlines, rehypothecation) |
| **Margin & Credit Risk** | Daily margin calculation, margin calls, limits | CAs change prices, quantities and identifiers → risk of **false margin calls** |
| **Equity Finance / Securities Lending desk** | Manages borrows and loans, locates, fees | Recalls, manufactured payments, elections on lent securities, loan transformations |
| **Synthetic / Delta One desk** | Manages client swaps and their hedges | Pass-through, notional adjustments, Extraordinary Events, 871(m) |
| **Treasury / Funding** | Funds the PB balance sheet | Large CA cash flows (special dividends, cash mergers) |
| **Collateral Management** | Manages collateral received and posted (tri-party, CCP, counterparties) | Collateral securities hit by a CA → **substitution** |
| **Operations**: Clearing & Settlement, **Corporate Actions**, Cash management, Reconciliations, Stock record | Post-trade execution | **Your scope** |
| **Client Service** | Daily point of contact for funds and admins | Receives **CA queries** |
| **Tax** | Withholding, reporting, QI/QDD status | Applicable rates, substitute payments, 871(m) |
| **Product Control / Finance** | P&L, accounting | Accruals, write-offs |
| **Regulatory Reporting / Compliance** | 15c3-3, 15c3-1, Reg SHO… | Aged breaks → **capital charges**, 14e-4 compliance |
| **Technology (ITO)** | Systems and automation | **You** |

---

## 10D. The daily trade cycle in PB

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-23** (sheet `P3bis_PB_Profondeur`)

```
T (trade date)
 1. The fund executes with an Executing Broker (EB): "buy 100,000 XYZ"
 2. The EB "gives up" the trade to the PB → the PB affirms/accepts the trade
 3. If it's a short sale: the fund must have obtained a LOCATE (Reg SHO)
    from the PB BEFORE the order → the Equity Finance desk confirms availability
 4. The trade goes into clearing (NSCC) → settlement at DTC on T+1

Evening of T
 5. Margin calculation on the full portfolio (closing prices)
 6. Financing accrual (debit/credit interest, borrow fees)
 7. Income accruals: DIVIDENDS RECEIVABLE / PAYABLE for securities going ex on T+1   ← CA
 8. Reporting to the client and its administrator (positions, cash, accruals, CA files)

T+1 (settlement)
 9. Delivery/receipt at DTC; securities borrowed to cover the short
10. Any fails → handled by settlement; if a fail spans a record date → CLAIM          ← CA
11. The fund admin reconciles its books against PB files → queries if there's a gap  ← CA
```

> **Why this matters to you:** the files sent every night to the client and its administrator contain **the CA accruals**. If your pipeline catches a break **before** these files go out, the error is fixed before the client sees it.

---

## 10E. Margin and risk

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-24** (sheet `P3bis_PB_Profondeur`)

### US margin regimes (simplified)

| Regime | Principle |
|---|---|
| **Reg T** (Federal Reserve) | Classic **initial** margin: 50% for buying stocks |
| **FINRA Rule 4210** | Minimum **maintenance** margin (25% on longs, more on shorts) |
| **Portfolio Margin** | **Risk-based** margin on the overall portfolio (stress scenarios), more favorable to hedged portfolios |
| **House margin** | The PB's internal, stricter rules: **concentration**, **liquidity**, illiquid or distressed securities |

### How CAs disrupt margin

| Event | What happens | Risk if mishandled |
|---|---|---|
| **Dividend (ex-date)** | The price drops by the dividend amount; to offset it, a **dividend receivable** is created | If the accrual isn't booked: the long portfolio's value **drops artificially** → **false margin call**. For a short, the reverse: exposure drops but the dividend owed doesn't show → **risk understated** |
| **Split / Reverse split** | Price and quantity are adjusted **simultaneously** | If the price is adjusted but not the quantity (or vice versa): exposure **multiplied or divided by N** → absurd margin call or hidden risk |
| **Spin-off** | A new stock arrives, sometimes **without a price** for the first few days (*when-issued*) | New position valued at **zero** → false margin call; or position not recognized by the risk engine |
| **Stock merger** | Shares of A are replaced by shares of B | If CUSIP B isn't set up in reference data: **orphan** position, not margined |
| **Cash merger** | Securities are replaced by cash | Collateral transformed; the **gap** between effective date and pay date creates temporary exposure |
| **Tender (tendered shares)** | Tendered shares are **blocked** (*tendered, in contra*) until the result | They can no longer be **freely** sold, lent or used as collateral → liquidity and margin impact |
| **Rights issue** | Rights have value and an **expiration date** | Rights not valued or wrongly lapsed = loss of collateral value |
| **Delisting / Bankruptcy** | The security becomes illiquid, its price collapses | Collateral loses its value; **100% haircut**; massive margin calls |
| **Large special dividend** | Sharp price drop | Adjustment of options (OCC) and derivatives; if missed, false risk |

> **Automatable control idea:** on each ex-date or effective date, check that adjusted positions (quantity, price, CUSIP, accrual) are **consistent across the CA system, the margin system and the stock record** (see §10N).

---

## 10F. Equity Finance and Stock Loan

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-25** (sheet `P3bis_PB_Profondeur`)

### The mechanics

| Concept | Description |
|---|---|
| **Locate** | Before a short sale, the client must get confirmation from the PB that the security can be borrowed (**Reg SHO**, Rule 203) |
| **Sources of securities** | 1) **Client inventory** (rehypothecable long securities of other clients: **internalization**); 2) **Firm inventory**; 3) **Street borrow** from **agent lenders** (custodians lending the securities of pension funds, insurers…) |
| **GC vs Specials / HTB** | Easy-to-borrow security (low fee) vs scarce (high fee, sometimes tens of % per year) |
| **Recall** | The lender recalls its securities; the PB must replace them or force the client to buy back (**buy-in**) |
| **Close-out (Reg SHO Rule 204)** | Obligation to close out **fails to deliver** within strict timeframes |
| **Rebate / Fee** | Compensation on cash collateral and lent securities |
| **Loan collateral** | The PB posts cash or securities to the lender (generally 102% for USD), with a **daily mark-to-market** |

### How CAs impact Equity Finance

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-26** (sheet `P3bis_PB_Profondeur`)

| Situation | Impact |
|---|---|
| **Dividend record date** | Manufactured payments are **owed** to lenders. Some lenders **recall** their securities ahead of the record date (tax reasons or internal policy) → recalls to manage |
| **Shareholder meeting (proxy)** | Lenders (and long clients) who want to **vote** recall the securities → temporary scarcity, higher fees |
| **Announced merger / tender offer** | Merger arb funds **short the acquirer** → strong borrow demand → the security becomes **HTB**. Lenders recall to participate in the offer |
| **Voluntary event on lent securities** | The lender instructs the borrower, who must deliver the **economic outcome** of its choice. If the PB can't deliver → **liability / claim** |
| **Rights issue** | Borrow demand for rights/stock arbitrage; then loans must be adjusted (manufactured rights) |
| **Split** | The **loan quantity** must be multiplied and the **collateral price** adjusted. If the loan isn't adjusted → loan and collateral break |
| **Stock merger** | The loan in A becomes a loan in B (**loan transformation**), with a new CUSIP, a new quantity and cash in lieu of fractions |
| **Cash merger / Delisting** | The loan is **closed**: the borrower returns cash instead of securities |
| **Tender and shorts (Rule 14e-4)** | In the US, it is **forbidden to tender more than your net long position** (*short tendering*). A client long 500,000 and short 200,000 in the same security can tender only **300,000**. The PB must check client elections |

> **Watch point:** Equity Finance often uses **its own system** (and sometimes a market platform for loan management). Loan positions must be **reconciled against the CA system** on the record date. Otherwise: misallocated substitute payments and missing manufactured payments.

---

## 10G. Synthetic Prime in detail

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-27** (sheet `P3bis_PB_Profondeur`)

### Why clients use synthetic

- **Balance sheet efficiency** for the PB, hence better pricing for the client.
- **Access** to hard-to-reach markets (foreign markets, local constraints).
- **Operational simplicity** for the client: no settlement, no custody, no proxy.
- Built-in **leverage**.
- **Tax** reasons (regulated, notably by **871(m)** in the US).

### Mechanics of a portfolio swap

```
Client (hedge fund)                        PB (Synthetic desk) 
   │   receives the performance of XYZ       │
   │   + DIVIDENDS (pass-through)  ◄─────────┤  holds the physical HEDGE (100,000 XYZ)
   │   pays financing (SOFR + spread)───────►│  receives the actual dividends at DTC
   │                                          │
   └── Periodic resets (monthly…): settlement of P&L, financing and accumulated dividends
```

### How CAs impact synthetic

| Event | Swap treatment (per ISDA contract / confirmation) | Common breaks |
|---|---|---|
| **Cash dividend** | **Dividend amount** = X% of gross (100% for some clients, net of withholding for others); paid on the pay date or at **reset** | Wrong pass-through %; 871(m) withholding not applied; payment date inconsistent with the reset |
| **Dividend on a short swap** | The client **pays** the dividend | Wrong amount or sign |
| **Split / Stock dividend** | **Adjustment** of the number of reference shares and the initial price (notional unchanged) | Adjustment not applied → client P&L wrong |
| **Spin-off** | The swap is adjusted: either a new line on the spin-off, or a price adjustment | New line not created → exposure lost |
| **Merger / Tender (Extraordinary Events)** | Per the contract: **adjustment** (*Calculation Agent Adjustment*), **modified calculation**, or **termination** (*cancellation and payment*) | Treatment not aligned with what happens to the physical hedge |
| **Elections** | The client does not "legally" elect on a swap, but the contract or practice may let it **express a preference**; the PB elects on its **hedge** | Client choice ≠ choice applied to the hedge → economic mismatch |
| **Proxy** | The client has **no** voting right through a swap | Client requests to handle (governance question) |

> **Key recon (B11):** for each event, **what the PB receives on the hedge** must cover **what the PB pays the client on the swap** (+ withholding remitted to the IRS). Structural differences (tax, pass-through %) must be **explained**; anything else is a break.

---

## 10H. Treasury, funding and collateral

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-28** (sheet `P3bis_PB_Profondeur`)

| Topic | CA impact |
|---|---|
| **Cash forecasting** | Treasury needs to know **upcoming CA flows** (dividends receivable and payable, cash mergers, bond redemptions). A large special dividend or a cash merger of several hundred million dollars changes the day's funding need |
| **Cash not received** | A payment break (B5) = expected cash that isn't there → funding cost, even an **overdraft** on the DTC account |
| **Collateral posted** (tri-party, CCP, repo counterparties) | If a security posted as collateral has a CA (merger, delisting, bond call), it must be **substituted** or the flow managed (manufactured) |
| **Collateral received** | Its value changes with CAs; collateral must be revalued |
| **FX** | Events in foreign currencies (ADRs, foreign securities) → conversions and FX risk |
| **Payments to clients** | The PB often credits clients on the pay date **even if the market hasn't paid yet** (*contractual income*) or only on receipt (*actual income*), depending on internal policy **[TO BE CONFIRMED]**. Under *contractual*, every missing market payment becomes a **cash hole** and a loss risk |

> **Contractual vs Actual settlement:** this is a **key question** to ask internally. It changes the nature of the risk. Under *contractual*, B5 breaks represent money **already advanced** to the client.

---

## 10I. Reporting, NAV and client experience

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-29** (sheet `P3bis_PB_Profondeur`)

### What the client receives (in general)

- Daily files: **positions**, **trades**, **cash balances**, **financing accruals**, **income accruals** (dividends and coupons receivable and payable), **CA notifications**, **projected income**.
- Event notifications (portal, email, SWIFT for some clients).
- Election and payment confirmations.
- Monthly statements, annual tax reporting (1099 / 1042-S).

### Why it's sensitive: the **NAV**

Hedge funds compute a **NAV** (*Net Asset Value*), often monthly, sometimes more frequently. Their investors subscribe and redeem at that NAV.

- A **missing dividend** or a **wrong accrual** in the PB files → the administrator finds a gap → **query** to the PB.
- If the gap isn't resolved before the NAV close, the fund has to **estimate**, or even **restate its NAV** later. Investors and the fund's auditors take a very dim view of that.
- **Month-end periods are therefore critical:** every CA break affecting clients must be explained **before** NAVs close.

### Administrator queries

Typical questions: "Why is the XYZ dividend net of 30%?" "Why did the accrual disappear without a payment?" "Why does the post-split quantity differ from our calculation?" "Why did we receive a substitute payment and not a dividend?"

> **Product idea:** a **client-oriented** break report (by fund) would let Client Service respond **proactively**, even before the query.

---

## 10J. PB-specific regulation

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-30** (sheet `P3bis_PB_Profondeur`)

| Rule | Content (simplified) | Link with CAs |
|---|---|---|
| **SEC Rule 15c3-3** (Customer Protection) | Possession/control of fully paid and excess margin client securities; **Reserve Formula** (periodic calculation of amounts owed to clients, to be locked in a special account) | Dividends and income owed to clients feed the calculations; securities hit by a CA must stay in possession/control (watch reorgs and securities in transit) |
| **SEC Rule 15c3-1** (Net Capital) | Minimum broker-dealer capital, with **deductions** for certain risky items | Unresolved securities differences (**short securities differences**) and aged receivables can trigger **capital charges** beyond certain timeframes. **An aging break can cost capital** |
| **SEC Rule 17a-13** | **Quarterly securities count** and stock record reconciliation | CA-related position differences must be explained |
| **Reg T / FINRA 4210** | Margin | See §10E |
| **Reg SHO** | Locate, close-out of fails, short sale rules | Fails around record dates → claims, buy-ins |
| **SEC Rule 14e-4** | Prohibits **tendering more than your net long position** | Mandatory check on client elections |
| **IRS (Chapter 3 / FATCA / 871(m) / QI / QDD)** | Withholding and tax reporting | See §17 |
| **Books & Records (17a-3 / 17a-4)** | Record retention | Traceability of CAs and elections: your pipeline must **archive** its inputs and outputs |

> **Message to carry to the sponsor:** reducing break aging is not just a productivity gain. It also has a **regulatory and capital benefit**. Validate the exact thresholds with Regulatory Reporting / Finance **[TO BE CONFIRMED]**.

---

## 10K. CA impact matrix across PB functions

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-31** (sheet `P3bis_PB_Profondeur`)

| Event | Stock record / Positions | Margin / Risk | Equity Finance (loans/borrows) | Synthetic (swaps) | Client / Admin / NAV | Tax |
|---|---|---|---|---|---|---|
| **Cash dividend (DVCA)** | Eligible positions on the record date | Accrual required on the ex-date | Manufactured owed/received; possible recalls | Dividend pass-through | Accruals in the files; payment | Withholding per status; substitute vs real dividend |
| **Large special dividend** | Same + possible **due bill** | Sharp price drop; OCC option adjustment | Large manufactured payments | Pass-through + possible adjustment | Visible NAV impact | Characterization (ROC?) |
| **Split** | Quantities × N, **due bill** if ≥ 25% | Price ÷ N **in sync** | Loans × N, collateral adjusted | Number of reference shares adjusted | Quantities in the files | — |
| **Reverse split** | Quantities ÷ N, **fractions → CIL** | Same | Loans adjusted, fractions | Adjustment | CIL to explain | CIL = sale (1099-B) |
| **Spin-off** | New position, new CUSIP | New security **to be priced** | New loans on the spin-off security | New swap line or adjustment | New position + cost basis | Cost basis allocation (8937) |
| **Cash merger** | Positions closed out, cash received | Collateral turned into cash | **Loans closed** in cash | Termination or adjustment (ISDA) | End of position | Taxable sale |
| **Stock / mixed merger** | Old CUSIP → new; fractions | New security in the risk engine | **Loan transformation** | Adjustment or termination | New position | Depends on structure |
| **Tender / Buyback (voluntary)** | Securities **blocked** after tender | Tendered securities less liquid | Recalls, **liability** on lent securities, **14e-4** | Choice on the hedge ≠ client preference | **Client election**, deadline, proration | Taxable sale |
| **Rights issue** | Rights credited, then shares | Rights to be priced, expiration | Manufactured rights | Adjustment | Election (exercise/sell/lapse) | — |
| **Name / CUSIP change** | Old → new mapping | Reference data updated | Loans on the new CUSIP | Mapping | Files with the new CUSIP | — |
| **Delisting / Bankruptcy** | Security sometimes removed from DTC | 100% haircut, margin calls | Loans hard to close out | Extraordinary event | Tricky NAV valuation | Losses (worthless) |
| **Bond coupon / call** | Nominal, DTC lottery (partial call) | Bond collateral value | Manufactured coupon (repo) | Credit swaps / bond TRS | Interest accruals | 1099-INT / 1042-S |
| **Shareholder meeting (proxy)** | Voting positions (excluding lent securities) | — | **Recalls** to vote | No voting right | Vote requests, activists | — |

---

## 10L. Timeline of an event from the PB's perspective

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-32** (sheet `P3bis_PB_Profondeur`)

Example: a **voluntary cash tender offer** on a US stock held by several funds (longs, shorts, loans, swaps).

| When | Who | What |
|---|---|---|
| **D-30** Announcement | Event Management | Capture (Bloomberg, DTC), event setup, golden copy |
| D-30 | Equity Finance | Anticipates higher borrow demand; watches recalls |
| D-29 | CA Ops / Client Service | **Notification to affected clients** (longs, swaps) with the **client deadline** |
| D-29 | Risk | Review of concentrations in the target and the acquirer |
| D-20 → D-5 | Clients | Send their elections; merger arb elects very early |
| D-5 | CA Ops | **Election checks**: election ≤ available position; **14e-4** compliance (net long); lent securities identified |
| D-5 | Equity Finance | **Recalls** of the lent securities needed for the elections |
| D-3 → D-1 | CA Ops | Chasing clients who haven't responded; preparing instructions |
| **D** Expiration (often 5 p.m. or midnight NY time) | CA Ops | **ATOP** instructions at DTC; **guaranteed delivery / protect** for securities not yet returned |
| D | Synthetic desk | Election decision on the physical **hedge** |
| D+1 → D+2 | Equity Finance / Ops | **Covering the protects** (did the recalled securities come back in time?) |
| D+2 → D+5 | Agent | Result, possible **proration** |
| D+5 | CA Ops | Client allocations per the proration; position updates |
| D+5 → D+7 | DTC | **Payment** of cash and return of unaccepted shares |
| D+7 | CA Ops / Recon | **Expected vs received recon** (cash and returned shares) |
| D+7 | Synthetic | Swap settlement/adjustment |
| D+7 | Equity Finance | Loans closed or transformed; **liabilities** settled |
| D+8 | Client / Admin | Fund-side recon → possible queries |
| D+30 | Claims | Follow-up of open claims (lending counterparties, fails) |

---

## 10M. Typical PB incident scenarios

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-33** (sheet `P3bis_PB_Profondeur`)

| # | Scenario | Cause | Consequence |
|---|---|---|---|
| 1 | **Missed election** by a merger arb fund | Notification not received, internal deadline miscalculated | The fund misses the offer → **the PB compensates** the difference in value (can reach millions) |
| 2 | **Over-election** | Election on a position already sold or lent | Instruction rejected or protect not covered → compensation |
| 3 | **Protect not covered** | Lent securities not returned in time | The client loses the benefit of the offer → compensation |
| 4 | **Short tendering** | Election > net long position | **Regulatory violation** (14e-4) |
| 5 | **Split out of sync** | Price adjusted in the margin system but not the quantity | Erroneous margin call → client complaint, even a wrongful forced liquidation |
| 6 | **Dividend not credited** to the client | Missing accrual or unallocated payment | Admin query; NAV error; interest charged incorrectly |
| 7 | **Wrong withholding rate** | Tax status not up to date | Under-withholding (IRS risk, penalties) or over-withholding (client complaint, reclaim) |
| 8 | **Manufactured payment not claimed** | Loan break not detected | Outright loss after a certain time |
| 9 | **Swap not adjusted** after a spin-off | Swap line not created | Client P&L wrong; dispute with the client |
| 10 | **Unanticipated recall** before a vote | Activist wants to vote | Fail, buy-in, friction with the client |
| 11 | **Event not captured** on an illiquid security | Missing from a source | Nothing is done → loss discovered late |

> These scenarios are **your pitch** to the sponsor: every automated control should map to one or more of these risks.

---

## 10N. Implications for your project

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-34** (sheet `P3bis_PB_Profondeur`)

**PB-specific** use cases, added to the backlog in §32 (not replacing it):

| # | PB use case | Sources | What it detects | Scenarios covered (§10M) |
|---|---|---|---|---|
| UC11 | **Election availability check**: client elections vs available position (unlent longs + recallable securities) vs net long (14e-4) | Elections, stock record, Equity Finance | Over-election, short tendering, protects at risk | 2, 3, 4 |
| UC12 | **Recall planner**: lent securities needed for upcoming elections or votes | Voluntary events and meetings, loans, elections | Recalls to launch at D-N | 3, 10 |
| UC13 | **CA consistency check cross-systems** on ex-date / effective date: consistent quantities, prices, CUSIPs, accruals across CA system, margin, stock record, Equity Finance and swaps | All systems | Out-of-sync splits, unpriced spin-offs, unmapped CUSIPs | 5, 9 |
| UC14 | **Income allocation check**: real dividend vs substitute payment vs short charges vs manufactured | Positions, loans, payments | Inconsistent allocations | 6, 8 |
| UC15 | **Swap vs hedge dividend recon** (already UC7, detailed PB version: pass-through % by client, 871(m)) | Swap system, DTC payments, contracts | Unexplained differences | 9 |
| UC16 | **Client-facing break report** by fund, before NAV close (month-end) | Breaks + account/fund mapping | Breaks to explain proactively to clients | 6, 7 |
| UC17 | **Event coverage check**: does every held or lent security have its events captured in all sources? | Positions, Bloomberg, DTC | Missing event | 11 |

**Additional PB questions to ask internally:**
- [ ] Client income payment: **contractual** or **actual**?
- [ ] How is Equity Finance informed of upcoming elections and votes?
- [ ] Where does the 14e-4 control live today?
- [ ] Which clients are on **synthetic**, and with what pass-through %?
- [ ] Have legacy clients and systems from other platforms been fully migrated?
- [ ] Which CA breaks feed capital or reserve calculations (15c3-1 / 15c3-3)?
- [ ] NAV calendar of the main clients (month-end) and volume of administrator queries?

---

# PART IV — BY ASSET CLASS

## 11. Equities, ADRs, ETFs/funds, REITs/MLPs, SPACs

### 11.1 US equities

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-35** (sheet `P4_Classes_Actifs`), **EX-36** (sheet `P4_Classes_Actifs`)

- Volume is dominated by **DVCA**.
- Frequent events: **splits**, **spin-offs**, **mergers**, **tender offers**, **buybacks**.
- **Fractions:** a 1:10 reverse split on 1,005 shares gives 100.5. The issuer pays the fraction as **cash in lieu (CIL)**, often at the aggregate sale price of the fractions. For tax purposes it is a **sale** (reported on Form 1099-B).
- **Spin-off:** you also need to handle the **cost basis allocation** between parent and spin-off (IRS Form **8937** published by the issuer).

### 11.2 **ADRs** (*American Depositary Receipts*)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-37** (sheet `P4_Classes_Actifs`)

- US certificates representing foreign shares, issued by a **depositary bank**.
- An ADR dividend = foreign dividend, converted into USD (**FX**), minus **foreign withholding tax**, minus **ADR fees** (depositary fees per ADR).
- **Common breaks:** FX rate, foreign withholding rate, fees, ADR/ordinary share **ratio**. ADR dates can differ from those of the local security.

### 11.3 ETFs and funds

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-38** (sheet `P4_Classes_Actifs`)

- Distributions of dividends, **capital gains** (often at year-end), **return of capital**.
- **Year-end reclassification:** the tax character (ordinary dividend / ROC / capital gain) is sometimes **only known early the following year** → corrections to 1099 reporting.

### 11.4 REITs and MLPs

- **REITs:** distributions are often reclassified at year-end (ROC, capital gain), same logic as funds.
- **MLPs** (*Master Limited Partnerships*): these are partnerships, not corporations. Distributions are largely *return of capital*, reporting goes through **K-1**, and there are **specific withholding regimes for non-US holders**. This is a pure Tax topic **[TO BE CONFIRMED WITH TAX]**.

### 11.5 **SPACs**

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-39** (sheet `P4_Classes_Actifs`)

- **Units** = share + fraction of a warrant, which **separate** (*unit split*).
- **Redemption rights:** before the *business combination*, holders may ask to have their shares redeemed at their pro rata share of the trust (a voluntary event with a deadline).
- Then the merger (**de-SPAC**): CUSIP and name change, warrants converted.
- SPAC arbitrage funds are typical PB clients.

### 11.6 Rights and warrants

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-40** (sheet `P4_Classes_Actifs`)

- **Rights offering:** distribution of rights (RHDI) → exercise (EXRI), sale (SLLE) or lapse (LAPS). Sometimes with **oversubscription**.
- **Warrants:** exercise (EXWA), ratio adjustments after other CAs.

---

## 12. Fixed Income: corporates, munis, Treasuries, MBS/ABS

### 12.1 Corporate bonds (at DTC)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-41** (sheet `P4_Classes_Actifs`), **EX-42** (sheet `P4_Classes_Actifs`)

- **INTR**: coupons. Amount = face × rate × fraction per the **day count convention** (30/360, ACT/360, ACT/365…).
- **Floating rate notes:** rate reset each period (today on **SOFR** for USD) → frequent **rate breaks**.
- **REDM** (maturity), **MCAL** (full call), **PCAL** (partial call).
  - **Partial call at DTC:** DTC allocates the redeemed securities among participants by **lottery**, then the PB must **reallocate** among its clients using a fair method. **Frequent breaks** on this allocation.
- **BPUT**: holder put.
- **CONS**: consent solicitations (often with a **consent fee**).
- **TEND / EXOF**: debt buyback or exchange offers, sometimes with an **early tender premium** and an early deadline.
- **Default / Bankruptcy:** missed coupons, restructuring distributions (new securities, cash, *equity*). Very manual.
- **PIK** (*Payment in Kind*): coupon paid in securities.

### 12.2 Municipal bonds

- Official source: **EMMA (MSRB)**.
- Specifics: **pre-refunding / escrow** (CUSIP changed), **mandatory puts / tenders**, **sinking funds**.
- The tax treatment of muni interest (often federally tax-exempt) affects reporting.

### 12.3 US Treasuries

- Held via **Fedwire**, not at DTC.
- Few events: coupons and maturities. But watch **TIPS** (inflation-indexed principal) and **STRIPS**.

### 12.4 **MBS / ABS** (*Mortgage / Asset-Backed Securities*)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-43** (sheet `P4_Classes_Actifs`)

- Principal is paid down **every month** according to a **factor** (pool factor) published monthly.
  - `Current face = Original face × Factor`
  - `Principal payment = Original face × (Factor_prev − Factor_new)`
- **Agency MBS** (Fannie Mae, Freddie Mac, Ginnie Mae): at Fedwire. **Non-agency** (private label) and ABS: generally at DTC, with data from **trustee reports**.
- Record date generally at the end of the previous month, with a **payment delay** specific to each program type.
- **Common breaks:** factor differs between sources, factor published late, *interest shortfall*, retroactive adjustments.
- **TBA** (*To Be Announced*): forward trades on MBS; CAs affect allocation timing.

---

## 13. Listed and OTC derivatives

### 13.1 Listed options (OCC)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-44** (sheet `P4_Classes_Actifs`)

- The **OCC** adjusts contracts after certain CAs and publishes **Information Memos**.
- **Adjusted:** splits, reverse splits, spin-offs, mergers, **special dividends** (above a threshold set by the OCC). In general, **ordinary dividends are not adjusted**.
- After adjustment you get an **adjusted option**: new symbol, new **deliverable** (e.g., 100 shares + cash + spin-off shares), modified multiplier.
- For a PB: client option positions, their margin and their valuation must be updated.

### 13.2 Futures

- Single-stock futures: similar adjustments.
- Index futures: the **index provider** handles the CA impact within the index (*index divisor*).

### 13.3 OTC (swaps, TRS, CFDs)

- Governed by the **ISDA Equity Derivatives Definitions** (or the confirmation): *Potential Adjustment Events*, *Extraordinary Events* (**Merger Event**, **Tender Offer**, **Nationalization**, **Insolvency**, **Delisting**).
- The **Calculation Agent** (often the bank) determines the adjustment.
- **Dividend pass-through** on swaps: *dividend amount* = % of the gross or net dividend, per the contract.
- **Key recon:** dividends received on the physical **hedge** vs dividends paid on the **swap**.

### 13.4 Convertibles

- **Conversion ratio** adjustment after CAs on the underlying (dividends above a threshold, splits…).
- **Deemed distributions** (Section 305(c)): the ratio adjustment can be treated for tax purposes as a deemed dividend, with withholding for non-US holders **[TO BE CONFIRMED WITH TAX]**.

---

## 14. Securities Lending and Repo

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-45** (sheet `P4_Classes_Actifs`)

| Concept | Description |
|---|---|
| **Lender / Borrower** | The lender transfers legal title but keeps the **economic benefit** |
| **Manufactured payment** | The borrower pays the lender the equivalent of each distribution |
| **Recall** | The lender recalls the securities (to vote, elect or sell) |
| **Voluntary CA on loaned securities** | The lender instructs the borrower, who must deliver the economic outcome |
| **Income tracking** | Tracking manufactured payments owed and received. **A major source of breaks and claims** |
| **Repo** | Same logic: the party that temporarily transfers the securities keeps the economic benefit (*manufactured coupon*) |
| **Mark-to-market** | After a CA, the value of the loan and its collateral changes (split, spin-off) → readjustment |

In the US, DTC offers a **Claim Connect** service to facilitate settlement of certain income claims between participants.

---

# PART V — DAY-TO-DAY OPERATIONS

## 15. The detailed lifecycle

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-46** (sheet `P5_Operations`)

| # | Step | Inputs | Outputs | Controls | Possible breaks |
|---|---|---|---|---|---|
| 1 | **Event capture** | Bloomberg, DTC, Broadridge, custodians, agents | Raw events | Completeness (no held security without a known event) | Missing event |
| 2 | **Scrubbing / Validation** | Multi-source raw events | **Golden copy** | Source comparison, 4-eyes | **Event data breaks** |
| 3 | **Position identification** | Stock record, client positions, lending, swaps | Eligible positions at record date | Stock record vs DTC recon | **Position breaks** |
| 4 | **Entitlement calculation** | Golden copy + positions + tax status | Expected entitlements (gross, tax, net) per account | Recalculation, comparison with DTC/custodian entitlement | **Entitlement breaks** |
| 5 | **Client notification** | Golden copy + positions | Notifications (portal, email, SWIFT) | Completeness, timeliness | Client not notified |
| 6 | **Election collection** | Client responses | Aggregated elections | Election ≤ available position; deadline | **Over-election**, late election |
| 7 | **Instruction** | Aggregated elections | Instructions to DTC (e.g., via **ATOP** for tenders) / custodian | MT567 statuses | Instruction rejected |
| 8 | **Accrual booking** | Entitlements | *Income receivable / payable* entries on the ex-date | Accounting recon | **Accrual breaks** |
| 9 | **Payment / Settlement** | DTC/custodian payments, MT566 | Cash/securities received and allocated to clients | **Expected vs received** recon | **Cash / stock breaks** |
| 10 | **Client allocation** | Amounts received | Client credits/debits | Sum of allocations = amount received | **Allocation breaks** |
| 11 | **Claims** | Breaks with counterparties | Claims raised and received | Aging tracking | Unsettled claims |
| 12 | **Tax reporting** | Payments, tax statuses | 1099, 1042-S | — | Reclassifications |
| 13 | **Close-out** | — | Event closed | No open breaks left | — |

---

## 16. How a PB CA team is organized, and a typical day

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-47** (sheet `P5_Operations`)

### 16.1 Typical teams (your firm's actual organization is **[TO BE CONFIRMED INTERNALLY]**)

| Team | Scope |
|---|---|
| **Announcements / Event Management** | Capture and validation (golden copy) |
| **Income / Dividends** | Mandatory cash events, accruals, payments |
| **Mandatory Reorg** | Splits, mergers, CUSIP changes |
| **Voluntary Reorg / Elections** | Tenders, rights, elections, deadlines, protects |
| **Claims** | Market claims, manufactured payments, compensation |
| **Tax** | Withholding, documentation, reclaims, reporting |
| **Proxy** | Votes |
| **Client Service** | Client and fund admin queries |
| **Offshore** | Part of the processing is often done in offshore centers (India, Portugal…) **[TO BE CONFIRMED]** |

### 16.2 A typical day (NYC)

| Time | Activity |
|---|---|
| 7:00–9:00 | Review overnight feeds (Bloomberg, DTC), new announcements, updates |
| 9:00–10:00 | Review **today's payments** (pay date): expected vs received |
| 10:00–12:00 | Break investigation, emails to custodians, agents and lending counterparties |
| 12:00–2:00 | **Today's voluntary deadlines**: client chasers, election aggregation, ATOP instructions |
| 2:00–4:00 | Fund admin and client queries, claims |
| 4:00–6:00 | Prep for tomorrow's record date, position checks, management reporting |

**Where ops lose time (= your opportunities):** extracts from several systems, copy-paste into Excel, VLOOKUPs, manual break hunting, repetitive emails, aging tracked in shared files.

---

## 17. US taxation

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-48** (sheet `P5_Operations`)

### 17.1 Principles

| Beneficial owner | Baseline regime for US dividends |
|---|---|
| **US person** (valid W-9) | No withholding (except **backup withholding** at 24% if documentation is missing or invalid). Reporting on **1099-DIV** / 1099-INT / 1099-MISC / 1099-B |
| **Non-US person** (W-8BEN, W-8BEN-E, W-8IMY…) | **NRA withholding** at **30%** by default, reduced if a **tax treaty** applies (*treaty rate*, often 15% for portfolio dividends). Reporting on **1042-S** |
| Portfolio **interest** (*portfolio interest exemption*) | Often exempt for non-US persons, subject to conditions |

### 17.2 Key concepts

- **QI** (*Qualified Intermediary*): intermediary status with the IRS that allows applying treaty rates without passing on beneficial owner identities.
- **QDD** (*Qualified Derivatives Dealer*): status related to derivatives and 871(m).
- **FATCA** (Chapter 4) and **NRA** (Chapter 3): two coexisting withholding regimes.
- **Section 871(m)**: withholding on **dividend equivalents** paid to non-US persons through derivatives on US equities (notably delta-one swaps, with phase-in rules for the rest). **Very important in synthetic PB.**
- **Section 305(c)**: deemed distributions (convertibles, ratio adjustments).
- **Tax reclaim**: recovering over-withheld tax (especially on foreign securities).
- **Income characterization**: the same payment can be a dividend, ROC, capital gain, interest or substitute payment. Each has its own tax treatment.

### 17.3 Typical tax breaks

| Symptom | Likely cause |
|---|---|
| Net received = gross × 70% instead of 85% | Tax documentation not applied, wrong treaty rate |
| Withholding on a US client | Missing W-9 → backup withholding |
| Withholding on a ROC | Reclassification not yet known |
| Gap on a swap | 871(m) calculation |

---

## 18. Accounting: accruals and income

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-49** (sheet `P5_Operations`)

- On the **ex-date**: book an **income receivable** for longs and an **income payable** for shorts.
- On the **pay date**: settle the receivable against the cash received.
- If the cash doesn't arrive, the receivable stays open: an **aged receivable**, a break indicator.
- **Key recon:** *open accruals* vs *expected payments*. Any accrual still open after pay date + N days is a break to investigate.
- P&L impact: an unresolved break can end up as a **write-off** (loss) if the bank paid the client without collecting from the market.

---

# PART VI — BREAKS (the core of your project)

## 19. Break taxonomy

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-50** (sheet `P6_Breaks`)

| # | Family | What is compared | Granularity | Typical root causes | Timing |
|---|---|---|---|---|---|
| B1 | **Event data break** | Golden copy vs Bloomberg vs DTC vs Broadridge | Event | Different rate, dates, ratio, currency, options, status; missing event | From announcement, then on every update |
| B2 | **Missing event** | Held positions vs known events | Security | Security held with no event captured (or the reverse) | Before record date |
| B3 | **Position break** | Internal positions (stock record) vs DTC/custodian position on the record date | Security × location | Unsettled trades, fails, booking, lending not reflected | Record date |
| B4 | **Entitlement break** | Internal entitlement vs DTC/custodian entitlement | Event × depot account | B1 or B3, rounding, fractions | Between record date and pay date |
| B5 | **Cash break** | Expected vs received (MT566, nostro, DTC) | Event × account | Missing, partial or late payment, wrong rate, tax, currency | Pay date and after |
| B6 | **Stock break** | Securities expected vs received (splits, spin-offs, stock dividends) | Event × account | Ratio, fractions, CUSIP | Pay/effective date |
| B7 | **Tax break** | Expected tax (per client status) vs tax withheld | Event × client | Documentation, treaty rate, characterization | Pay date |
| B8 | **Allocation break** | Σ client allocations vs amount received | Event | Rehypo/substitute payment, rounding, shorts | After payment |
| B9 | **Short / Borrow break** | Dividends debited on client shorts vs paid to lenders | Event | Rate, positions, cover timing | Pay date |
| B10 | **Lending / Manufactured break** | Manufactured owed vs received (loans/borrows) | Event × counterparty | Loan position, rate, timing | Pay date + |
| B11 | **Swap vs Hedge break** | Swap dividends (pass-through) vs dividends received on the hedge | Event × swap | Pass-through %, 871(m), notional | Pay date |
| B12 | **Election break** | Client elections vs instructions sent vs available position | Voluntary event | Over-election, missing instruction, uncovered protect | Before the deadline (**critical**) |
| B13 | **Deadline risk** | Voluntary events with a close deadline and clients who haven't responded | Event × client | No client response | D-5 → D |
| B14 | **Accrual break** | Open accruals vs payments | Event | Payment not received, wrong accrual | After pay date |
| B15 | **Claim break** | Open claims (raised/received) | Claim | Counterparty doesn't pay | Aging |
| B16 | **Reference data break** | CUSIP/ticker/name after a reorg vs static data | Security | Missed update | Effective date |
| B17 | **Derivative adjustment break** | OCC adjustments vs option positions | Contract | Adjustment not applied | Effective date |

> **Suggested prioritization** (to validate with the sponsor): **B12/B13** (high loss risk, time-critical), **B5** (high volume, simple), **B1** (upstream, it prevents the rest), then B4, B7, B9, B10, B11.

---

## 20. Matching keys and tolerances

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-51** (sheet `P6_Breaks`)

### 20.1 Matching an **event** across sources

Order of preference:

1. **COAF** (if present in both sources)
2. **CUSIP/ISIN + CAEV + Pay date** (+ ex-date)
3. **CUSIP + CAEV + Record date**
4. Approximate (**fuzzy**) matching: CUSIP + type + dates within ±N days, for cases where one source has a wrong date

Pitfalls:

- Several events on the same day for the same security (regular + special dividend).
- CUSIP change (use a mapping table).
- Event type differences between vendors (one vendor says `CAPD`, another `DVCA` with a flag).

### 20.2 Matching a **payment**

- Key: **Event key + account + currency**
- Then compare amounts: **gross**, **tax**, **net**.
- Handle **many-to-one** (several partial payments for one entitlement) and **reversals** (payment cancelled then redone).

### 20.3 Tolerances (**[TO BE VALIDATED WITH OPS]**)

| Type | Typical tolerance |
|---|---|
| Cash rounding | ±$0.01 per line, or ±$1 per event |
| Rate | Compare to 6–8 decimals |
| Securities quantity | 0 (exact), except fractions handled as CIL |
| Dates | 0 days, unless an explicit tolerance applies (e.g., holiday) |

### 20.4 Automatic root-cause classification (*auto root-cause*)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-52** (sheet `P6_Breaks`)

Simple heuristic rules that ops really appreciate:

```
if received == 0                         → "Unpaid"
if received / expected ≈ 0.70 / 0.85     → "Tax rate difference"
if received == expected_on_other_qty     → "Position difference"
if |diff| < tolerance                     → "Rounding" (auto-close)
if pay_date > today                       → "Not yet due" (not a break)
if event status == CANC                  → "Cancelled event"
```

---

## 21. Claims management

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-53** (sheet `P6_Breaks`)

| Claim type | Situation |
|---|---|
| **Market claim** | Trade bought before the ex-date but settled after the record date: the seller received the distribution, the buyer claims it |
| **Reverse claim** | The reverse: the PB received it wrongly and must pay it back |
| **Due bill claim** | Due bill window (§4.4) |
| **Manufactured claim** | Lending / repo |
| **Fail claim** | Trade failing over the record date |
| **Transformation** | For reorgs: the pending trade in the old security becomes a trade in the new one |

**Metrics:** number and amount of open claims, **aging** (0–30, 31–60, 61–90, 90+ days), counterparties most in arrears. Old claims end up as **write-offs**.

---

## 22. KPIs, risks and controls

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-54** (sheet `P6_Breaks`)

### 22.1 Operational KPIs

- **STP rate** (% of events processed with no manual intervention)
- Number and amount of **open breaks**, by family
- Break **aging**
- **Time to detect** and **time to resolve**
- **Missed elections / late instructions** (target: 0)
- **Client queries** (volume, response time)
- CA-related **losses / write-offs**
- **FTE hours** spent per task (basis of the business case)

### 22.2 Risks

- **Missed election** (direct loss, client claim)
- **Incorrect entitlement** (unrecoverable overpayment)
- **Tax errors** (IRS regulatory risk, penalties)
- **Client communication failure**
- **Key-person dependency** (one person knows the Excel file)
- **EUC risk**: uncontrolled Excel macros (see §30)

---

# PART VII — WORKED EXAMPLES

## 23. Nine worked cases

### Case 1 — Dividend with long, short, rehypothecation and a non-US client

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-55** (sheet `P7_Cas_Pratiques`)

**Event:** XYZ `DVCA` $0.50/share. Record date = ex-date = 11/10. Pay date = 11/20.

| Account | Position | Status |
|---|---|---|
| Fund A (US, W-9) | +200,000 long, of which 80,000 rehypothecated and on loan | US person |
| Fund B (Cayman, W-8BEN-E, no treaty) | +100,000 long | Non-US |
| Fund C | −50,000 short | — |
| Street borrow (to cover C) | +50,000 | Counterparty: Lender L |
| Street loan (rehypo from A) | −80,000 | Counterparty: Borrower K |

**Expected DTC position:** 200,000 + 100,000 + 50,000 − 80,000 − 50,000 (sold by C, already delivered) = **220,000**

**Expected cash from DTC:** 220,000 × 0.50 = **$110,000**

**Allocations:**

| Flow | Calculation | Amount |
|---|---|---|
| Fund A, real dividend | 120,000 × 0.50 | $60,000 |
| Fund A, substitute payment | 80,000 × 0.50 | $40,000 (paid by K to the PB) |
| Fund B, gross | 100,000 × 0.50 = $50,000, 30% withholding = $15,000 | **$35,000 net** |
| Fund C, short debit | 50,000 × 0.50 | −$25,000 |
| PB → Lender L (manufactured) | 50,000 × 0.50 | −$25,000 |
| Borrower K → PB (manufactured) | 80,000 × 0.50 | +$40,000 |

**Possible breaks:** DTC pays $105,000 (DTC position at 210,000 → B3/B5); K doesn't pay (B10); Fund B is withheld at 15% instead of 30% even though there is no treaty (B7).

### Case 2 — 4:1 split with a due bill

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-56** (sheet `P7_Cas_Pratiques`)

**Event:** 4-for-1 split (`SPLF`). Record date 12/01, pay date 12/08, ex-date 12/09 (distribution ≥ 25% → ex-date after the pay date).

- A client sells 10,000 shares on 12/03 (settles 12/04, after the record date, but before the ex-date).
- The stock trades "cum": the buyer is entitled to the 30,000 additional shares.
- The seller, registered on the record date, receives the 30,000 shares on 12/08 but **owes them** → **due bill**.
- **Break if** the system doesn't generate the due bill: the selling client keeps 30,000 extra shares and the buyer claims them.

### Case 3 — Cash + stock merger with fractions

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-57** (sheet `P7_Cas_Pratiques`)

**Event:** A is acquired by B. For each A share: **$25 cash + 0.3375 B shares**. B price = $80.

Client: 1,001 A shares.

- Cash = 1,001 × 25 = **$25,025**
- B shares = 1,001 × 0.3375 = 337.8375 → **337 B shares** + **0.8375 fraction**
- Cash in lieu = 0.8375 × sale price of fractions (e.g., $80) = **$67.00**
- **Common breaks:** rounding of fractions at depot account level vs client level (the PB aggregates: 337.8375 + other clients' fractions can add up to whole shares at DTC!), new B CUSIP not mapped.

> **Subtle PB point:** DTC calculates fractions **on the PB's aggregated position**, but the PB must allocate **client by client**. The sum of client fractions ≠ the DTC fraction → a small structural difference to explain (allocation rule **[TO BE CONFIRMED]**).

### Case 4 — Tender offer with guaranteed delivery and proration

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-58** (sheet `P7_Cas_Pratiques`)

**Event:** partial tender offer at $50 for **60%** of XYZ shares (`TEND`, `VOLU`). Expires 12/15 at 5:00 p.m. NY time. Guaranteed delivery allowed, protect deadline at T+1.

- Fund M (merger arb) holds 1,000,000 shares, 400,000 of which are rehypothecated.
- It **elects** to tender everything.
- The PB instructs 600,000 via **ATOP** (available shares) + **400,000 as a protect** (guaranteed delivery).
- The PB **recalls** the 400,000 lent shares. If they come back before the protect deadline, they are delivered. Otherwise: **failed protect**, the client loses participation on those shares and the PB may have to compensate.
- The offer is oversubscribed: **proration** at 72%. Fund M receives 720,000 × $50 = **$36M**; 280,000 shares are returned.
- **Breaks:** elections > position (B12), uncovered protect (B12), proration result differs between the agent and the internal calculation (B5/B6).

### Case 5 — Rights issue

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-59** (sheet `P7_Cas_Pratiques`)

**Event:** 1 right (RHDI) per share; 5 rights buy 1 new share at $20. The right is tradable.

Client: 10,000 shares → 10,000 rights.

- EXER option: subscribes for 2,000 shares, pays $40,000.
- SLLE option: sells the rights.
- LAPS option: lets them expire (value lost!). For clients who don't respond, the PB's **default option** is crucial.
- OVER option: asks for more shares; proration possible.
- **Breaks:** rights credited ≠ expected, subscription cash debited but shares not received.

### Case 6 — Partial bond call at DTC

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-60** (sheet `P7_Cas_Pratiques`)

**Event:** `PCAL` on 20% of the issue, at 101.

- The PB holds $10M face (aggregated, for 5 clients).
- The **DTC lottery** selects $2.5M at the PB (not necessarily 20%!).
- The PB must allocate the $2.5M among its 5 clients using an **impartial** method (internal lottery or pro rata, per policy **[TO BE CONFIRMED]**).
- **Breaks:** internal allocation ≠ DTC amount, client informed of the wrong amount.

### Case 7 — MBS payment (factor)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-61** (sheet `P7_Cas_Pratiques`)

Original face $10,000,000. Previous factor 0.85, new factor 0.83. 5% coupon, monthly interest.

- Principal = 10,000,000 × (0.85 − 0.83) = **$200,000**
- Interest = 10,000,000 × 0.85 × 5% / 12 = **$35,416.67**
- **Typical break:** the factor used internally (vendor) is 0.8301 instead of 0.83 → $1,000 principal difference.

### Case 8 — ADR dividend

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-62** (sheet `P7_Cas_Pratiques`)

ADR ratio 1 ADR = 2 ordinary shares. Local dividend EUR 1.00/share. EUR/USD = 1.08. Foreign withholding 25%. Depositary fee $0.02/ADR.

- Gross per ADR = 2 × 1.00 × 1.08 = $2.16
- Withholding = $0.54
- Fee = $0.02
- **Net = $1.60/ADR**
- **Breaks:** different FX rate (conversion date), withholding at 15% (with treaty documentation) vs 25%.

### Case 9 — Equity swap (synthetic prime)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-63** (sheet `P7_Cas_Pratiques`)

Non-US client long 100,000 XYZ via a swap; contractual pass-through of 100% of gross; dividend $0.50. The dividend equivalent is subject to 871(m) at the applicable rate (here 30%, no treaty).

- Dividend amount owed to the client = $50,000, minus 871(m) withholding ($15,000) = **$35,000**
- Hedge: the PB holds 100,000 physical shares and receives $50,000 from DTC.
- **Swap vs hedge recon:** $50,000 received, $35,000 paid to the client and $15,000 remitted to the IRS → OK.
- **Break if** the pass-through is set up wrong (e.g., 85% instead of 100%) or the 871(m) withholding isn't applied.

---

# PART VIII — PROJECT MANAGEMENT

## 24. Your dual PM + Dev role

| Hat | You are responsible for… | Trap |
|---|---|---|
| **PM** | Scope, priorities, planning, risks, communication, ops buy-in, benefits measurement | Spending all your time coding and no longer steering |
| **Dev** | Reliable, tested, maintainable, secure Dagster pipelines | Coding what's asked without challenging the need |

**Practical rule:** block fixed "PM" slots (e.g., Monday morning for planning, Friday afternoon for reporting) and protect **deep work** blocks for dev.

**Your unique value:** you understand both the business (after this guide) and the technology. You can **translate** ops needs into codable rules, and IT constraints into options the sponsor can understand.

---

## 25. The lifecycle of an ops automation project

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-64** (sheet `P8_Project_Mgmt`)

```
 0. Framing         → Charter, sponsor, objectives, scope, stakeholders
 1. Discovery       → As-is mapping, time measurement, source inventory
 2. Prioritization  → Use case backlog, value × effort, MVP choice
 3. Design          → Business rules, data mapping, architecture, output mockup
 4. Build (sprints) → Iterative dev, demos to ops every 2 weeks
 5. Test / UAT      → Tests with real data, ops sign-off
 6. Parallel run    → Old and new process side by side (2–4 weeks)
 7. Go-live         → IT change management, cutover
 8. Hypercare       → Reinforced support for 2–4 weeks
 9. BAU / Handover  → Runbook, support model, monitoring
10. Benefits        → Measure gains vs baseline, communicate results
```

**Method:** **pragmatic agile** works well. 2-week iterations, a demo to users every time, backlog in Jira (or the internal tool), documentation in Confluence (or equivalent).

**The most important principle:** deliver **a first useful use case in 6–8 weeks** rather than a full platform in 9 months. Credibility comes from the first report that saves ops time.

---

## 26. Discovery: mapping and measuring

### 26.1 As-is mapping

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-65** (sheet `P8_Project_Mgmt`)

For each manual task, fill in a card:

| Field | Example |
|---|---|
| Task name | Recon of today's US dividend payments |
| Team / person | Income team, 2 analysts |
| Frequency | Daily |
| Trigger | DTC payments report received (what time?) |
| Inputs (systems/files) | CA system export, DTC report, nostro cash extract |
| Steps | 1. Extract… 2. Copy into Excel… 3. VLOOKUP… 4. Filter… 5. Investigate… 6. Emails… |
| Output | Excel of breaks, emails to custodians |
| Duration | 2 h/day/person |
| Volume | ~300 lines/day, ~20 breaks |
| Pain points | Unstable formats, duplicates, missing data |
| Risks | Missed break = money not claimed |
| Implicit rules | "We ignore differences < $5", "For ADRs, we check the depositary report" |

### 26.2 Measuring time (*time & motion study*)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-66** (sheet `P8_Project_Mgmt`)

Since you don't know yet how long it takes:

1. Ask ops to **log their time for 2 weeks** (simple table: task, start, end, volume).
2. Or **observe them** (*shadowing*): sit with an analyst for half a day. This is **the best thing you can do** in your first weeks.
3. Compute a **baseline**: hours/day × days/year × number of people → **FTE**.

```
Example: 2 h/day × 250 days × 3 people = 1,500 h/year ≈ 0.8 FTE
```

This baseline is **the foundation of the business case** and of benefits measurement. Without it, you'll never be able to prove the project's value.

### 26.3 Data inventory

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-67** (sheet `P8_Project_Mgmt`)

For each source: owner, format, access mode (API, file, database), frequency, availability time, available history, quality, **IT contact**.

---

## 27. Working with the Sponsor

### 27.1 Who is the sponsor?

Probably a head of CA / PB operations, or of ITO **[TO BE CONFIRMED]**. They **fund**, **arbitrate** and **champion** the project. They talk in terms of **risk, cost, FTE, incidents, clients, audit**, not technology.

### 27.2 Questions to ask the sponsor (first meeting)

1. What is the **#1 problem** you want to solve? What keeps you up at night (losses, missed elections, client complaints, audit)?
2. What does **success look like in 3 months**? And in 12 months?
3. How do you measure success: FTE freed up, fewer breaks, faster detection, losses avoided?
4. Who are the **key users** and **experts** (SMEs) I can lean on? How much of their time?
5. What is the priority **scope** (event types, asset classes, regions)?
6. Are there **ongoing projects** that overlap with this one (new CA system, Broadridge migration, strategic program)?
7. What are the **constraints**: budget, regulatory deadlines, audit, change freezes (*freeze periods*)?
8. How do you want to be kept informed (frequency, format)? Do we need a **steering committee**?
9. Who decides when several teams' needs conflict?
10. What are the **political risks** (teams worried about automation, central IT preferring another solution)?

### 27.3 The **Project Charter** (1 page, to be signed off)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-68** (sheet `P8_Project_Mgmt`)

See the template in §31.1.

### 27.4 Communicating with the sponsor

- Short **weekly status** (template in §31.4): RAG, progress, next steps, **decisions needed**, risks.
- **Monthly steerco**: benefits delivered, roadmap, trade-offs.
- **Golden rule:** **no surprises**. Bad news shared early with an action plan builds trust; discovered late, it destroys it.
- Talk in **impact**: "The payment breaks report is in production: it saves the Income team 1.5 hours a day and caught 3 unpaid dividends worth $45,000 in the first week."

---

## 28. Working with IT

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-69** (sheet `P8_Project_Mgmt`)

### 28.1 Likely IT contacts

| Role | Why you need them |
|---|---|
| Source system owners (CA system, stock record, lending, swaps) | API/DB/file access, data documentation |
| Dagster / data platform team | Environments, deployment, standards |
| Infrastructure / Middleware | Network shares, SFTP, MQ, service accounts |
| Security / IAM | Entitlements, secrets, encryption |
| Production support / Run | Monitoring, on-call, incidents |
| Architecture | Solution sign-off (*design review*) |
| Release / Change management | Production releases (tickets, CAB) |

### 28.2 Questions to ask IT

**Data**
1. What sources are available for: CA events, positions (stock record), payments, lending, swaps, client tax status?
2. Access mode for each: REST API, database (read via a view/replica?), dropped files (where, when, what format)?
3. Is there a **data lake / data warehouse** that already exposes this data?
4. Is there a **data dictionary**? Who are the data owners?
5. What is the data latency (real time, D-1, intraday)?
6. Are there **test environments** with realistic data (anonymized?)?

**Dagster platform**
7. Dagster OSS or **Dagster+**? Which version? Who administers it?
8. DEV / UAT / PROD environments? How is deployment done (CI/CD, Git, code review)?
9. Where do jobs run (Kubernetes, VM, on-prem)? Network access to the sources?
10. How are **secrets** managed (vault, environment variables)?
11. Existing standards: code structure, approved libraries, logging, testing?

**Operations**
12. Who supports production (L1/L2/L3)? Is a **runbook** required?
13. Monitoring and alerting: which tool, which thresholds?
14. **Change** process: lead time, CAB, freeze periods?

**Outputs**
15. Sending email: internal SMTP relay, allowed addresses, attachments allowed?
16. Where to drop Excel files (SharePoint, network share)?

### 28.3 Tips

- **Request access in week 1.** Entitlements in a bank can take **several weeks**: it's often the critical path.
- Plan for **service accounts** (not your personal credentials) in production.
- Document the **interfaces** (data contract: schema, frequency, time, owner).
- Get your architecture validated early (*design review*): it avoids a rejection at go-live.

---

## 29. Working with Ops (SMEs)

- Ops are **experts** and **busy**. Respect their time: short, prepared sessions at quiet hours (not around the day's deadlines).
- Have them **co-design** the output: show an Excel mockup in week one and iterate.
- **Fear of automation:** position the tool as a way to **remove low-value tasks** so they can focus on investigation and clients.
- Ask them for **real examples of breaks** (with their resolution). They will become your **test cases**.
- Identify a **champion** in each team: they'll test, give feedback and advocate the tool to colleagues.
- **Parallel run** is mandatory: the tool must find **at least** the same breaks as the manual process before replacing anything.

---

## 30. Governance in a bank: controls, EUC, data, audit

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-70** (sheet `P8_Project_Mgmt`)

| Topic | What it means for you |
|---|---|
| **EUC** (*End-User Computing*) | Critical Excel files and macros are an identified risk (EUC inventory). **Replacing an EUC with a controlled pipeline** is a strong argument for your business case. You will also need to **decommission** the old EUC. |
| **SDLC controls** | Versioned code (Git), code review, tests, DEV/PROD separation, deployment traceability |
| **Data confidentiality** | Hedge fund position data is **highly confidential**. Reports distributed on a *need-to-know* basis. Watch out for external emails. |
| **Access** | Least privilege, periodic access reviews |
| **Audit trail** | Keep inputs, outputs and rule versions for each run (Dagster helps: metadata, run history) |
| **Controls** | If your report becomes a **key control**, it will be tested by audit/op risk: documentation, evidence of execution, exception handling |
| **Retention** | Retention period for reports and data (internal policy) |
| **Models / business rules** | Document every rule (tolerances, mapping) with its **business owner** and sign-off date |

---

## 31. Ready-to-use templates

### 31.1 Project Charter (1 page)

```markdown
# Project Charter — CA Breaks Automation (Prime Brokerage US)

**Sponsor:** [Name, title]
**PM / Lead Dev:** [You]
**Date:** [ ]  **Version:** 0.1

## Problem
CA PB US teams detect breaks (events, positions, payments, tax, elections)
manually through extracts and Excel files.
This process takes [X h/day], is error-prone and detects breaks late.

## Objectives (SMART)
1. Automate payment (cash) break detection on US dividends by [date].
2. Reduce manual recon time by [X]% by [date].
3. Alert on 100% of voluntary events with a missing election at D-3.

## Scope
- IN: US, DVCA/INTR/… events, PB accounts, Excel + email outputs
- OUT (phase 1): non-US, proxy, tax reporting

## Deliverables
- Dagster pipelines in production, daily reports, runbook, rule documentation

## Stakeholders
Sponsor, CA Ops (Income, Reorg, Claims), IT (source owners, Dagster platform), Tax, Risk/Control

## Milestones
M1 Discovery complete | M2 MVP in UAT | M3 UC1 go-live | M4 UC2…

## Expected benefits
FTE freed up, breaks detected earlier (D0 vs D+3), fewer losses, EUCs removed

## Main risks
Data access, data quality, SME availability, IT priorities

## Assumptions / Dependencies
[ ]
```

### 31.2 RACI (example)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-71** (sheet `P8_Project_Mgmt`)

| Activity | Sponsor | PM/Dev (you) | Ops SMEs | IT Sources | IT Platform | Risk/Control |
|---|---|---|---|---|---|---|
| Use case prioritization | **A** | R | C | I | I | I |
| Business rules | I | R | **A** | I | — | C |
| Data access | I | R | I | **A** | C | — |
| Development | I | **A/R** | C | C | C | — |
| UAT | I | R | **A** | — | — | I |
| Production release | I | R | I | C | **A** | I |
| Run / Support | I | C | I | — | **A** | — |
| Control validation | I | C | C | — | — | **A** |

*R = Responsible, A = Accountable, C = Consulted, I = Informed*

### 31.3 RAID Log

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-72** (sheet `P8_Project_Mgmt`)

| ID | Type | Description | Impact | Likelihood | Owner | Action | Status |
|---|---|---|---|---|---|---|---|
| R1 | **Risk** | Access to the positions database delayed | High | Medium | You | Request access in week 1, escalate to sponsor if > 3 weeks | Open |
| A1 | **Assumption** | The Bloomberg Data License CA feed is already available | — | — | IT | To confirm | Open |
| I1 | **Issue** | DTC file format differs between UAT and PROD | Medium | — | IT | — | Open |
| D1 | **Dependency** | CA system migration project | High | — | Sponsor | Sync the roadmaps | Open |

### 31.4 Weekly Status Report

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-73** (sheet `P8_Project_Mgmt`)

```markdown
# Status — CA Breaks Automation — Week of [ ]

**Overall status: 🟢 / 🟡 / 🔴**   (Scope 🟢 | Schedule 🟡 | Resources 🟢)

## Highlights
- UC1 (DVCA cash breaks): in UAT with the Income team, 2 minor bugs fixed.
- Positions API access granted.

## Next steps
- UC1 parallel run from [ ] to [ ].
- UC2 business rules workshop (elections) with Voluntary Reorg.

## Decisions needed
- Approve a $1 per event rounding tolerance (Income Ops).

## Risks / Watch points
- Reduced SME availability in December (year-end, high volume).

## Benefits to date
- [X] breaks detected automatically, [Y] hours saved.
```

### 31.5 User story (format)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-74** (sheet `P8_Project_Mgmt`)

```
As an Income CA analyst,
I want to receive an Excel file every day at 10 a.m. listing US dividends
where the payment received differs from the expected amount by more than $1,
so that I can start investigating before clients reach out to us.

Acceptance criteria:
- All events with pay date = D-1 or D are covered.
- Each line contains: CUSIP, name, type, pay date, account, expected, received, difference, category, aging.
- Differences < $1 are excluded but counted in a summary tab.
- If the source data hasn't arrived by 9:45 a.m., an alert email is sent.
```

### 31.6 Business rule card (to be signed off by ops)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-75** (sheet `P8_Project_Mgmt`)

```
Rule: CASH_TOLERANCE_USD
Description: cash difference ignored if |expected - received| ≤ 1.00 USD per event × account
Business owner: [Name, team]
Approved on: [date]
Rationale: DTC rounding on large positions
Review: annual
```

---

## 32. Prioritized use case backlog

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-76** (sheet `P8_Project_Mgmt`)

| # | Use case | Break family | Value | Complexity | Proposal |
|---|---|---|---|---|---|
| UC1 | **Cash payment recon** (expected vs received) on US dividends and coupons | B5, B14 | High | Low/Medium | **MVP** |
| UC2 | **Election & deadline monitor** (voluntary events, client responses, available positions, protects) | B12, B13 | **Very high** (risk) | Medium | **MVP or Phase 2** |
| UC3 | **Event data recon** (Bloomberg vs DTC vs internal system) | B1, B2 | High (prevention) | Medium | Phase 2 |
| UC4 | **Position recon on record date** (stock record vs DTC) | B3 | High | Medium/High | Phase 2 |
| UC5 | **Tax rate check** (rate applied vs tax status) | B7 | Medium/High | Medium | Phase 3 |
| UC6 | **Short dividend & lending manufactured payments recon** | B9, B10 | High in PB | High | Phase 3 |
| UC7 | **Swap vs hedge dividend recon** | B11 | High in synthetic PB | High | Phase 3 |
| UC8 | **Claims aging dashboard** | B15 | Medium | Low | Quick win |
| UC9 | **Reference data / CUSIP change check** | B16 | Medium | Low | Quick win |
| UC10 | **MBS factor recon** | B5 (FI) | Medium | Medium | Phase 3 |

**Prioritization method:** score = (hours saved + risk avoided + visibility) / effort. Have the sponsor validate the matrix.

---

## 33. Classic pitfalls

1. **Automating a poorly understood process.** Shadow before you code.
2. **Starting with a generic platform.** Deliver a concrete use case first.
3. **Underestimating data access.** It's often 50% of the timeline.
4. **Ignoring ops' implicit rules** ("we always ignore that custodian on Mondays"). Get them **stated and documented**.
5. **Too many false positives.** A 500-line report where 480 lines are irrelevant will be abandoned within 2 weeks. Work on tolerances and auto-classification.
6. **No baseline.** You won't be able to prove the gains.
7. **No parallel run.** Risk of missing breaks at cutover.
8. **No run plan.** Who looks when the job fails at 7 a.m.?
9. **No history.** You must be able to say: "the break existed on 11/12 and was resolved on 11/15."
10. **Forgetting US holidays** and month-ends (volumes, payment calendar).

---

# PART IX — DAGSTER IMPLEMENTATION

## 34. Target architecture

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-77** (sheet `P9_Dagster`)

```
        SOURCES                         DAGSTER                                  OUTPUTS
┌──────────────────────┐   ┌──────────────────────────────────────────┐   ┌──────────────────┐
│ Bloomberg DL (SFTP)  │──▶│ RAW (ingestion, 1 asset per source)       │   │ Excel breaks     │
│ DTC (files/API)      │──▶│   - file sensors / API schedules          │   │ Summary email    │
│ Broadridge (extract) │──▶│                                           │   │ History table    │
│ Internal CA system   │──▶│ STAGING (normalization → canonical model) │──▶│ (DB / parquet)   │
│ Positions / stock rec│──▶│   - quality asset checks                  │   │ Dashboard        │
│ Cash / nostro        │──▶│                                           │   │ (later)          │
│ Lending / Swaps      │──▶│ CORE (golden events, expected entitlements│   └──────────────────┘
│ Client tax status    │──▶│        received payments)                 │
└──────────────────────┘   │ RECON (matching, breaks, categorization)  │
                           │ REPORT (excel, email, history)            │
                           └──────────────────────────────────────────┘
```

**Principles:**

- **1 asset = 1 dataset**; layered as `raw → staging → core → recon → report`.
- **Daily partitions** (business date): rerun a specific day, run a backfill.
- **Idempotence**: rerunning a run gives the same result.
- **Business rules in configuration** (YAML/table), not hard-coded.
- **Break history** (status, first detection, resolution) for aging.
- **Asset checks** to block a report built on incomplete data.

---

## 35. Canonical data model

### 35.1 `ca_event` (simplified golden copy)

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-78** (sheet `P9_Dagster`)

| Field | Type | Example |
|---|---|---|
| event_key | str | `US0378331005|DVCA|2026-11-13` |
| source | str | `BBG` / `DTC` / `INTERNAL` |
| source_event_id | str | Bloomberg / DTC / internal ID |
| coaf | str | Official reference if available |
| cusip / isin | str | |
| caev | str | `DVCA` |
| camv | str | `MAND` |
| ex_date / record_date / pay_date | date | |
| currency | str | `USD` |
| gross_rate | decimal | 0.25 |
| ratio_new / ratio_old | decimal | 4 / 1 |
| status | str | `PREU` / `COMP` / `CANC` |
| version_ts | timestamp | version timestamp |

### 35.2 `position_snapshot`

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-79** (sheet `P9_Dagster`)

| Field | Example |
|---|---|
| business_date, cusip, account_id, account_type (`LONG`/`SHORT`/`LOAN`/`BORROW`/`SWAP_HEDGE`/`FIRM`), quantity, location (`DTC`, `FED`, custodian), tax_status, treaty_rate |

### 35.3 `expected_entitlement`

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-80** (sheet `P9_Dagster`)

| Field | Example |
|---|---|
| event_key, account_id, quantity, gross_amount, tax_rate, tax_amount, net_amount, currency, payment_type (`DIVIDEND`/`SUBSTITUTE`/`SHORT_CHARGE`/`MANUFACTURED`) |

### 35.4 `received_payment`

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-81** (sheet `P9_Dagster`)

| Field | Example |
|---|---|
| source (`DTC`/`MT566`/`NOSTRO`), source_ref, event_key (or cusip + date), account_id, gross, tax, net, currency, value_date |

### 35.5 `ca_break`

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-82** (sheet `P9_Dagster`)

| Field | Example |
|---|---|
| break_id (stable hash), break_type (`B5_CASH`…), event_key, account_id, expected, actual, diff, diff_usd, category (`UNPAID`/`TAX_RATE`/`POSITION`/`ROUNDING`/`UNKNOWN`), first_seen_date, last_seen_date, aging_days, status (`OPEN`/`RESOLVED`), comment |

> The **stable break_id** (hash of `break_type + event_key + account_id`) lets you track a break from one day to the next and compute aging.

---

## 36. Dagster code skeleton

> Illustrative code, to be adapted to the sources and internal standards. System names are fictitious.

### 36.1 Project structure

```
ca_breaks/
├── pyproject.toml
├── ca_breaks/
│   ├── definitions.py
│   ├── resources.py          # API / DB / SMTP / file system connections
│   ├── partitions.py
│   ├── config/
│   │   └── rules.yaml        # tolerances, mappings, recipients
│   ├── assets/
│   │   ├── raw.py            # raw ingestion
│   │   ├── staging.py        # normalization
│   │   ├── core.py           # golden events, expected, received
│   │   ├── recon.py          # matching + breaks
│   │   └── report.py         # excel + email + history
│   ├── checks.py             # asset checks
│   ├── sensors.py            # file sensors
│   └── lib/
│       ├── matching.py       # matching engine (pure Python, testable)
│       ├── categorize.py
│       └── calendar.py       # US business days
└── tests/
    ├── fixtures/             # anonymized files of real cases
    └── test_matching.py
```

### 36.2 Partitions and resources

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
    token: str  # injected from the secrets manager, never hard-coded

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
    root: str  # e.g.: /mnt/shares/ca/inbound

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
        msg.set_content("See the HTML version.")
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

### 36.3 Ingestion and normalization

```python
# assets/raw.py + staging.py (condensed)
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
    """Map source columns to the canonical model."""
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
# assets/core.py (condensed)
import dagster as dg
import pandas as pd
from ..partitions import business_days


@dg.asset(partitions_def=business_days, group_name="core")
def expected_entitlements(
    golden_events: pd.DataFrame,          # validated events (pay_date = partition)
    stg_positions_at_record: pd.DataFrame # positions at each event's record date
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

### 36.5 Recon and asset checks

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
        description="The DTC payments file must not be empty on a business day",
    )


@dg.asset_check(asset="stg_received_payments")
def no_null_cusip(stg_received_payments: pd.DataFrame) -> dg.AssetCheckResult:
    n = int(stg_received_payments["cusip"].isna().sum())
    return dg.AssetCheckResult(passed=n == 0, metadata={"null_cusips": n})
```

### 36.6 Sensor on a dropped file + schedule

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-83** (sheet `P9_Dagster`)

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
    done_flag = path + ".done"  # convention: the producer drops a .done file
    if os.path.exists(done_flag):
        yield dg.RunRequest(run_key=f"dtc-{today}", partition_key=today)
    else:
        yield dg.SkipReason(f"Waiting for {done_flag}")
```

> **File best practices:** wait for an **end-of-write flag** (`.done`, or stable file size) before reading; archive processed files; never delete an input.

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

## 37. The matching engine

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-84** (sheet `P9_Dagster`)

Keep it in **pure Python/pandas** (no Dagster) so it's easy to test.

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
        return "TAX_DIFFERENCE"          # same gross, different net → tax
    if row["gross_expected"]:
        ratio = row["gross"] / row["gross_expected"]
        if 0 < ratio < 1:
            return "PARTIAL_OR_POSITION"
        if ratio > 1:
            return "OVERPAID_OR_POSITION"
    return "UNKNOWN"


def reconcile_cash(expected: pd.DataFrame, received: pd.DataFrame,
                   tolerance: float, business_date: str) -> pd.DataFrame:
    rec = received.groupby(KEYS, as_index=False)[["gross", "tax", "net"]].sum()  # handles partial payments
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

### 37.1 History and aging

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-85** (sheet `P9_Dagster`)

Every day, merge today's breaks with the history table:

- Break present yesterday **and** today → `last_seen_date = today`, aging increases.
- Break present yesterday but **not** today → `status = RESOLVED`, `resolved_date = today`.
- New break → `first_seen_date = today`.

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

> Ideally use a **database** (or versioned parquet files) for the history, not an Excel file.

---

## 38. Outputs: Excel and email

### 38.1 Recommended Excel structure

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-86** (sheet `P9_Dagster`)

| Tab | Content |
|---|---|
| **Summary** | Number and amount of breaks by category and by aging, comparison with the previous day |
| **New breaks** | Today's new breaks (investigation priority) |
| **Open breaks** | All open breaks, sorted by USD amount descending |
| **Resolved** | Breaks resolved since yesterday (evidence of follow-up) |
| **Data quality** | Asset check results, missing files, rejected lines |
| **Parameters** | Tolerances, sources, data timestamps (audit traceability) |

### 38.2 Generation (openpyxl via pandas)

```python
# assets/report.py (condensed)
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
        for ws in xw.book.worksheets:                 # minimal formatting
            ws.freeze_panes = "A2"
            ws.auto_filter.ref = ws.dimensions

    html = f"""
    <p>CA Cash Breaks — {d}</p>
    <ul>
      <li>New breaks: <b>{len(new_b)}</b></li>
      <li>Open breaks: <b>{len(open_b)}</b> ({open_b['diff'].abs().sum():,.2f} USD)</li>
      <li>Open &gt; 30 days: <b>{(open_b.aging_days > 30).sum()}</b></li>
    </ul>"""
    mailer.send(to=["ca-income-ny@…"], subject=f"[CA] Cash breaks {d}", html=html, attachments=[path])
    return path
```

> **Confidentiality:** internal recipients only, list managed in configuration and approved by the sponsor. If internal policy requires it, send a **link** to the share rather than an attachment **[TO BE CONFIRMED]**.

---

## 39. Testing, quality, deployment, run

### 39.1 Tests

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-87** (sheet `P9_Dagster`)

- **Unit tests** on `lib/` (matching, categorization, entitlement calculation) using the **worked cases from Part VII** as fixtures.
- **Golden tests**: a real (anonymized) day → the expected output is validated by ops and frozen.
- **Regression tests** on every rule change.
- Dagster tests: `dg.materialize([...], resources={...})` with mocked resources.

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

### 39.2 Data quality (recommended asset checks)

- File present and non-empty on a business day
- Row count within the usual range (± X% vs the average of the last 20 days)
- No null CUSIP, no non-numeric amount
- Every position has a tax status
- **Freshness**: data for the right day (not yesterday's file)

### 39.3 Deployment

- Git + code review + CI (lint `ruff`, tests `pytest`) + per-environment deployment.
- Per-environment configuration (DEV/UAT/PROD) via environment variables and secrets.
- **Version the rules** (`rules.yaml`) and record their version in run metadata.

### 39.4 Run

- **Alerting** on a failed run or a blocking asset check (email/Teams to support and to you).
- **Runbook:** what to do if the DTC file doesn't arrive, how to rerun a partition, who to contact.
- **SLA:** report available before [time]; on failure, a "report delayed" email to ops.
- **Backfills:** partitions let you regenerate history or replay after a fix.

---

# APPENDICES

## Appendix A — Glossary

| Term | Meaning |
|---|---|
| **Accrual** | Booking of income receivable/payable before the payment |
| **ADR** | American Depositary Receipt: US certificate representing a foreign share |
| **Agent (paying/transfer/exchange)** | Issuer-side intermediary |
| **Aging** | Age of a break or claim |
| **ATOP** | Automated Tender Offer Program: DTC system for tenders and exchange offers |
| **Backup withholding** | US withholding (24%) when tax documentation is missing |
| **Break** | Reconciliation difference |
| **Buy-in** | Forced purchase of undelivered securities |
| **Cash in lieu (CIL)** | Cash paid instead of fractional shares |
| **Cede & Co.** | DTC's nominee |
| **Claim** | Claim against a counterparty |
| **COAF** | Official Corporate Action Event Reference |
| **Cum / Ex** | With / without the entitlement |
| **CUSIP** | US security identifier |
| **Default option** | Option applied when no instruction is received |
| **Due bill** | Obligation to pass a distribution on to the buyer (US window) |
| **DTC / DTCC** | US central securities depository / its holding company |
| **Election / Instruction** | The holder's choice on an event with options |
| **Entitlement** | What a holder is entitled to |
| **EUC** | End-User Computing (Excel, macros…) |
| **Ex-date** | First trading day without the entitlement |
| **Factor** | Remaining proportion of principal (MBS/ABS) |
| **Fail** | Trade not settled on its intended date |
| **Golden copy** | Validated version of an event |
| **Guaranteed delivery / Protect** | Tender instruction with deferred delivery of securities |
| **Manufactured payment** | Compensating payment in a loan/repo |
| **Market claim** | Claim for a distribution after a trade spanning the record date |
| **Merger arbitrage** | Hedge fund strategy betting on mergers |
| **NRA** | Non-Resident Alien (US withholding regime) |
| **OCC** | Options Clearing Corporation |
| **Over-election** | Election greater than the available position |
| **Pay date** | Payment date |
| **Prime Broker** | Bank providing services to hedge funds |
| **Proration** | Pro rata reduction when an offer is oversubscribed |
| **Proxy** | Shareholder meeting vote |
| **QI / QDD** | Intermediary / derivatives dealer tax statuses |
| **Recall** | Recalling lent securities |
| **Record date** | Date on which holders are captured |
| **Rehypothecation** | Broker's reuse of client securities |
| **Reorg** | Reorganization (mandatory/voluntary) |
| **Scrubbing** | Cleaning/validating event data |
| **Stock record** | The broker-dealer's securities ledger |
| **STP** | Straight-Through Processing |
| **Substitute payment** | Payment "in lieu" of a dividend (lent securities) |
| **Tender offer** | Tender offer |
| **TRS / Portfolio swap** | Total return swap (synthetic prime) |
| **Treaty rate** | Reduced withholding rate under a tax treaty |
| **Withholding tax (WHT)** | Tax withheld at source |

## Appendix B — SWIFT/ISO fields to know

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-88** (sheet `Annexes`)

| Field (MT564/566) | Meaning |
|---|---|
| `20C::CORP` | Event reference (provider) |
| `20C::COAF` | Official reference |
| `23G` | Message function (NEWM, REPL, REPE, CANC, WITH) |
| `22F::CAEV` | Event type |
| `22F::CAMV` | MAND / CHOS / VOLU |
| `25D::PROC` | Processing status (PREU, COMP…) |
| `35B` | Security identifier (ISIN + description) |
| `93a::ELIG` / `SETT` | Eligible / settled balance |
| `98a::XDTE / RDTE / PAYD / EFFD` | Ex / Record / Pay / Effective date |
| `98a::MKDT / RDDT` | Market / Response deadline |
| `92a::GRSS / NETT / WITF / TAXR` | Gross / net / withholding rate |
| `92D::NEWO / ADEX` | New/old ratio / additional ratio |
| `13A::CAON` | Option number |
| `22F::CAOP` | Option type |
| `17B::DFLT` | Default option (Y/N) |
| `19B::ENTL / GRSS / NETT / TAXR` | Amounts (entitlement, gross, net, tax) |
| `90a::PRPP / OFFR` | Price (offer, subscription) |

## Appendix C — Checklist of questions to ask internally

> 📊 **Excel example(s)** — `CA_PB_US_Exemples.xlsx`: **EX-89** (sheet `Annexes`)

**Organization**
- [ ] Org chart of the PB CA teams (NYC, offshore): who does what?
- [ ] Who exactly is the sponsor, and who are the SMEs by event type?

**Systems**
- [ ] What is the main CA system? (Broadridge? Which product? In-house?)
- [ ] Where do announcements come from: Bloomberg Data License, DTC ISO 20022, others?
- [ ] Which system holds the stock record? Lending? Swaps? Tax statuses?
- [ ] What reconciliation tools already exist (SmartStream TLM, Duco, in-house…)?

**Processes**
- [ ] Which reconciliations are done today, at what time, with which files?
- [ ] What are the 10 most frequent break types? The most costly?
- [ ] What CA incidents or losses have occurred in the last 2 years?
- [ ] How are PB client elections handled (channel, internal deadline, default option)?
- [ ] How are substitute payments allocated?
- [ ] How do fund admin queries come in, and how are they tracked?

**Data & IT**
- [ ] Data access (API, DB, files), test environments, service accounts
- [ ] Dagster platform: version, deployment, standards, support
- [ ] Email/sharing policy for client data

**Controls**
- [ ] Are the current Excel files in the EUC inventory?
- [ ] Will the future report be a key control?

## Appendix D — 30-60-90 day plan

| Period | PM goals | Dev goals | Learning goals |
|---|---|---|---|
| **Days 1–30** | Meet the sponsor and teams, shadowing (≥ 3 half-days), measure the baseline, write the charter, build the backlog, request **all access** | Environment setup, first "hello data" pipeline on a real source, UC1 matching prototype on exported data | Master Parts I–VI; vocabulary; read 20 real MT564/566 |
| **Days 31–60** | Validate prioritization, steerco #1, UC1 business rules signed off | UC1 in UAT, Excel co-designed with ops, asset checks, history | Go deeper on PB (rehypo, shorts, swaps), tax |
| **Days 61–90** | UC1 parallel run, go-live, first benefits measured, UC2 kickoff | UC1 in prod (runbook, alerting), UC2 design (elections/deadlines) | Complex cases: voluntary reorgs, protects, MBS |

## Appendix E — Further resources

- **DTCC**: public documentation on Asset Services / Corporate Actions (CA Web, ISO 20022, ATOP, Claim Connect) — dtcc.com
- **SMPG**: CA market practices (smpg.info)
- **ISITC**: US market practices (isitc.org)
- **SWIFT**: documentation for MT5xx categories and ISO 20022 `seev` messages
- **OCC**: Information Memos on option adjustments (theocc.com)
- **SEC**: Rule 15c3-3 (Customer Protection), Rule 10b-17
- **FINRA**: Uniform Practice Code (ex-dates, due bills)
- **IRS**: publications on NRA withholding (Pub. 515), Section 871(m), Forms W-8/W-9, 1042-S, 1099
- **MSRB EMMA**: muni data
- **Dagster**: official documentation (assets, partitions, sensors, asset checks, resources) — docs.dagster.io

---

*Living document: fill in the sections flagged **[TO BE CONFIRMED INTERNALLY]** as you meet people, and turn it into project documentation.*

---

## Appendix F — US terminology bridge (NYC desk jargon)

> Addition: the vocabulary you'll **actually** hear on the desk and in emails in New York, with its definition and its equivalent or a note. Organized by theme to bridge with the rest of the guide.

### F.1 Elections and voluntary events

| US term | Definition | Equivalent / note |
|---|---|---|
| **Voluntary Reorg** (*Vol Reorg*) | Team and family of events where the holder chooses (tenders, exchange offers, rights, consents, puts) | `VOLU` / `CHOS` events |
| **Mandatory Reorg** (*Mand Reorg*) | Reorganizations with no choice: splits, cash mergers, name/CUSIP changes, calls | `MAND` events other than distributions |
| **Election** / **Instruction** | The choice communicated by the client (option + quantity) | Market side: MT565 / ATOP |
| **Non-election** / **No response** | The client didn't respond before the *cutoff* | Triggers the default option (CHOS) or *no action* (VOLU) |
| **Non-electing holders** | Holders who didn't elect. In a merger with an election, they receive whatever is left after proration (often a cash/stock mix) | Explain to clients: the result can be surprising |
| **Default option** / **Default election** | Option applied automatically on non-election | Field `17B::DFLT//Y` |
| **"We'll default them"** | "We'll apply the default option to them" | Desk phrase |
| **Take no action** / **NOAC** | Explicit choice to do nothing | Better than silence: you know the client saw the event |
| **Cutoff** / **Client cutoff** / **Internal deadline** | Deadline set by the PB for its clients, ahead of DTC's | The guide's *client deadline* |
| **DTC cutoff** / **Market deadline** / **Agent deadline** | DTC's or the agent's deadline | `RDDT` / `MKDT` |
| **Expiration** / **Expiration date** | End of the offer, often 5:00 p.m. or 11:59 p.m. NY time (*"5 p.m. New York City time"*, *"one minute after 11:59 p.m."*) | Always check the exact time in the *Offer to Purchase* |
| **Extended** / **Extension** | The offer is extended. The new deadline applies and elections already received generally remain valid | A classic source of confusion: chase the non-responders again |
| **Withdrawal rights** / **Withdraw** | Right to **withdraw** securities already tendered as long as the offer hasn't expired | The client can change its mind: the ATOP instruction must be cancelled |
| **Tender** / **Tendered shares** / **Contra CUSIP** | Tendering into the offer. Tendered shares are blocked at DTC, often under a temporary **"contra" CUSIP** | They can no longer be sold, lent or used as collateral |
| **ATOP** | *Automated Tender Offer Program*: DTC system to instruct tenders and exchange offers | — |
| **Offer to Purchase** / **Letter of Transmittal** | Official offer documents (terms, price, deadlines) | The "primary source" to check when in doubt |
| **Protect** / **NGD** (*Notice of Guaranteed Delivery*) | Tender instruction **without** delivering the securities right away, with a commitment to deliver them before the **protect deadline** (often 1 to 2 business days after expiration) | Used when securities are on loan, failing or in transit |
| **Cover the protect** / **Protect cover** | Actually delivering the securities promised by the protect | — |
| **"We're short the protect"** / **Failed protect** | We don't have the securities to cover the protect | Risk of client compensation |
| **Over-election** / **Over-instructed** | Election greater than the available position | Break B12 |
| **Short tender** / **Rule 14e-4** | Tendering more than your net long position (prohibited) | Mandatory check (EX-25) |
| **Proration** / **Prorated** / **Proration factor** | Partial acceptance when the offer is oversubscribed (e.g., 72% accepted) | The rest is returned to the holder |
| **Odd lot** / **Odd-lot priority** | Holder of fewer than 100 shares, sometimes exempt from proration | Rare but exists in buybacks |
| **Dutch auction** | Buyback where holders bid a price within a range and the issuer sets a single clearing price | `BIDS` event |
| **Early tender premium** / **Early tender date** | Premium paid to bondholders who tender before an early date | Common on *debt tenders* |
| **Consent solicitation** / **Consent fee** | Request for a bondholder vote to amend the contract (*indenture*), sometimes with a fee | `CONS` event |
| **Rights** / **Subscription** / **Oversubscription privilege** | Subscription rights; ability to request additional shares | `RHDI` / `EXRI` / `OVER` |
| **Lapse** / **Lapsed rights** | Rights expired unexercised: their value is lost | `LAPS` |
| **Buyer protection** | Mechanism by which the **buyer** in a not-yet-settled trade passes its election to the **seller**, who must deliver the chosen outcome | Mostly used on elections with open trades |
| **Recall** | Recalling lent securities (to elect, vote or sell) | Equity Finance |
| **Liability** / **Voluntary liability** | The borrower's obligation to deliver to the lender the economic outcome of its election | Securities lending |

### F.2 Income (dividends, coupons, distributions)

| US term | Definition | Equivalent / note |
|---|---|---|
| **Income** / **Divs** / **Distributions team** | Team handling cash and stock payments with no choice | — |
| **Declared** / **Declaration date** | Board decision date | ≈ Announcement date |
| **Ex date** / **"Goes ex"** | "The stock goes ex-dividend" | `XDTE` |
| **Record date** / **"Holders of record"** | Holders registered on the record date | `RDTE` |
| **Payable date** / **Pay date** | Payment date | `PAYD` |
| **Regular dividend** / **Special dividend** / **Extra dividend** | Recurring / one-off / one-off top-up dividend | *Specials* can trigger OCC adjustments |
| **Due bill** / **Due bill period** / **Interim accounting** | See §4.4: distribution owed to the buyer when the ex-date falls after the pay date | US specific |
| **Accrual** / **Receivable** / **Payable** | Income receivable / payable booked on the ex-date | §18 |
| **Contractual income** / **Contractual settlement** | The PB credits the client on the pay date **even if** the market hasn't paid yet | Risk: cash advance |
| **Actual income** / **Actual settlement** | The client is credited **on receipt** of the market payment | — |
| **Unpaid** / **Outstanding** / **Aged** | Expected payment not received / break that is getting old | UNPAID category |
| **Short dividend** / **Short charge** | Dividend debited on a short client's account | §10.2 |
| **Manufactured dividend** / **Substitute payment** / **Payment in lieu (PIL)** | Compensating payment made instead of a real dividend (lent securities) | Different tax treatment |
| **Cash in lieu (CIL)** | Cash instead of fractional shares | — |
| **Paydown** / **Factor** / **P&I** | Monthly principal repayment (MBS/ABS) / pool factor / *Principal & Interest* | §12.4 |
| **Day count** / **30/360** / **ACT/360** | Interest calculation convention | EX-41 |
| **Reset** / **Refix** | Rate reset on an FRN or a swap | Classic cause of rate breaks |

### F.3 Reconciliation and break management

| US term | Definition | Equivalent / note |
|---|---|---|
| **Break** | Reconciliation difference | — |
| **Rec** / **Recon** | Reconciliation | *"Run the rec"* = launch the recon |
| **Expected vs Actual** / **Expected vs Received** | Expected amount vs actual amount | Core of UC1 |
| **Matched** / **Unmatched** / **Partially matched** | Reconciled / not reconciled / partly reconciled | — |
| **Aging** / **"Aged break"** / **Aging buckets** | Age of a difference and its buckets (0-30, 31-60, 61-90, 90+) | EX-53 |
| **Root cause** / **RCA** | Root cause / *Root Cause Analysis* | Automatic categorization |
| **Tolerance** / **Threshold** | Level below which a difference is ignored | To be validated (EX-75) |
| **Write-off** | Booking a difference as a loss | Last resort |
| **Claim** / **Market claim** / **Reverse claim** | Claim against a counterparty / the reverse | §21 |
| **Fail** / **Fail to deliver (FTD)** / **Fail to receive (FTR)** | Trade not settled on the delivery side / receive side | Cause of claims |
| **Stock record** / **Box** / **Seg** | Securities ledger / securities in possession / segregated securities | §9.4 |
| **Short securities difference** / **Long securities difference** | Securities shortfall / surplus in the stock record | Possible capital cost (§10J) |
| **Nostro** / **DDA** (*Demand Deposit Account*) | The bank's cash account | Cash side of the recon |
| **STP** (*Straight-Through Processing*) | Processing with no manual intervention | KPI |
| **Exception** / **Exception queue** | Item to be handled manually | What your tool should reduce |
| **Escalation** | Raising an issue to management or the client | — |

### F.4 Prime Brokerage and financing

| US term | Definition | Equivalent / note |
|---|---|---|
| **PB** / **Prime** | Prime brokerage | — |
| **Cash prime** vs **Synthetic prime** | Physical securities vs exposure through swaps | §10G |
| **Delta One** | Desk or products replicating an underlying's performance 1:1 (swaps, TRS) | Synthetic |
| **Debit balance** / **Credit balance** | Cash borrowed by the client / client cash held at the PB | Basis of financing |
| **Rebate** | Interest paid back to the client on its short-sale cash | — |
| **Locate** | Prior confirmation that a security can be borrowed before a short sale | Reg SHO |
| **GC** / **HTB** / **Special** | Easy / hard / very hard to borrow security | CAs often make securities HTB |
| **Borrow** / **Loan** / **Street side** | Securities borrowed / lent with external counterparties | Equity Finance |
| **Internalization** | Covering one client's short with another client's longs | Rehypo |
| **Rehypo** (*Rehypothecation*) | Reuse of client margin securities | §9.2 |
| **Fully paid** / **Excess margin** | Paid-for securities / securities beyond the margin requirement, to be protected | Rule 15c3-3 |
| **Buy-in** | Forced purchase of undelivered securities | — |
| **Margin call** / **House call** / **Maintenance call** | Margin call (internal rules / maintenance) | §10E |
| **Haircut** | Discount applied to a collateral's value | 100% on an unpriced security |
| **Give-up** / **Executing broker (EB)** | Trade executed at another broker, then transferred to the PB | §10D |
| **Fund admin** / **Administrator** | Provider that keeps the fund's books and computes its NAV | Source of queries |
| **NAV strike** | Official NAV calculation (often monthly) | Breaks must be explained before it |
| **Multi-prime** | Fund using several PBs | — |

### F.5 Tax and documentation

| US term | Definition | Equivalent / note |
|---|---|---|
| **NRA withholding** | Withholding on US income paid to non-residents (30% by default) | Chapter 3 |
| **Treaty rate** / **Treaty benefits** | Reduced rate under a tax treaty | Requires valid documentation |
| **W-9** / **W-8BEN** / **W-8BEN-E** / **W-8IMY** | Tax status forms (US / non-US individual / non-US entity / intermediary) | Expirations = tax breaks |
| **Backup withholding** | 24% withholding if a US person has no valid W-9 | — |
| **1099-DIV / -INT / -B / -MISC** | Annual tax reporting for US persons | Substitute payments on 1099-MISC |
| **1042-S** | Reporting of income paid to non-US persons | — |
| **QI** / **QDD** | Qualified Intermediary / Qualified Derivatives Dealer | IRS statuses |
| **871(m)** / **Dividend equivalent** | Withholding on dividend equivalents from derivatives (non-US) | Synthetic |
| **Reclass** / **Year-end reclassification** | Year-end change of tax character (ROC, capital gains) | ETFs, REITs, funds |
| **ROC** (*Return of Capital*) | Return of capital, not taxed like a dividend | — |
| **Form 8937** | Issuer form describing a CA's impact on cost basis | Spin-offs, reorgs |
| **Cost basis** | Tax cost basis | EX-36 |

### F.6 Participants, systems and acronyms

| US term | Definition | Equivalent / note |
|---|---|---|
| **DTC** / **DTCC** / **"the Street"** | US central depository / its holding company / all the other banks and brokers | — |
| **Cede** / **Cede & Co.** | DTC's nominee on issuers' registers | — |
| **Participant** / **Participant number** | Bank or broker that is a DTC member, identified by a number | The PB is a participant |
| **CA Web** | DTC's corporate actions portal | — |
| **Transfer agent** / **TA** | Keeps the shareholder register | Computershare, Equiniti… |
| **Paying agent** / **Trustee** | Makes payments / represents bondholders | — |
| **Info agent** / **Tender agent** / **Exchange agent** | Agents for an offer | — |
| **OCC** / **Info Memo** | Options clearing house / adjustment memo | §13.1 |
| **CUSIP** / **"New CUSIP"** / **"CUSIP change"** | US identifier / identifier change after a reorg | EX-08 |
| **Golden copy** / **Scrub** / **Scrubbing** | Validated version of an event / data cleansing | §6.2 |
| **Ops** / **Middle office** / **Back office** | Operations; post-trade control; processing and settlement | — |
| **BAU** (*Business As Usual*) | Day-to-day running (vs project) | Target after go-live |
| **EUC** | *End-User Computing*: Excel, macros, tools outside IT | §30 |
| **SME** (*Subject Matter Expert*) | Business expert | — |
| **UAT** / **Parallel run** / **Go-live** / **Hypercare** | Test and production release phases | §25 |
| **CAB** (*Change Advisory Board*) | Committee that approves production releases | IT |
| **SteerCo** | Steering committee | §27 |
| **RAG** (*Red / Amber / Green*) | Project status color code | Status report |

### F.7 Typical phrases heard on the desk

| Phrase | What it means |
|---|---|
| *"The client didn't respond, we'll default them."* | No response: the default option is applied. |
| *"The offer got extended to the 23rd."* | The offer is extended: new deadline, chase the non-responders again. |
| *"We're short the protect, call Equity Finance."* | We don't have the securities to cover the protect: the recall needs to be expedited. |
| *"They over-instructed by 50k."* | The election exceeds the position by 50,000 shares. |
| *"DTC hasn't paid yet, it's sitting in unpaid."* | The payment hasn't arrived; the break is in the UNPAID category. |
| *"It's a tax break, the W-8 expired."* | Tax difference caused by expired tax documentation. |
| *"The admin is chasing us on the Granite special."* | The fund's administrator is following up on the Granite special dividend. |
| *"Raise a claim against the street."* | Open a claim against the external counterparty. |
| *"It's aged, we need to escalate before month-end."* | The break is old and must be escalated before the close (NAV, capital). |
| *"Is it contractual or actual?"* | Is the client credited on the pay date or on receipt of the payment? |
| *"That's a due bill situation."* | Trade in the due bill window: the distribution is owed to the buyer. |
| *"Check the Info Memo."* | Check the adjustment published by the OCC for the options. |
