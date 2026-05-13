# HESTIA License FAQ (AGPL-3.0 + Commercial Dual)

Version 1.0 Draft

This FAQ answers common questions about using HESTIA. In the event of any inconsistency between this FAQ and the terms of `LICENSE.md`, **the terms of `LICENSE.md` take precedence**. This FAQ does not constitute legal advice; please consult an attorney as needed.

---

## Table of Contents

1. [Basics: Understanding the License Structure](#1-basics-understanding-the-license-structure)
2. [License Determination: Which License Applies to Me](#2-license-determination-which-license-applies-to-me)
3. [AGPL-3.0 Obligations: Duties of Non-Commercial Users](#3-agpl-30-obligations-duties-of-non-commercial-users)
4. [Outputs: Understanding License B's Publication Obligation](#4-outputs-understanding-license-bs-publication-obligation)
5. [Technical Scenarios: Specific Use Cases and Obligations](#5-technical-scenarios-specific-use-cases-and-obligations)
6. [License Transition: When Circumstances Change](#6-license-transition-when-circumstances-change)
7. [Commercial Subscription: License C Details](#7-commercial-subscription-license-c-details)
8. [Contributors: About the CLA](#8-contributors-about-the-cla)
9. [Compliance and Legal](#9-compliance-and-legal)

---

## 1. Basics: Understanding the License Structure

### Q1-1. Is HESTIA open source software?

Yes, **License A adopts the GNU Affero General Public License version 3.0 (AGPL-3.0)**, and AGPL-3.0 is an OSI-approved open source license.

However, this software employs a triple-license structure, and **AGPL-3.0 is offered only to non-commercial users; commercial users must comply with License B or License C**.

### Q1-2. Why are there three licenses?

To provide freedom to the non-commercial community as open source, while ensuring that commercial use contributes to the project (either through code publication or financial contribution), thereby sustaining the project.

- **License A (AGPL-3.0, for non-commercial users)**: Completely free, open source. Supports individuals, research, NPOs, and the community
- **License B (Reciprocal, for commercial users at no cost)**: Commercial use is free, but you contribute by publishing your outputs as open source
- **License C (Commercial subscription, for paying commercial users)**: You contribute through monetary payment; outputs may remain private; priority support included

### Q1-3. Is this the same model as MySQL or Redis?

Very similar. MySQL uses GPL+commercial, and Redis (as of 2025) uses a triple license of AGPL+RSAL+SSPL. This project follows the same philosophy as the multi-license structures that Redis and Elastic adopted in 2024-2025, which include AGPL.

### Q1-4. Is this a "no commercial use" open source license?

**No, commercial use is not prohibited.** Commercial use is fully permitted, but in that case you must choose License B or License C instead of License A (AGPL-3.0).

This is consistent with the fact that AGPL-3.0 itself does not prohibit commercial use (it guarantees all four freedoms). This project simply does not offer AGPL-3.0 to commercial users.

---

## 2. License Determination: Which License Applies to Me

### Q2-1. I want to use this for personal hobby programming. Which license?

**License A (AGPL-3.0).** Personal hobbies, learning, and self-improvement do not constitute "commercial use" as defined in LICENSE.md Chapter 0, Section 0.1.

### Q2-2. I want to use this for academic research in a university lab. Which license?

**License A (AGPL-3.0).** Under LICENSE.md Chapter 0, Section 0.2(b), use by educational institutions and research organizations for non-profit purposes is excluded from commercial use.

### Q2-3. I want to use this in an NPO. Our funding comes from grants. Which license?

**License A (AGPL-3.0).** Under LICENSE.md Chapter 0, Section 0.2(b), use by charitable organizations for non-profit purposes is excluded from commercial use regardless of funding source.

### Q2-4. I'm at a startup and plan to incorporate this into a product we're developing, but we have no revenue yet. Which license?

**License B or License C.** Even with zero revenue, if "revenue generation is reasonably foreseeable" (LICENSE.md 0.1(b)) -- for example, if it's stated in your business plan -- it constitutes commercial use.

However, if your annual gross revenue is below **10,000,000 JPY (for Japan-based entities) or USD 50,000 (for entities outside Japan)**, you may use it under License A through the small business relief self-declaration procedure per LICENSE.md Chapter 0, Section 0.2(d).

### Q2-5. I'm an employee at a for-profit company, but I just want to evaluate it on my own PC first. Which license?

**License A (AGPL-3.0)** during the evaluation period. Under LICENSE.md Chapter 0, Section 0.2(c), use for evaluation and verification purposes without running in production is excluded from commercial use for up to 180 days from the date of first use.

After 180 days, or when you begin using it in production, you must switch to License B or License C. Note that this evaluation period relief can only be reused after 12 months have elapsed since the end of the previous use (to prevent infinite loops).

### Q2-6. I want to use this for internal business efficiency at a for-profit company. No external revenue is generated. Which license?

**License B or License C.** Even without direct revenue, using this software to improve business efficiency (i.e., generating economic benefit) constitutes commercial use. LICENSE.md Chapter 0, Section 0.1 defines "revenue" to include "consideration of monetary value," so efficiency gains fall within the commercial scope.

However, the small business relief (0.2(d)) or the evaluation period relief (0.2(c)) may apply, as described in Q2-4.

### Q2-7. I'm a freelance contractor incorporating this into a client's product. I'm a sole proprietor. Which license?

**License B or License C.** Contracted development work is explicitly commercial use under LICENSE.md Chapter 0, Section 0.1(b)(iii). It does not matter whose revenue is generated -- yours or the client's (Section 0.3).

If the client is a License C subscriber, you may be covered under that subscription as a "contracted service provider." Please confirm with the client in advance.

### Q2-8. The boundary between "commercial" and "non-commercial" is unclear to me. What should I do?

Under LICENSE.md Chapter 0, Section 0.4, you should **make a written inquiry to the licensor before starting use**. Until you receive an answer, you must limit your use to License A or License C.

---

## 3. AGPL-3.0 Obligations: Duties of Non-Commercial Users

### Q3-1. If I use AGPL, do I have to publish the source code of my entire application?

**No. This is the most common misconception about AGPL.**

AGPL-3.0 requires publication of **the source code of this software itself (and any modifications thereto)**, not your entire application.

For example, if you build a cloud application that calls this software, you are not required to publish the business logic, domain models, or UI of that application. The publication obligation arises only if you modify this software itself and provide it over a network.

### Q3-2. Does merely "using" this software trigger a source code publication obligation?

**No. Using this software unmodified does not trigger any source code publication obligation.**

AGPL's source publication triggers are:

1. **Distribution**: Distributing modified binaries or modified source code to third parties
2. **Network interaction** (AGPL Section 13): Running a modified version on your servers and providing its functionality to third parties over a network

Simply "using" it without distributing or providing it over a network does not trigger either.

### Q3-3. I modified it internally, and only my colleagues within the same company use it. Is there a publication obligation?

**Generally, no.** AGPL's "network interaction" provision refers to providing to "users," and **providing within the same legal entity is generally not considered to fall within this scope** (however, sharing across entities or to affiliates may constitute "providing to third parties").

However, if internal use constitutes **commercial use** (see Q2-6), you would not be eligible for License A in the first place and would need License B or License C.

### Q3-4. I modified this software, built a SaaS, and offer it for free. What are my publication obligations?

**Yes, a publication obligation for your modifications arises.**

Under AGPL Section 13, if you provide a modified version to users over a network, you must provide those users with prominent access to the source code of the modified portion. Specifically, one of the following is required:

- Display a "Source Code" link within the application and publish the modified source on GitHub or similar
- Include the source code URL in API response headers
- Clearly state how to obtain the source in a user manual or footer

### Q3-5. If I offer a modified version as SaaS, can I proceed under License A (AGPL) even if it's paid?

**No, you cannot.** Once you receive compensation for SaaS, it constitutes commercial use (Chapter 0, Section 0.1), so License A is not available and you must choose License B or License C.

### Q3-6. I plan to submit a bug fix Pull Request to this project. Do I still need to publish my code?

When you submit a PR to this project, you grant the licensor re-licensing rights for the PR code portion via the CLA. This is an ideal use in the spirit of AGPL -- contributing modifications back to the community.

On the other hand, your application-side code (the code that calls this software) does not need to be published, as stated in Q3-1.

---

## 4. Outputs: Understanding License B's Publication Obligation

### Q4-1. What exactly are "Outputs" under License B?

Under the definition in LICENSE.md Chapter 0, Section 0.6, "Outputs" means "any information, data, code, models, documents, or other electronic products generated, output, or derived by using this software."

The nature of outputs depends on the software's functionality. For example:

- Code generator -> generated code
- AI training tool -> trained model weights
- Data processing tool -> processed results/data
- Document generation tool -> generated documents

However, **your original input data itself** is not included in Outputs (0.6(a)).

### Q4-2. What's the difference between License B's and License A's (AGPL) publication obligations?

The key difference is **what must be published**.

| | License A (AGPL) | License B |
|---|---|---|
| Publication target | Modifications to this software itself | Outputs generated using this software |
| Trigger | Distribution / network interaction of modified version | Commencement of commercial use |
| Publication format | Under AGPL-3.0 | Under a specified open source license |

License A concerns "the source code of the software itself," while License B concerns "outputs created using the software" -- **completely different subjects**.

### Q4-3. My company's confidential information and customer data are in the outputs. I can't publish them. What should I do?

You have the following options:

1. **Partial publication**: Under LICENSE.md 2.4.6, portions containing information that cannot be published are exempt from the publication obligation. However, this only applies when specific portions are unpublishable due to third-party rights, etc. -- simply "wanting to keep it secret" does not qualify.

2. **When the majority or core of the outputs are unpublishable**: Under the proviso of LICENSE.md 2.4.6, **you must switch to License C (subscription)**.

3. **Separation from input data**: Clearly separate what is publishable (derived from this software's processing) from what is not (original input data) before publishing.

In practice, commercial projects handling customer data typically choose **License C**.

### Q4-4. I published the outputs under the MIT License. Does that fulfill License B's obligation?

Yes. The MIT License is one of the specified licenses in LICENSE.md 2.4.1. Others include Apache-2.0, GPL-3.0, AGPL-3.0, LGPL-3.0, and MPL-2.0.

However, you must also satisfy the publication medium (2.4.2), timing (within 30 days, 2.4.3), duration (12 months continuous, 2.4.4), and notification obligation (within 30 days, 2.4.5) requirements.

### Q4-5. How do I provide the publication notification for outputs?

Under LICENSE.md 2.4.5, within 30 days of starting publication, send the following information to the licensor by email:

- Publication URL (e.g., GitHub repository)
- Applied license (e.g., MIT, Apache-2.0)
- Publication start date
- Identifying information about you/your organization

Send to: legal@aquaxis.com

### Q4-6. Can I take down the publication after one year?

Yes, under LICENSE.md 2.4.4, the minimum publication period is 12 months. After 12 months, you may stop publishing. Note the following:

- The 12-month period starts from the "publication start date"
- Even if you stop using this software within the 12 months, the publication obligation continues for the full period
- Taking down the publication does not mean License B is terminated

### Q4-7. I want to incorporate code generated by this software into a customer's closed-source product and deliver it. Is this possible under License B?

**No, it is not.** License B imposes a publication obligation on outputs, so the customer cannot keep those outputs closed.

In this case, you need **License C (subscription)**. License C grants the right to keep outputs private (Section 3.3), allowing you to incorporate outputs into a customer's closed-source product.

---

## 5. Technical Scenarios: Specific Use Cases and Obligations

### Q5-1. I'm distributing this software in a Docker container. What are the AGPL implications?

Containerizing with Docker is "packaging," not "modification," so it does not trigger AGPL's modification trigger. However:

- If you distribute the container to third parties, you must maintain AGPL's attribution requirements (copyright notices, license notices)
- If you modify this software inside the container, those modifications are subject to AGPL's distribution/network interaction triggers
- If you run the container on a server and provide its functionality to users over a network, AGPL Section 13 applies

In short, **containerization itself is fine, but you need to look at whether the contents are modified and whether network interaction is involved**.

### Q5-2. I'm just calling this software's REST API from my own application. What are the implications?

Calling unmodified this software via API means **your application code is not affected by AGPL**.

AGPL's copyleft triggers when "this software is modified," and does not propagate to applications that "merely call this software." API calls are generally not considered derivative works, so your application is treated as an independent work.

However, **if your use is commercial, you cannot choose License A in the first place**. See Q2-6 and similar questions for that scenario.

### Q5-3. I statically linked this software as a library. Does it become a derivative work?

As a general rule, static linking is more likely to be considered a "combined work" and may fall under AGPL's copyleft scope. This could potentially require your entire application to be distributed under AGPL.

To avoid this risk, consider **dynamic linking** (shared libraries / DLLs), **inter-process communication** (APIs, IPC, gRPC), or switching to **License B or License C**.

If you intend to use this software as a library commercially, **License C subscription is recommended**. Subscribers have the right to keep outputs private, making embedded use safe.

### Q5-4. I'm developing a plugin (add-on). Will my plugin be subject to AGPL?

Whether a plugin is affected by AGPL depends on the technical architecture and legal interpretation.

- **Runs as an independent process, communicates via standard APIs**: Likely treated as an independent work -> may not be subject to AGPL
- **Directly calls this software's internal APIs or depends on internal data structures**: More likely to be considered a derivative work -> may be subject to AGPL

Many AGPL/GPL projects on Linux state a policy of treating plugins as independent works, but **you should confirm this project's policy with AQUAXIS TECHNOLOGY individually**.

### Q5-5. I'm creating separate software that communicates with this software. Is my software also under AGPL?

When communicating across process boundaries via network or IPC, your software is generally treated as an independent work and is not affected by AGPL.

However, the FSF takes the position that "tightly coupled communication" or "sharing data structures" may constitute a derivative work. If you're unsure, consult an attorney or consider License C.

### Q5-6. I want to fork this software and publish it as a completely separate project. Is that possible?

Yes, **it is possible within the scope of AGPL**. If you fork and publish:

- The forked project must be published under AGPL-3.0
- You do not have the right to redistribute under License B or License C (without a CLA)
- The licensor's trademarks (product name, logo) may not be used (LICENSE.md Section 2.8)

To avoid trademark confusion, **give the forked project its own name**.

### Q5-7. Can I use this software's source code as AI training data?

This is a novel issue not anticipated by AGPL, and the industry is still debating it. As a general matter:

- Incorporating source code into training data and internalizing it within a model may constitute reproduction or adaptation, potentially subjecting it to AGPL obligations
- If a trained model "memorizes" and outputs AGPL-covered code, AGPL obligations may propagate to that output

This project takes the position that **prior written permission is required for AI training use**. Commercial AI training use requires License C (with special provisions).

---

## 6. License Transition: When Circumstances Change

### Q6-1. I was using this as non-commercial, but I'm starting a business and going commercial. How do I transition?

At the point of commencing commercial use, you must choose License B or License C (LICENSE.md Section 1.6).

Key points:

- **Modifications and distributions made before commercialization under non-commercial use remain under AGPL-3.0** (no retroactive effect)
- **From the date commercial use begins**, License B's output publication obligation or License C's subscription fees apply
- The definition of commercial use is in LICENSE.md Section 0.1 (revenue generated or reasonably foreseeable)

### Q6-2. I was on License B, but now I need to incorporate outputs into a customer's closed-source product. Can I switch to License C?

Yes. By entering into a License C subscription agreement, you can immediately transition to License C.

Treatment of outputs after transition:

- **Outputs generated on or after the contract start date** are not subject to publication obligations (Section 3.3)
- **Outputs generated before the contract start date that have already been published** remain as-is (no retroactive unpublishing)
- **Outputs generated before the contract start date that have not yet been published** require individual discussion regarding publication obligations

In practice, **carefully plan the timing of transitioning from License B to License C**. Outputs already within their 12-month publication obligation period (Section 2.4.4) must complete that publication period even after transitioning to License C.

### Q6-3. I canceled my License C subscription. Can I keep using the software?

After cancellation, you must choose one of the options in LICENSE.md Section 3.7:

(a) Completely stop using the software and delete all copies

(b) Transition to License B (outputs from this point forward are subject to publication obligations)

(c) Switch to non-commercial use and transition to License A (only if you cease all commercial use)

Important note on (b): **Outputs generated before the contract end date are not retroactively subject to publication obligations** (Section 3.7(b) proviso). However, commercial use after the contract end date is subject to License B's rules.

### Q6-4. My evaluation period (180 days) has ended. What should I do?

The evaluation period relief under LICENSE.md Section 0.2(c) expires after 180 days. After the period ends, choose one of the following:

- If you want to move to production or continue evaluation -> License B or License C
- If you want to stop evaluating -> Cease use and delete copies
- If you want to continue evaluation further -> You may reuse the evaluation period relief after 12 months have elapsed since the end of the previous use

### Q6-5. I was receiving small business relief, but my revenue exceeded the threshold. What should I do?

Under LICENSE.md Section 0.2(d), as part of your self-declaration, you are obligated to notify the licensor within 30 days of when your revenue exceeds the threshold.

After notification:

- **Use on or after the date the threshold was exceeded** becomes subject to License B or License C
- **Use before the threshold was exceeded** remains covered by the relief period
- Failure to declare may result in retroactive loss of relief, and past use may be treated as commercial

---

## 7. Commercial Subscription: License C Details

### Q7-1. What is License C's pricing?

**The standard license fee, based on the subscriber's location, is as follows (see LICENSE.md Section 3.5):**

| Category | Location | Standard License Fee (per year / per contract) |
|---|---|---|
| Domestic | Within Japan | **1,000,000 JPY** (excluding tax, consumption tax additional) |
| International | Outside Japan | **USD 10,000** (excluding applicable local taxes) |

- The standard contract period is **1 year** (multi-year or monthly contracts are not offered unless separately agreed)
- The license fee includes: commercial use rights, output privacy rights, and standard support (email, business hours, 3-business-day response, **up to 4 cases per year**)
- The following are available at additional cost (optional):
  - **Additional inquiries beyond 4 per year**
  - On-site support, hands-on training, custom development
  - Extremely large-scale deployments
  - Support in languages other than Japanese
  - Extended support for prior major versions
- Individual contracts may offer different billing units (per developer, per server, usage-based pricing) and multi-year discounts

For specific quotes, visit http://aquaxis.com/sales.

### Q7-2. What does "priority support" in the subscription mean?

The term "priority" means that **License A and B licensees receive no support, whereas License C licensees receive direct support from us**.

Standard support consists of the following single plan (see LICENSE.md Section 3.4 and SUPPORT.md Chapter 2):

| Item | Details |
|---|---|
| Support channel | Email only (support@aquaxis.com) |
| Support hours | Business days during business hours (weekday 9:00-18:00 JST) |
| Initial response | Within 3 business days |
| **Annual case limit** | **4 cases / contract year** |
| Support language | Japanese |

No tiered plans, ticket systems, or additional channels such as Slack, phone, or video conferencing are provided. More advanced support (on-site, custom development, English language, extended support, additional inquiries beyond 4 per year, etc.) is available as separate paid services upon consultation.

### Q7-3. We have 10 developers and 3 production servers. How is pricing calculated?

**The standard fee is "per subscriber organization."** Therefore, even with 10 developers and 3 production servers, **1 contract (1,000,000 JPY/year for Japan-based entities, or USD 10,000/year for entities outside Japan) covers the entire setup**.

However, the following may result in separate pricing under individual contracts (see LICENSE.md Sections 3.5.3, 3.5.4):

- Extremely large-scale environments (exceeding separately defined scale thresholds)
- Use across multiple independent companies or business entities (cross-group)
- If you explicitly want per-developer or per-server billing

For an accurate quote, provide your specifications at http://aquaxis.com/sales.

### Q7-4. Can I keep outputs completely private during my subscription?

Yes. Under LICENSE.md Section 3.3, License C licensees have no obligation to publish outputs. You are free to incorporate outputs into customers' closed-source products, process confidential internal data, or generate proprietary outputs.

However, **redistribution of this software itself** is a separate matter; even subscribers may not redistribute this software without authorization (Section 3.3 proviso).

### Q7-5. How fast is the subscription support response?

Standard support initial response time is **within 3 business days**. Support is provided during business hours on business days (weekday 9:00-18:00 JST, excluding year-end/new year and national holidays), and the only channel is email. Formal terms are defined in the separate "Support Terms" (SUPPORT.md Chapter 2).

For comparison, License A / B licensees have access only to community channels, with no response guarantees.

### Q7-6. How are the "4 cases per year" counted?

As defined in SUPPORT.md Section 2.6.2, cases are counted as follows:

- **1 question / 1 issue = 1 case**. Multiple email exchanges about the same issue count as a single case
- Related follow-up questions arising from the original inquiry (matters that are an extension of the same issue) are treated as the same case
- Questions on a **different topic** from the original inquiry are counted as separate cases
- Inquiries determined to be **outside the scope of this service** (e.g., questions about third-party software) are not counted
- Reopened cases where reinvestigation is needed due to the licensor's circumstances are not counted

Cases **reset each contract year**, and unused cases do not carry over to the next year. The remaining case count is notified upon completion of each case.

### Q7-7. What happens if I use all 4 annual cases?

You have three options:

1. **Purchase additional inquiry cases individually**: Additional inquiry cases are available as a separate paid option. Pricing and unit quantities are individually quoted. Contact support@aquaxis.com
2. **Wait until the next contract year**: If not urgent, cases reset at the next renewal
3. **Refer to public documentation and community channels**: Visit http://aquaxis.com/faq and http://aquaxis.com/community (no response guarantees from the licensor)

On-site support, custom development, English language support, and extended support for prior versions are also available as separate paid services upon consultation.

### Q7-8. Can I get a refund if I cancel within 30 days?

Under LICENSE.md Section 3.6(b), paid license fees are generally non-refundable (unless the subscription agreement specifies otherwise). Cancellation takes effect on the last day of the contract period.

### Q7-9. I want to use this software on a private cloud. Do I need a subscription?

If "private cloud" means internal use that constitutes commercial use (see Q2-6), then **License B or License C** is required.

Under License B, there is an output publication obligation, and outputs generated in the cloud environment (logs, processed results, models, etc.) may be subject to publication. To avoid this, **License C is the practical choice for cloud deployments**.

---

## 8. Contributors: About the CLA

### Q8-1. Why can't my PR be merged without signing the CLA?

Because this project uses a dual-license structure, the licensor needs to obtain from contributors "the right to re-license not only under License A but also under License B/C." The CLA provides the legal mechanism for this. Without a CLA, your contribution could not be distributed under License B/C.

### Q8-2. Does signing the CLA mean I give up my copyright?

**No, you do not give up your copyright.** Under the CLA (Section I-4), you retain your copyright. You only grant the licensor broad licensing and re-licensing rights.

### Q8-3. After signing the CLA, can I change my mind and withdraw my contribution?

For future contributions, you can withdraw by providing written notice of CLA termination. However, **the license grant for contributions already submitted is perpetual and irrevocable** (Section I-8). This prevents the project from becoming legally unstable due to a contributor's retroactive withdrawal.

### Q8-4. I'm an employee at a company, but I want to contribute on my own time. What should I do about the CLA?

Check the box in CLA Section I-7 confirming that your employer has no rights over the contribution. However, please verify that:

- You are not using a company PC or network
- You are doing this outside of work hours
- You are not using company confidential information or work knowledge
- Your employment contract does not assign your personal creative works to the company

If unsure, we strongly recommend consulting your company's legal or compliance department in advance.

### Q8-5. How do I get my company to sign the CLA (CCLA)?

The typical process is:

1. Present this CCLA draft to your company's legal or compliance department
2. Create a list of designated contributors within the company (names, emails, GitHub usernames)
3. Have an authorized company officer sign or electronically execute the CCLA
4. Submit the signed document and designated contributor list to the licensor

### Q8-6. After signing the CCLA, how do I add new employees?

Under CCLA Section II-6, you can add new designated contributors by notifying the licensor via email from the contact person. A new CCLA is not required.

### Q8-7. Are bug reports and feature suggestions also subject to the CLA?

The CLA's definition of "Contribution" (Common Definition (b)) includes issue reports, feature suggestions, documentation improvements, and translations. However, simple bug reports and feature suggestions without source code are often accepted without CLA signatures in practice. Check this project's guidelines at http://aquaxis.com/community/guidelines.

---

## 9. Compliance and Legal

### Q9-1. My company's legal department says "AGPL is banned." What should I do?

Many companies restrict AGPL use by internal policy. In this case, you can address it with one of the following:

1. **Switch to License B**: Apply License B (reciprocal) instead of AGPL -> Does not fall under the AGPL internal policy, but output publication obligations apply
2. **Switch to License C**: Completely avoid AGPL through a subscription contract -> No internal policy issues either

Choose based on your company's situation. If the goal is to avoid AGPL entirely, License C is the cleanest choice.

### Q9-2. An OSS audit detected this software. How should I respond?

Audit tools (FOSSA, Black Duck, Snyk License, etc.) will often detect this software as AGPL-3.0. If your company is actually using it under License C, you should:

1. Explain to the audit tool that "this project uses a triple-license structure, and we are using it under License C"
2. Attach a copy of your subscription agreement to the audit log
3. Confirm that this software's file headers contain the License C SPDX identifier (`LicenseRef-ProductName-Commercial`, etc.)

This will exempt you from AGPL obligations in the audit report.

### Q9-3. What SPDX identifier should I use?

As shown in `LICENSE.md` Appendix A, we recommend:

```
SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-ProductName-Commercial
```

For commercial users (License B/C), record an identifier corresponding to your actually selected license separately in your internal SBOM (Software Bill of Materials) management.

### Q9-4. How should I register License B/C contracts in our contract management system?

You can organize them in a typical contract management system as follows:

- **Contract type**: Software license agreement
- **Counterparty**: AQUAXIS TECHNOLOGY
- **Classification**: License B (free reciprocal) / License C (paid subscription)
- **Contract start/end dates**: As stated in the License C subscription agreement
- **Key obligations**: License B -- output publication obligation; License C -- subscription fee payment
- **Renewal terms**: For License C, whether auto-renewal is in effect and cancellation notice deadlines

### Q9-5. In an M&A (company acquisition), how is this software's license transferred?

This is a common issue for both License B and License C:

- **License B (reciprocal)**: Generally, the acquired company's usage rights transfer to the acquiring company. However, the definition of "commercial use" (revenue, business scope, etc.) may change, so you should re-evaluate post-acquisition use patterns.
- **License C (subscription)**: Many subscription agreements include "assignment restriction" clauses, which may require **prior consent from the licensor** for assignment in an M&A context. Check the terms of your subscription agreement.

Please consult with AQUAXIS TECHNOLOGY in advance of any M&A.

### Q9-6. I'm trying to patent something related to this software. Is there a problem?

All three licenses -- License A (AGPL), License B, and License C -- contain patent retaliation clauses (LICENSE.md Section 2.3, AGPL Section 11, etc.). If you file a patent infringement lawsuit against this software, that licensee's license automatically terminates.

In other words, you are prohibited from using this software while asserting patent rights over it. This is a standard clause in dual-license OSS projects.

### Q9-7. How should a foreign subsidiary (US, EU, etc.) use this software?

This draft specifies Japanese law as the governing law (LICENSE.md Section 4.7). **Additionally, this license designates Japanese as the authoritative text, and the Japanese text takes precedence even when used outside Japan** (LICENSE.md Section 4.12). If a foreign subsidiary uses this software as an independent legal entity, the following options are available:

1. **Parent company executes a blanket contract**: The parent company enters into a subscription agreement and explicitly covers subsidiaries in the contract scope through an individual agreement
2. **Each subsidiary contracts locally**: Each subsidiary enters into a separate agreement with the licensor (however, understanding the Japanese authoritative text is a prerequisite; an English reference translation can be provided upon request)

AQUAXIS TECHNOLOGY currently primarily serves the Japanese market, but international expansion inquiries are welcome at http://aquaxis.com/sales.

### Q9-8. Is there an English version of the agreement? Which text is authoritative?

This license and related documents (CLA, SUPPORT, SELF-DECLARATION, FAQ) are **written in Japanese as the authoritative version** (LICENSE.md Section 4.12, CLA.md Section I-11(e), SUPPORT.md Section 8.6, SELF-DECLARATION.md introduction).

English and other translations may be provided for the convenience of users, but the following rules apply:

- **Japanese authoritative text always takes precedence**: In the event of any interpretive differences, contradictions, or ambiguities between a translation and the Japanese text, **the Japanese authoritative text shall prevail**
- **User's obligation to understand**: Users who do not read Japanese must **adequately understand the content of the Japanese authoritative text at their own expense and responsibility before agreeing to this license**
- **Same applies for use outside Japan**: These provisions apply equally when the user is located outside Japan and when this software is used outside Japan

**Practical guidance**:
- When foreign subsidiaries or business partners evaluate this license, we recommend review by their internal legal department or an attorney who can read Japanese
- Even if AQUAXIS TECHNOLOGY provides an English reference translation, that translation is not a legally binding document and is positioned as supplementary material to the Japanese authoritative text

### Q9-9. Is there a compliance checklist?

We recommend the following:

**At initial evaluation:**
- [ ] Determine whether use is commercial or non-commercial (LICENSE.md Section 0.1)
- [ ] Check if any exclusion provisions apply (0.2)
- [ ] Determine if small business relief thresholds are met (0.2(d))
- [ ] Decide whether to use the evaluation period relief (0.2(c))

**When selecting License B:**
- [ ] Share the output definition internally (0.6)
- [ ] Decide on the publication license for outputs (2.4.1)
- [ ] Prepare the publication medium (2.4.2)
- [ ] Set a publication schedule within 30 days (2.4.3)
- [ ] Plan for 12-month publication maintenance (2.4.4)
- [ ] Establish a notification process to the licensor (2.4.5)
- [ ] Consider carve-outs for unpublishable information (2.4.6)

**When selecting License C:**
- [ ] Determine the contract unit (standard is per organization; per-developer or per-server if individually agreed)
- [ ] Confirm whether optional support is needed (on-site, extended, English, etc.)
- [ ] Confirm renewal and cancellation notice deadlines
- [ ] Establish internal policies for output handling

**When planning to contribute:**
- [ ] Obtain internal approval for the CLA (ICLA or CCLA)
- [ ] Create a designated contributor list (for CCLA)
- [ ] Integrate into the GitHub workflow (e.g., CLA Assistant)

---

## Contact

For questions not resolved by this FAQ, please contact:

- **General license questions**: contact@aquaxis.com
- **Commercial use and subscription quotes**: sales@aquaxis.com
- **For contributors**: http://aquaxis.com/community
- **For legal and compliance**: legal@aquaxis.com

---

**This FAQ is intended to provide general information about this project's license and does not constitute legal advice. For specific situations, please consult an attorney. In the event of any inconsistency between this FAQ and the terms of `LICENSE.md`, `LICENSE.md` takes precedence.**