# HESTIA Software License (Triple License)

Version 1.0.0

Copyright (C) 2026 AQUAXIS TECHNOLOGY. All Rights Reserved.

---

## Preamble

This software (hereinafter "this Software") is offered under one of three licenses, depending on the user's type of use. Before using, copying, modifying, or distributing this Software, users must determine their type of use in accordance with Chapter 0 and comply with the conditions of the corresponding license.

| Type of Use | Applicable License | Fee | Output Publication | Support |
|---|---|---|---|---|
| Non-Commercial Use | **License A: AGPL-3.0** | Free | No obligation | Community only |
| Commercial Use (Free) | **License B: Reciprocal License** | Free | Publication required | Community only |
| Commercial Use (Subscription) | **License C: Commercial Subscription License** | Paid (1,000,000 JPY/year excl. tax in Japan; USD 10,000/year outside Japan) | No obligation | Standard support (email, business hours, 3-business-day response, 4 cases/year) |

By using, copying, modifying, or distributing this Software, the user is deemed to have agreed to the applicable license for their type of use and the conditions of Chapter 0 and Chapter 4. If the user does not agree to the conditions of any license, the user must not use, copy, modify, or distribute this Software.

---

## Chapter 0: Definitions and Common Provisions

### 0.1 Commercial Use

"Commercial Use" means any use of this Software that falls under any of the following:

(a) Revenue (consideration, fees, usage charges, subscription fees, advertising revenue, brokerage commissions, or any other monetary or monetarily valuable consideration, regardless of designation) is actually being generated in connection with the use of this Software, whether to the user or a third party

(b) Revenue generation is reasonably foreseeable in connection with the use of this Software. "Reasonably foreseeable" means any of the following:

  (i) The user's business plan, budget, proposals, contracts, internal approval documents, or public information includes revenue or earnings premised on the use of this Software

  (ii) A product or service incorporating this Software is already being sold or is in a state ready for sale, and receipt of consideration is intended

  (iii) This Software is being used for for-profit contracted development, consulting, or SaaS provision

### 0.2 Exclusions from Commercial Use

Notwithstanding the provisions of Section 0.1, the following uses shall not be considered commercial use:

(a) Uses that clearly fall under the non-commercial purposes defined in Chapter 1 (License A)

(b) **Use by Non-Profit Organizations**

Use for **non-profit purposes** by charitable organizations, educational institutions (including accredited primary, secondary, and higher education institutions and research organizations), public research institutions, public safety and health organizations, environmental protection organizations, and government agencies, regardless of whether consideration is received and regardless of funding sources or obligations attached to such funding.

(c) **Limited Use for Evaluation and Verification**

Use of this Software without running it in a production environment, solely for the purpose of evaluation, verification, technical investigation, or internal experimentation, for a period not exceeding **180 days** from the date of first use. The same user (including the affiliated scope defined in Section 0.3) may use this provision again only after **12 months** have elapsed since the end of the previous use.

(d) **Use by Small Businesses**

Where the user's (or, if the user is a corporation, the total of the corporate group to which the user belongs) annual gross revenue for the most recent fiscal year is below the following threshold based on the location of the user's registered headquarters or principal place of business:

  (i) Users whose registered headquarters or principal place of business is within Japan: **Less than 10,000,000 JPY**

  (ii) Users whose registered headquarters or principal place of business is outside Japan: **Less than 50,000 USD**

For the application of (ii) above, if the user uses a currency other than US dollars, the amount shall be converted to US dollars at the exchange rate (Bank of Japan published rate or equivalent public exchange rate) on the last day of the user's most recent fiscal year.

If the user has offices both within and outside Japan, the location of the registered headquarters shall prevail.

However, users applying this provision must follow the self-declaration procedure separately established (http://aquaxis.com/declaration) and declare this to the licensor. If the declaration contains false or materially incorrect information, this provision shall retroactively cease to apply, and the user shall retroactively comply with the conditions of Chapter 2 or Chapter 3.

### 0.3 Revenue and Attribution

The revenue referred to in Sections 0.1 and 0.2(d) shall be attributed regardless of whether it is generated by the user, the user's employer, the business entity to which the user belongs or is affiliated, the client for whom the user provides contracted services, or an affiliated company under the user's management or control.

### 0.4 Ambiguity

If there is ambiguity as to whether a use of this Software constitutes commercial use, the user shall make a written inquiry to the licensor before commencing such use, and shall limit use to that based on Chapter 1 or Chapter 3 until an answer is received.

### 0.5 License Selection and Exclusivity

The user shall select **one license** from Chapter 1, Chapter 2, or Chapter 3, as applicable to their type of use, and apply it. If the type of use changes (e.g., from non-commercial to commercial, from commercial free to subscription), the user shall promptly complete the procedure to switch to the license applicable to the new type of use.

### 0.6 Outputs

In this license, "Outputs" means any information, data, code, models, documents, or other electronic products generated, output, or derived by the user through the use of this Software. However, the following are not included in Outputs:

(a) The user's own input data per se

(b) Information subject to third-party copyrights, trade secrets, personal information, or other legal protections that cannot be published by law or contract

---

## Chapter 1: License A: Non-Commercial License (AGPL-3.0)

### 1.1 License Designation

For users subject to this chapter (users determined to be non-commercial users in accordance with Chapter 0), this Software is offered under the terms of the **GNU Affero General Public License version 3.0 (AGPL-3.0)**.

The full text of AGPL-3.0 is included in `LICENSES/AGPL-3.0.txt` (or the `COPYING` file) distributed with this license package, or is available at:

https://www.gnu.org/licenses/agpl-3.0.txt

### 1.2 Scope Limitation

The license grant under AGPL-3.0 in this chapter applies **only to the non-commercial use defined in Chapter 0**. AGPL-3.0 is not granted to commercial users. Commercial users shall comply with the conditions of Chapter 2 or Chapter 3.

### 1.3 AGPL-3.0 Obligations

Users subject to this chapter must comply with all obligations defined by AGPL-3.0. These include, but are not limited to:

(a) **Copyleft Obligation**: When distributing modified versions of this Software, the obligation to distribute the entire modified version under AGPL-3.0

(b) **Source Code Publication Obligation upon Network Interaction (AGPL-3.0 Section 13)**: When modifying this Software and providing the modified version to third parties (including end users) over a network, the obligation to provide all users accessing the modified version over the network with a prominent opportunity to obtain the corresponding source code. This applies when this Software is used as a SaaS, web service, API, or other network service (the so-called "SaaS loophole" closure provision)

(c) **Source Code Provision Obligation**: The obligation to make the source code of this Software (or modified versions) available concurrently with any binary or network-based distribution

(d) **Attribution Obligation**: The obligation to maintain the copyright notices and license notices of AGPL-3.0

(e) **Change Notification Obligation**: The obligation to clearly state a summary of material changes made to this Software

### 1.4 Treatment of Outputs

Users subject to this chapter are **not obligated to publish Outputs** as defined in Chapter 0, Section 0.6. However, this does not exempt the user from the source code provision obligations for this Software itself (and modifications thereto) required by AGPL-3.0, including provision over a network.

### 1.5 Support

No support is provided to users subject to this chapter. Users may use the public community channels (http://aquaxis.com/community) at their discretion, but no response is guaranteed.

### 1.6 Transition to Commercial Use

If a user who was subject to this chapter begins commercial use, the user shall comply with the conditions of Chapter 2 or Chapter 3 from the point of commencement of such commercial use. In this case, modifications and distributions made under non-commercial use prior to commercialization continue to be governed by AGPL-3.0.

### 1.7 Prevention of Misconceptions Regarding AGPL-3.0

To prevent common misconceptions among users of this chapter, the following is hereby clarified:

(a) AGPL-3.0 does not require the publication of the user's entire application source code. The publication obligation is limited to **this Software itself and modifications thereto**.

(b) For applications built merely by using this Software without modification, the source code of such applications need not be published (however, whether they constitute derivative works or combined works depends on the specific technical architecture).

(c) The network interaction publication obligation of AGPL-3.0 (Section 1.3(b)) arises only when a **modified version** of this Software is provided over a network. Merely using this Software unmodified internally does not trigger the publication obligation.

---

## Chapter 2: License B: Reciprocal License (For Commercial Users at No Cost)

### 2.1 Definitions

In this chapter, the definitions in Chapter 0 and the following definitions apply:

(a) "This Chapter's Licensee" means a user who has been determined to be a commercial user in accordance with Chapter 0 and has not entered into a subscription agreement under Chapter 3.

(b) "Licensor" means AQUAXIS TECHNOLOGY.

### 2.2 License Scope

The Licensor hereby grants this Chapter's Licensee a worldwide, non-exclusive, non-transferable, revocable (subject to the conditions of Section 2.5) license, **free of charge**, to:

(a) Copy, run, and display this Software

(b) Modify this Software and create derivative works

(c) Use this Software for any purpose, including commercial purposes

(d) Distribute this Software and its derivative works (subject to the conditions of Section 2.5)

### 2.3 Patent License

The Licensor hereby grants this Chapter's Licensee a free, non-exclusive, worldwide patent license for patents owned or hereafter acquired by the Licensor that are necessary for the practice of this Software. If this Chapter's Licensee or its affiliated parties file a patent infringement lawsuit against this Software, the Licensor, or other contributors, the license under this section shall automatically terminate at the time such lawsuit is filed.

### 2.4 Output Publication Obligation (Core Provision of This Chapter)

This Chapter's Licensee is obligated to publish all **Outputs** (as defined in Chapter 0, Section 0.6) generated through commercial use of this Software under the following conditions:

**2.4.1 Publication License**

Outputs must be published under one of the following open source licenses:

- MIT License
- Apache License 2.0
- GNU General Public License version 3.0 (GPL-3.0)
- GNU Affero General Public License version 3.0 (AGPL-3.0)
- GNU Lesser General Public License version 3.0 (LGPL-3.0)
- Mozilla Public License 2.0 (MPL-2.0)

**2.4.2 Publication Medium**

Outputs must be placed in one of the following public repositories:

- Public repository services such as GitHub, GitLab, or Bitbucket
- A website operated by the user that is accessible to anyone on the internet
- Other publication media approved in writing by the Licensor in advance

**2.4.3 Publication Timing**

Outputs must be published within **30 days** of the date the user begins commercial use of those Outputs.

**2.4.4 Publication Duration**

Outputs must remain publicly available for a minimum of **12 months** from the start of publication. This period continues even if the user stops using this Software.

**2.4.5 Notification Obligation**

The user shall notify the Licensor in writing (including email) of the publication URL and applied license within **30 days** of starting publication.

**2.4.6 Publication Exemption Scope**

Information that falls under Chapter 0, Section 0.6(b) (information that cannot be legally published due to third-party rights, trade secrets, personal information, etc.) is exempt from the publication obligation for those specific portions only. However, if the majority or core portion of the Outputs falls under this exemption, the user must enter into a subscription agreement under Chapter 3 rather than using this chapter.

### 2.5 Effect of Violation

If this Chapter's Licensee fails to comply with the publication obligation defined in Section 2.4, the license under this chapter shall **automatically terminate**. After license termination, the user must take one of the following measures:

(a) Immediately cease all use of this Software and delete all copies of this Software

(b) Enter into a new subscription agreement under Chapter 3

The Licensor reserves the right to seek injunctive relief and damages under applicable law if the violation continues.

### 2.6 Support

No support is provided to this Chapter's Licensee. Users may use the public community channels (http://aquaxis.com/community) at their discretion, but no response time, resolution certainty, or any other service level is guaranteed.

### 2.7 Copyright and Attribution

This Chapter's Licensee must maintain the following notices in all copies of this Software and its derivative works:

(a) The original copyright notice of this Software

(b) A reference to this license (URL of this LICENSE file)

(c) If distributing derivative works, a clear statement that they are derivative works

### 2.8 Trademarks

This chapter does not grant any right to use the Licensor's trademarks, service marks, logos, or trade names.

---

## Chapter 3: License C: Commercial Subscription License

### 3.1 Definitions

(a) "This Chapter's Licensee" means a user who has been determined to be a commercial user in accordance with Chapter 0 and has entered into an active subscription agreement with the Licensor.

(b) "Subscription Agreement" means an individual agreement entered into between the user and the Licensor regarding the commercial use of this Software, which defines fees, billing units, contract period, support level, and other individual conditions.

(c) "Contract Period" means the effective period of the Subscription Agreement. **The standard contract period is 1 year (365 days)** and is renewed by continuation unless otherwise agreed. Multi-year lump-sum contracts and monthly contracts are not offered unless separately agreed.

### 3.2 License Scope

The Licensor hereby grants this Chapter's Licensee a worldwide, non-exclusive, non-transferable, revocable license to commercially use this Software during the contract period. The scope of rights granted shall be as defined in the Subscription Agreement.

### 3.3 Output Privacy Right

This Chapter's Licensee is **not obligated** to publish Outputs as defined in Chapter 2, Section 2.4. This Chapter's Licensee may freely use, modify, and distribute Outputs for its own business, customers, affiliates, or other purposes while keeping them private. However, this section does not transfer the copyright of this Software itself to this Chapter's Licensee, and unauthorized redistribution of this Software is prohibited.

### 3.4 Right to Receive Support

During the contract period, this Chapter's Licensee is entitled to the following support (hereinafter "Standard Support"):

(a) **Support Channel**: Email (support@aquaxis.com)

(b) **Support Hours**: Business days during business hours (weekday 9:00-18:00 JST, excluding year-end/new year and national holidays)

(c) **Initial Response Time**: Within 3 business days

(d) **Annual Inquiry Count**: Maximum 4 cases per contract year. Additional cases require enrollment in a separate paid option service

(e) **Support Language**: Japanese

(f) **Scope**: Technical questions, bug reports, configuration and installation assistance, and documentation questions regarding the current major version of this Software

While License A and License B licensees receive no support, this Chapter's Licensee receives direct support from the Company under the above conditions. This is what "priority" means in this chapter.

Details of support (case counting method, out-of-scope items, usage flow, Contractor cooperation obligations, disclaimers, etc.) are defined in the separate "Support Terms" (SUPPORT.md).

### 3.5 License Fees and Billing

**3.5.1 Standard License Fee**

The standard license fee per contract period (1 year) per contract is as follows, based on the location of this Chapter's Licensee's registered headquarters or principal place of business:

| Category | Location | Standard License Fee (per year / per contract) |
|---|---|---|
| Domestic | Users with registered headquarters or principal place of business within Japan | **1,000,000 JPY** (excluding consumption tax) |
| International | Users with registered headquarters or principal place of business outside Japan | **USD 10,000** (excluding applicable local taxes) |

The above fee is the standard fee per **one subscriber organization (a single contract entity within a corporation or corporate group)**.

**3.5.2 What Is Included in the Fee**

The standard license fee includes:

(a) Commercial use rights for this Software (Section 3.2)

(b) Output privacy rights (Section 3.3)

(c) Right to receive Standard Support (Section 3.4): Email, business hours, 3-business-day response standard support, **up to 4 cases per contract year**

(d) Right to receive standard maintenance updates and security patches

**3.5.3 What Is Not Included in the Fee (Optional Charges)**

The following are not included in the standard license fee and are charged separately:

(a) Additional inquiries beyond 4 per year (individual purchase of additional inquiry cases)

(b) On-site support, hands-on training, and custom development

(c) Additional fees for using this Software in extremely large-scale environments (exceeding separately defined scale thresholds)

(d) Support in languages other than Japanese

(e) Extended Support for prior major versions

**3.5.4 Billing Unit Exceptions**

By agreement between the Licensor and this Chapter's Licensee, different billing structures may be adopted through individual contracts. Exceptional billing units may include:

- Per-developer billing (number of people developing or debugging this Software)
- Per-production-server, CPU-core, or node billing
- Tiered billing based on the Licensee's annual revenue or number of employees
- Usage-based billing (API call count, data processing volume, etc.)
- Multi-year lump-sum discounts

**3.5.5 Consumption Tax and Value-Added Tax**

License fees for domestic (Japan) Licensees are subject to Japanese consumption tax, which is charged separately. License fees for international Licensees may be subject to applicable local taxes (VAT, GST, withholding tax, etc.), which the Licensee is responsible for paying.

**3.5.6 Payment Terms**

License fees shall be paid in a lump sum in advance at the time of contract execution or renewal. Details of payment methods, currency, and invoicing shall be defined in the individual contract.

### 3.6 Contract Termination

(a) **Expiration**: If the subscription agreement expires and is not renewed, the license under this chapter terminates on the expiration date.

(b) **Early Cancellation by User**: The user may cancel the subscription in accordance with the conditions defined in the subscription agreement. Paid license fees are non-refundable unless the subscription agreement specifies otherwise.

(c) **Termination by Licensor**: If the user materially breaches this license or the subscription agreement and fails to cure within 30 days after receiving a cure notice from the Licensor, the Licensor may terminate the contract.

### 3.7 Post-Termination Measures

If the license under this chapter terminates, the user must promptly take one of the following measures:

(a) Completely cease use of this Software and delete all copies

(b) Continue use under the conditions of Chapter 2 (License B). In this case, **Outputs generated after the termination date** are subject to the publication obligation under Chapter 2, Section 2.4. Outputs generated before the termination date are not retroactively subject to publication obligations.

(c) Continue use only for non-commercial purposes that satisfy the conditions of Chapter 1 (License A, AGPL-3.0)

### 3.8 Precedence of Individual Agreements

In the event of any conflict between the provisions of this chapter and the provisions of the subscription agreement, **the provisions of the subscription agreement shall prevail**. However, if the subscription agreement modifies essential conditions of this license, such modification must be explicitly agreed to in writing.

---

## Chapter 4: Common Provisions (Applicable to All Licenses)

### 4.1 Disclaimer of Warranties

This Software is provided "**AS IS**." The Licensor makes no warranties of any kind, express or implied, regarding this Software. This includes, but is not limited to, warranties of merchantability, fitness for a particular purpose, non-infringement of third-party rights, accuracy, completeness, continuity, security, and the absence of errors, bugs, or vulnerabilities. No warranty is made that this Software will meet the user's requirements, operate without interruption, or be compatible with other software or hardware.

### 4.2 Disclaimer and Limitation of Liability

The Licensor, its officers, employees, affiliates, agents, and contributors shall not be liable for **any damages** arising from or related to the use, inability to use, malfunction, interruption, defects, security breaches, data loss or corruption, or system interruption or failure involving this Software or systems incorporating this Software, whether the user or third parties, under any legal theory (contract liability, tort liability, product liability, strict liability, or otherwise).

The damages referred to in the preceding paragraph include, but are not limited to, all of the following:

(a) Direct damages

(b) Indirect damages, special damages, incidental damages, and consequential damages

(c) Lost profits, reduced revenue, lost business opportunities

(d) Loss, corruption, or leakage of data, content, or Outputs

(e) System downtime, business interruption, recovery costs, alternative procurement costs

(f) Damage to reputation or goodwill, deterioration of customer relationships

(g) Costs of responding to third-party claims (including attorney fees)

The above disclaimer of liability applies even if the Licensor was previously notified of the possibility of such damages.

### 4.3 Limitation by Mandatory Law

The provisions of this chapter do not apply to the extent that they cannot be disclaimed or limited under applicable mandatory law. However, even in such cases, the Licensor's liability is limited to the **minimum extent permitted by law**. For consumer contracts in Japan, this disclaimer does not apply to damages caused by the Licensor's willful misconduct or gross negligence.

### 4.4 Special Provisions for Subscription Users

For users who have entered into a paid subscription agreement under Chapter 3, the provisions of this chapter may be modified to the extent specified in the separately executed subscription agreement regarding the limitation of liability. However, unless such modification is made, the provisions of this chapter shall prevail.

### 4.5 User Indemnification

The user shall, at its own expense and responsibility, resolve any disputes with third parties arising from the use of this Software, and shall indemnify and defend the Licensor against any damages, costs (including attorney fees), or liabilities arising from such disputes.

### 4.6 Export Control

The user shall comply with all applicable export control laws and regulations (including Japan's Foreign Exchange and Foreign Trade Act, US Export Administration Regulations, and others) in connection with the use, copying, modification, and distribution of this Software.

### 4.7 Governing Law and Jurisdiction

This license shall be governed by **Japanese law**. Any and all disputes arising in connection with this license shall be submitted to the **Tokyo District Court** as the court of exclusive agreed jurisdiction of first instance.

### 4.8 Severability

If any provision of this license is held invalid or unenforceable, the validity of the other provisions shall not be affected. The invalid or unenforceable provision shall be reformed to the provision that most closely approximates the intent and effect of the original provision and is valid and enforceable.

### 4.9 Entire Agreement

This license (in the case of Chapter 3, this license and the subscription agreement) constitutes the entire agreement between the user and the Licensor regarding the use of this Software and supersedes all prior agreements, understandings, representations, or warranties regarding this matter.

### 4.10 License Revision

The Licensor may publish future versions of this license. Users who have obtained this Software under a specific version of this license shall not lose their rights under that version. However, users may voluntarily migrate to future versions.

### 4.11 Contact

Inquiries, declarations, and notices regarding this license shall be directed to:

- Licensor: AQUAXIS TECHNOLOGY
- Email: contact@aquaxis.com
- Website: http://aquaxis.com

### 4.12 Language and Authoritative Text

This license and all documents related to this license (including the attached "Support Terms," CLA, Self-Declaration System Operations Manual, FAQ, etc.) are **written in Japanese as the authoritative version**.

Even if a translation of this license (English or other languages) is produced, such translation is provided for the convenience of users only. In the event of any interpretive differences, contradictions, or ambiguities between the Japanese authoritative text and the translation, **the Japanese authoritative text shall prevail**.

This provision **applies equally when the user is located outside Japan and when this Software is used outside Japan**, and users who do not read Japanese are obligated to adequately understand the content of the Japanese authoritative text at their own expense and responsibility before agreeing to this license.

This provision, together with Section 4.7 (Governing Law and Jurisdiction), serves as the standard for legal interpretation agreed upon by both the Licensor and the user.

---

## Appendix A: Source Code Header Template

All source files of this Software shall include the following header:

```
/*
 * [File name / Project name]
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-ProductName-Commercial
 *
 * Copyright (C) 2026 AQUAXIS TECHNOLOGY. All Rights Reserved.
 *
 * This software is triple-licensed. You may use it under one of:
 *   (A) GNU Affero General Public License version 3 (AGPL-3.0)
 *       — for non-commercial use only
 *   (B) HESTIA Reciprocal Commercial License 1.0
 *       — for commercial use, with obligation to publish derived Outputs
 *   (C) HESTIA Commercial Subscription License
 *       — for commercial use, paid, with priority support and no Output
 *       publication obligation
 *
 * See LICENSE.md for full terms. Commercial use without a subscription
 * requires compliance with the Output publication obligation.
 *
 * THIS SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.
 */
```

## Appendix B: User Selection Flowchart

```
        Want to use this Software
                    |
                    v
        ┌───────────────────────┐
        │ Determine whether      │
        │ commercial or          │
        │ non-commercial per     │
        │ Chapter 0 Sections 0.1-0.2
        └───────────┬───────────┘
                    |
      ┌─────────────┴─────────────┐
      |                           |
      v                           v
   Non-Commercial              Commercial
   Use                         Use
      |                           |
      v                           v
 ┌─────────────┐         ┌───────────────────┐
 │ License A   │         │ Subscription        │
 │ (AGPL-3.0)  │         │ agreement?          │
 │             │         └─────────┬─────────┘
 │ - Free      │                   │
 │ - No output │       ┌───────────┴──────────┐
 │   publication│       |                      │
 │   obligation │       v                      v
 │ - AGPL      │  No agreement          Has agreement
 │   obligations│     |                        │
 │   (modified │     v                        v
 │   source    │ ┌─────────────┐      ┌─────────────┐
 │   publication│ │ License B   │      │ License C   │
 │   for net    │ │ (Reciprocal) │      │ (Subscription)│
 │   use, etc.) │ │             │      │              │
 │ - No support│ │ - Free      │      │ - Paid       │
 │             │ │ - Publish   │      │ - No output  │
 └─────────────┘ │   outputs   │      │   publication│
                 │   unconditionally│      │   obligation │
                 │ - No support │      │ - Priority   │
                 │             │      │   support     │
                 └─────────────┘      │   right       │
                                      └─────────────┘
```

## Appendix C: Small Business Self-Declaration Form (Reference Template)

```
─────────────────────────────────────────────
Small Business Self-Declaration Form (License B/C Exemption Declaration)

1. Declarant Information
   Organization name    : ________________________
   Registered headquarters or principal place of business : [ ] Within Japan / [ ] Outside Japan
   Representative name   : ________________________
   Contact person       : ________________________
   Email                : ________________________

2. Revenue Information
   Most recent fiscal year : ____/____ - ____/____
   Annual gross revenue   : ________________________ JPY/USD (select one)
   Corporate group existence : [ ] None / [ ] Yes (if applicable, group revenue is aggregated)
   Group aggregated total revenue: ________________________ JPY/USD

3. Intended Use of This Software
   ________________________________________
   ________________________________________

4. Declaration
   I declare that the information stated in this document is true and accurate,
   and that I will notify within 30 days if my revenue exceeds the applicable
   threshold (10,000,000 JPY for Japan-based entities, 50,000 USD for
   entities outside Japan).

   Date: ____/____/____
   Signature: ________________________
─────────────────────────────────────────────
```