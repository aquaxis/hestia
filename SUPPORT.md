# HESTIA Support Terms

Version 1.0.0

Copyright (C) 2026 AQUAXIS TECHNOLOGY. All Rights Reserved.

---

## Preamble

These Support Terms (hereinafter "these Terms") define the technical support services (hereinafter "these Services") provided by AQUAXIS TECHNOLOGY (hereinafter "the Company") to a contractor (hereinafter "the Contractor") who has entered into a license for HESTIA (the commercial subscription license defined in `LICENSE.md` Chapter 3, License C, hereinafter the "Subscription License").

These Terms constitute part of the individual subscription agreement (hereinafter the "Individual Agreement") entered into between the Contractor and the Company. In the event of any inconsistency between these Terms and the Individual Agreement, the Individual Agreement shall prevail.

---

## Chapter 1: Definitions

### 1.1 Definition of Terms

In these Terms, the following terms shall have the following meanings.

(a) "**These Services**" means the technical support services that the Company provides to the Contractor under these Terms.

(b) "**Business Day**" means any day other than Saturdays, Sundays, Japanese national holidays, and the period from December 29 to January 3 of the following year.

(c) "**Business Hours**" means the period from 9:00 AM to 6:00 PM Japan Standard Time (JST, UTC+9).

(d) "**Initial Response**" means a substantive response by the Company to an inquiry from the Contractor that includes acknowledgment of receipt and assignment of a responsible person. Automatic reply emails are not included in the initial response.

(e) "**Workaround**" means an operational or technical measure that temporarily mitigates the impact of a problem, even if it does not resolve it fundamentally.

(f) "**Permanent Fix**" means a modification (patch, version upgrade, configuration change, etc.) that resolves the root cause of a problem.

---

## Chapter 2: Support Content

### 2.1 Support Overview

These Services are provided under the following single plan.

| Item | Details |
|---|---|
| **Support Hours** | Business days during business hours (weekday 9:00-18:00 JST) |
| **Support Channel** | Email only |
| **Initial Response Time** | Within **3 business days** on a business-day basis |
| **Annual Maximum Inquiries** | **4 cases / contract year** |
| **Support Language** | Japanese |

If the annual maximum inquiries are exceeded, enrollment in a separate paid option service (additional support contract) is required. See Section 2.6 for details.

### 2.2 Support Channel

These Services use email as the only support channel.

- **Address**: support@aquaxis.com
- **Subject format**: `[Contractor Name] Brief description of inquiry` (e.g., `[Sample Corporation] API authentication error`)
- **Receipt confirmation**: An automatic receipt confirmation email is sent immediately by the Company's system (automatic replies are not included in the initial response)

Chat, phone, video conferencing, on-site support, Slack, Microsoft Teams, and other support channels are **not included** in these Services.

### 2.3 Excluded Days from Support Hours

The following periods are excluded from support hours:

- Saturdays and Sundays
- Japanese national holidays
- December 29 to January 3 of the following year (year-end/new year closure)
- Other temporary closure days notified by the Company in advance

Inquiries received outside business hours shall have their initial response time calculated starting from the beginning of business hours on the next business day.

### 2.4 Commencement and Calculation of Initial Response Time

The initial response time shall be calculated from the **business day on which the Company receives the inquiry from the Contractor** as the starting date, and the Company shall respond within 3 business days.

### 2.2 Resolution Target Time

No resolution target time is set after the initial response. The time to resolution varies depending on the nature of the problem, the Contractor's cooperation, third-party factors, and other circumstances. The Company will use best efforts to resolve issues, but does not guarantee a resolution timeframe.

### 2.6 Annual Inquiry Count

**2.6.1 Annual Maximum**

The Contractor may submit up to **4 inquiries per contract year** through these Services. The annual maximum is set for each subscription license contract period (typically 1 year).

**2.6.2 Inquiry Counting Method**

One inquiry is counted as follows:

(a) **1 question / 1 issue = 1 case**. Any email exchanges related to that inquiry are treated as the same single case regardless of the number of messages

(b) Follow-up questions and additional questions arising from the same inquiry that are along the same line as the original inquiry are treated as the same case

(c) Questions on a **different topic** from the original inquiry are counted as separate cases

(d) Inquiries determined to fall under the out-of-scope provisions (Section 3.2) are not counted

(e) Reopened cases where reinvestigation is needed after closure due to the Company's circumstances are not counted

**2.6.3 Carryover of Cases**

Unused inquiry cases within a contract year are **not carried over to the next contract year**. Cases are reset at each contract year renewal.

**2.6.4 Notification of Remaining Cases**

The Company will notify the remaining inquiry count upon completion of each inquiry response. The Contractor may also check the remaining count by contacting support@aquaxis.com.

**2.6.5 Exceeding the Annual Maximum**

If the Contractor has used all annual maximum inquiries (4 cases) and wishes to make further inquiries, the following options are available:

(a) **Individual purchase of additional inquiry cases**: Additional inquiry cases may be purchased as a separate paid option service (pricing, unit quantities, etc. are individually quoted)

(b) **Wait until the next contract year**: If not urgent, use the cases that reset at the next contract year renewal

(c) **Refer to public documentation and community channels**: Refer to the Company's public documentation (http://aquaxis.com/faq, http://aquaxis.com/community). However, no response guarantee from the licensor is provided on these channels

**2.6.6 Applying for Additional Support**

To apply for additional inquiry cases or other paid support (Section 3.3), contact support@aquaxis.com. After receipt, the Company will send an individual quote.

---

## Chapter 3: Support Scope

### 3.1 In-Scope Support

These Services cover the following:

(a) The **current major version** and its latest minor version of this software

(b) Answers to technical questions about this software

(c) Investigation of bug reports, providing workarounds, and providing fix patches for this software

(d) Assistance with configuration and installation of this software

(e) Questions about release notes and documentation for this software

### 3.2 Out-of-Scope Support

The following are out of scope for these Services:

(a) Issues related to use of this software versions other than those in scope (two or more major versions prior)

(b) Issues caused by modifications made by the Contractor to this software (however, the Company will confirm reproduction after reverting the modified portion)

(c) Issues attributable to the Contractor's applications, infrastructure, network, or third-party software, where there is no problem with this software itself

(d) Malfunctions of third-party plugins or extensions

(e) Design of the Contractor's operational structure or internal processes

(f) Operational training for the Contractor's employees or subcontractors

(g) Custom development for integrating this software with other software

(h) General technical consultations unrelated to this software

(i) Support in languages other than Japanese

### 3.3 Support Scope Extension

The Contractor may add the following through changes to the Individual Agreement or a separate paid service agreement. These are not included in the standard scope of these Services.

- Extended Support for prior versions
- Paid training
- Paid consulting
- Custom development

---

## Chapter 4: Support Usage Flow

### 4.1 Inquiry Submission

The Contractor shall submit inquiries with the following information:

(a) Contractor name and contact person name

(b) Environment information (this software version, OS, runtime environment, related software)

(c) Problem details (reproduction steps, expected behavior, actual behavior, error messages, logs)

(d) Remediation attempts and results already tried

(e) Impact scope (production/staging/development, number of affected users, etc.)

### 4.2 Initial Response

The Company shall, within the initial response time defined in Section 2.1 (within 3 business days):

(a) Acknowledge receipt of the inquiry

(b) Assign a responsible person

(c) Request additional information if needed

### 4.3 Investigation and Response

Depending on the inquiry content, the Company shall perform one or more of the following:

(a) Answer questions

(b) Investigate the cause of the problem

(c) Provide workarounds

(d) Provide permanent fixes (patches, version upgrades)

(e) Direct to documentation and knowledge base

### 4.4 Resolution Confirmation

The Company may close an inquiry when it receives confirmation of resolution from the Contractor, or when the Contractor has not responded for **10 business days** or more. If the same issue recurs after closure, the Contractor shall open a new inquiry.

---

## Chapter 5: Contract Period, Renewal, and Termination

### 5.1 Application Period

These Terms remain in effect during the Contractor's subscription license contract period.

### 5.2 Renewal

If the subscription license is renewed, these Terms are also renewed. The latest version at the time of renewal shall apply.

### 5.3 Termination

Upon termination of the subscription license, the provision of these Services under these Terms also terminates.

### 5.4 Post-Termination Data

Support-related data provided by the Contractor (logs, configuration information, email history, etc.) shall be retained for **3 years** after contract termination in accordance with the Company's data retention policy, and then deleted. If the Contractor wishes early deletion, the Contractor shall submit a written request to the Company.

---

## Chapter 6: Contractor's Cooperation Obligations

### 6.1 Information Provision Obligation

The Contractor shall, without delay and in response to the Company's reasonable requests, provide information necessary for the provision of these Services (version information, logs, reproduction steps, configuration information, etc.).

### 6.2 Handling of Confidential Information

If information provided by the Contractor to the Company contains trade secrets or personal information, the Contractor shall notify the Company in advance and, if necessary, enter into a separate non-disclosure agreement. It is recommended to anonymize or mask logs and data before providing them whenever possible.

### 6.3 Disclaimer for Delays Due to Insufficient Cooperation

If the provision of these Services is delayed because the Contractor has not provided necessary information, the Company shall not be responsible for exceeding the response time specified in Section 2.1.

---

## Chapter 7: Company's Obligations and Disclaimers

### 7.1 Company's Obligations

The Company shall provide these Services with the due care of a prudent manager in accordance with these Terms.

### 7.2 Disclaimers

The Company shall not be responsible for the following in providing these Services:

(a) The results of the Contractor's business activities or business continuity

(b) Loss or damage to the Contractor's data (backup by the Contractor is assumed)

(c) Operation or interoperability of third-party products

(d) Delays due to force majeure (large-scale disasters, war, pandemics, government regulations, communication infrastructure failures, etc.)

(e) Guarantee that these Services are suitable for the Contractor's specific purposes

### 7.3 Limitation of Liability

The Company's liability for these Services shall be limited, as specified in `LICENSE.md` Section 4.2 and the Individual Agreement, to the total amount of subscription fees paid by the Contractor to the Company during the 12 months preceding the occurrence of the damage, except in cases of willful misconduct or gross negligence.

---

## Chapter 8: General Provisions

### 8.1 Changes to Terms

The Company may change these Terms. Changes shall take effect **30 days** after email notification to the Contractor. If the Contractor does not agree to the changes, the Contractor may choose not to renew at the end of the contract period.

Material changes that are disadvantageous to the Contractor shall not be applied during the current contract period and shall take effect from the next contract renewal.

### 8.2 Notices

Notices under these Terms shall be given as follows:

- From the Company to the Contractor: Contractor's registered email address
- From the Contractor to the Company: support@aquaxis.com

### 8.3 Governing Law and Jurisdiction

These Terms, like `LICENSE.md` Section 4.7, shall be governed by Japanese law, and the Tokyo District Court shall be the exclusive agreed jurisdiction court of first instance.

### 8.4 Severability

If any provision of these Terms is held invalid, the remaining provisions shall continue in effect.

### 8.5 Order of Precedence

The order of precedence for documents related to these Terms shall be as follows:

1. Individual Agreement (subscription agreement)
2. `LICENSE.md` (especially Chapter 3 License C and Chapter 4 Common Provisions)
3. These Terms

### 8.6 Language and Authoritative Text

These Terms are **written in Japanese as the authoritative text**. Even if a translated version (English or other languages) of these Terms is produced, the translation is provided for reference only, and in the event of any interpretive differences or contradictions, **the Japanese authoritative text shall prevail**. This section applies equally when the Contractor is located outside Japan and when these Services are provided outside Japan.

This provision is intended to be consistent with `LICENSE.md` Section 4.12 (Language and Authoritative Text).

---

## Appendix: Inquiry Template

```
Subject: [Contractor Name] Brief description of inquiry

[Basic Information]
- Contractor Name       :
- Contact Person Name  :
- HESTIA Version       :
- Environment          : Production / Staging / Development / Test

[Problem/Question Details]
- Description          :
- Expected Behavior    :
- Actual Behavior      :
- Reproduction Steps   :

[Attachments]
- Logs: (attach if possible)
- Configuration files: (if applicable)
- Screenshots: (if applicable)

[Desired Support]
-
```