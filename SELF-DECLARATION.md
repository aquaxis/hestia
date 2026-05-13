# HESTIA Small Business Relief Self-Declaration System Operations Manual

Version 1.0.0

Copyright (C) 2026 AQUAXIS TECHNOLOGY. All Rights Reserved.

This document is the operations manual for the self-declaration system for the small business relief defined in `LICENSE.md` Chapter 0, Section 0.2(d). **In the event of any inconsistency between this document and the provisions of `LICENSE.md`, `LICENSE.md` shall prevail.**

**This document is written in Japanese as the authoritative text.** Even if a translation (English or other languages) of this document is produced, the translation is provided for reference only, and in the event of any interpretive differences or contradictions, the Japanese authoritative text shall prevail. This provision applies equally when the declarant is located outside Japan. Declarants who do not read Japanese are obligated to adequately understand the content of the Japanese authoritative text at their own expense and responsibility before making a declaration under this system. This provision is intended to be consistent with `LICENSE.md` Section 4.12 (Language and Authoritative Text).

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Declarant Guide](#2-declarant-guide)
3. [Declaration Processing Flow](#3-declaration-processing-flow)
4. [Annual Renewal and Threshold Excess Notification](#4-annual-renewal-and-threshold-excess-notification)
5. [Handling of False Declarations and Retroactive Invalidity](#5-handling-of-false-declarations-and-retroactive-invalidity)
6. [Audit Rights](#6-audit-rights)
7. [Personal Information and Data Protection](#7-personal-information-and-data-protection)
8. [Internal Operations Manual](#8-internal-operations-manual)
9. [Notification and Communication Templates](#9-notification-and-communication-templates)

---

## 1. System Overview

### 1.1 Purpose

This system is a relief measure that allows sole proprietors, small startups, and other small businesses that formally meet the criteria for commercial use but for whom the financial burden of a paid license (License C) would be disproportionate, to use this Software under License A (AGPL-3.0).

### 1.2 Eligibility Requirements

All of the following requirements defined in `LICENSE.md` Chapter 0, Section 0.2(d) must be met:

(a) The user's (or, if a corporation, the entire corporate group) annual gross revenue for the most recent fiscal year is below **10,000,000 JPY (for Japan-based entities) or 50,000 USD (for entities outside Japan)**, based on location

(b) Completion of the self-declaration procedure defined in this operations manual and receipt of an acknowledgment notification from the Licensor

(c) Fulfillment of the notification obligation if any changes occur to the declared information during the relief period

### 1.3 Scope of Application

Users granted this relief may use this Software under **License A (AGPL-3.0) conditions** even for commercial use during the relief period. The output publication obligation (License B, Section 2.4) and commercial subscription fees (License C) do not apply.

However, AGPL-3.0's own obligations (publication of modifications, source code publication upon network interaction, etc.) continue to apply. If you wish to avoid these obligations, you must choose License C subscription instead of this system.

### 1.4 Relief Period

As a general rule, **one fiscal year** (from the date of declaration receipt to the end of the declarant's next fiscal year) constitutes one relief period. Annual renewal declarations are required for continued use.

---

## 2. Declarant Guide

### 2.1 How to Declare

Declarations are submitted at http://aquaxis.com/declaration. See `self-declaration-form.html` for the form.

### 2.2 Information Required for Declaration

Declarants must prepare the following information:

- Organization/basic information (name, location, representative, contact person)
- Most recent fiscal year period (start date, end date)
- Annual gross revenue (excluding tax, whole number)
- Whether affiliated with a corporate group, and if so, the group's aggregated revenue
- Intended use and purpose of this Software
- Agreement to the declaration terms

### 2.3 Supporting Documentation Required for Declaration

**No documentation needs to be attached at the time of declaration.** However, if the Licensor subsequently requests documentation, the following must be prepared:

- Financial statements (income statement) for the most recent fiscal year
- Corporate group structure chart (if applicable)
- Commercial registry certificate or equivalent public document (for initial declarations)

These documents will be used for verification under Section 6 (Audit Rights).

### 2.4 Declaration Flow

```
 1. Declarant fills in the self-declaration form
         |
 2. Form submission
         |
 3. Automatic receipt confirmation email (to declarant)
         |
 4. Licensor reviews content (typically within 5 business days)
         |
 5. Review result notification
     |-- Approved: Declaration number and relief period notification
     +-- Needs confirmation: Request for additional information
         |
 6. After approval, declarant may use the software under License A during the relief period
         |
 7. Renewal reminder 30 days before relief period end (from Licensor)
         |
 8. Annual renewal declaration OR relief period expiration
```

### 2.5 Declaration Number

Each approved declaration is assigned a unique **declaration number** (e.g., `SBD-2026-00001`) by the Licensor. This number is used for:

- Reference during annual renewal declarations
- Reference during threshold excess notifications
- Reference during audit responses
- Reference for general inquiries to the Licensor

The declaration number is included in the approval notification email. **Please keep it safe and do not lose it.**

---

## 3. Declaration Processing Flow

### 3.1 Standard Processing Timeline from Receipt to Approval

- **Receipt confirmation**: Immediately after submission (automatic email)
- **Initial review**: Within 3 business days of receipt
- **Approval notification**: Within 5 business days of receipt (if no additional confirmation needed)
- **If additional confirmation is needed**: Up to 15 business days

### 3.2 Review Content

The Licensor reviews the declared information from the following perspectives:

(a) **Formal review**: Missing required fields, obvious inconsistencies, format violations

(b) **Substantive review**: Revenue threshold compliance, corporate group structure validity, legality of intended use

(c) **Cross-reference with public information**: Commercial registry, public financial statements (for listed companies), official websites, and other public information

(d) **Consistency with past declarations**: Continuity with past declaration content, presence of unusual fluctuations

### 3.3 Review Result Categories

Review results are notified in one of three categories:

| Category | Content | Action |
|---|---|---|
| **Approved** | No issues with declaration content | Relief period begins |
| **Conditionally Approved** | Minor concerns | Approved after additional information is provided |
| **Rejected** | Eligibility not met or significant concerns | No relief; select License B or C |

### 3.4 Common Reasons for Rejection

A declaration will be rejected if any of the following apply:

- Revenue clearly exceeds the threshold
- Intentional exclusion of corporate group entities is suspected
- False or materially incorrect information in the declaration
- Past history of false declarations under this system
- Intended use may violate laws or public order and morals

### 3.5 Actions After Rejection

After receiving a rejection notice, the declarant may choose one of the following:

(a) Re-declare after providing additional information

(b) Select License B (reciprocal, free, with output publication obligation)

(c) Select License C (commercial subscription, paid)

(d) Stop using this Software

### 3.6 Appeals

Declarants may appeal a rejection decision in writing. Appeals must be submitted to contact@aquaxis.com within 14 days of receiving the rejection notice. The Licensor will re-review the appeal within 15 business days of receipt and notify the declarant of the result.

---

## 4. Annual Renewal and Threshold Excess Notification

### 4.1 Annual Renewal

Since the relief period is one fiscal year, annual renewal declarations are required for continued use.

**Renewal Declaration Timing:**

- A renewal reminder email is sent **60 days before** the relief period expiration date
- Declarants must complete the renewal declaration **30 days before** the expiration date
- If the deadline passes, License A coverage ends upon relief period expiration

**Items to Confirm at Renewal:**

- Most recent fiscal year revenue (re-confirmation that it is within the threshold)
- Any changes in corporate group structure
- Any changes in intended use
- Any changes in company information

### 4.2 Threshold Excess Notification Obligation

If it becomes certain during the relief period that annual gross revenue will exceed **10,000,000 JPY (for Japan-based entities) or 50,000 USD (for entities outside Japan)** based on location (e.g., upon finalization of financial statements), the declarant must notify the Licensor in writing (email acceptable) **within 30 days**.

**Information to Include in the Notification:**

- Declaration number
- Date the excess became certain (e.g., financial statement finalization date)
- Revenue after the excess
- Cause of the excess (temporary, structural, presence of special factors such as M&A)
- Future plans for using this Software (transition to License B / transition to License C / cessation of use)

Failure to notify may result in the application of Section 5 (Handling of False Declarations).

### 4.3 Transition Period After Threshold Excess

Upon receiving notification of a threshold excess, the Licensor will grant the declarant a **30-day** transition period (which may be extended by mutual agreement). During the transition period, the declarant must choose one of the following and complete the transition:

(a) Transition to License B, with output publication obligations for commercial use after the transition date

(b) Enter into a License C subscription agreement

(c) Stop using this Software

### 4.4 Other Change Notification Obligations

The following changes also require notification within 30 days:

- **Corporate merger, division, or M&A**: Change in the licensee entity
- **Joining a corporate group**: Where a previously independent entity now has a parent company
- **Material change in intended use**: When using the software for purposes significantly different from the declaration
- **Change in company information**: Name, location, representative, contact person

---

## 5. Handling of False Declarations and Retroactive Invalidity

### 5.1 Definition of False Declaration

A "false declaration" is found when any of the following apply:

(a) Intentionally stating facts that differ from the truth regarding objective facts such as revenue or corporate group information

(b) Intentionally omitting material facts known at the time of declaration (revenue projections, planned M&A, etc.)

(c) Intentionally failing to fulfill the notification obligation upon exceeding the threshold

(d) Unreasonably narrowing the scope of the corporate group and intentionally excluding certain entities

(e) Any other intentional distortion of information to the declarant's advantage

### 5.2 How False Declarations Are Discovered

False declarations may be discovered through:

- Discrepancies between supporting documents and declared information during audits
- Cross-referencing with public information (financial announcements, listed company information, news reports, etc.)
- Reports from third parties
- Post-hoc correction declarations from the declarant themselves
- Discrepancy between the licensee's actual commercial use (e.g., large-scale SaaS offering) and their declaration

### 5.3 Measures Upon Confirmation of False Declaration

When the Licensor has reason to suspect a false declaration, the following steps are taken in order:

**Step 1: Inquiry**

- Notify the declarant in writing (email) of the concerns
- Request a response within 14 days

**Step 2: Consultation**

- If concerns are not resolved based on the declarant's response, conduct a meeting or conference
- Discuss the possibility of correction, the circumstances, and future action plans

**Step 3: Determination**

- If a false declaration is confirmed after consultation, the following measures are taken

### 5.4 Effects of Retroactive Invalidity

When a false declaration is confirmed, the following measures are taken:

(a) **Retroactive License Invalidation**: License A coverage is retroactively invalidated from the date of the false declaration. Use from that point onward must comply with License B or License C conditions.

(b) **Retroactive Application of License B**: If the declarant selects License B, outputs generated during the retroactive period are subject to publication obligations.

(c) **License C Equivalent Fee Claim**: If the declarant selects License C, subscription fees for the retroactive period, calculated from the retroactive application start date, will be charged.

(d) **Restriction on Re-Declaration**: The false declarant is prohibited from making declarations under this system for **3 years**.

(e) **Injunctive Relief**: The Licensor reserves the right to seek injunctive relief and damages as necessary.

### 5.5 Distinction Between Negligent Errors and False Declarations

Unintentional minor errors (calculation mistakes, data entry errors, etc.) are distinguished from false declarations. If the declarant promptly notifies upon discovering the error, the following applies:

- **Voluntary correction**: Corrected through a correction declaration, with no penalty
- **No retroactive invalidity**: As long as the correction is made within a reasonable period, past license validity is maintained

However, intentional false declarations disguised as "negligent errors" will be treated strictly.

---

## 6. Audit Rights

### 6.1 Basis for Audit Rights

As part of the self-declaration conditions in `LICENSE.md` Chapter 0, Section 0.2(d), the Licensor has the right to conduct audits within a reasonable scope to verify the validity of declared information.

### 6.2 Audit Scope

Audits are conducted from the following perspectives:

- Revenue validity (consistency with financial statements and sales ledgers)
- Corporate group structure validity (consistency with registry information and organizational charts)
- Intended use validity (consistency with actual usage)
- Notification obligation fulfillment (threshold excess notifications, change notifications)

### 6.3 Audit Frequency and Method

**Regular Audits:**

- Target: Randomly selected declarants (5% or fewer of annual declarations)
- Frequency: Approximately once per year
- Method: Written request for document submission

**Specific Audits:**

- Target: Declarants for whom suspicion of false declaration has arisen
- Frequency: As needed
- Method: Written investigation, on-site interviews if necessary

### 6.4 Audit Cooperation Obligation

Declarants are obligated to provide the following documents in response to reasonable audit requests from the Licensor:

(a) Financial statements (income statement and relevant portions of the balance sheet) for the most recent fiscal year and the audit period

(b) Documents demonstrating corporate group structure (list of group companies, capital relationship diagrams)

(c) Documents demonstrating actual use of this Software (usage logs, relevant portions of design documents)

(d) Commercial registry certificate or equivalent public document (latest version)

Audit cooperation shall in principle be provided within **30 days** of receiving notification.

### 6.5 Confidentiality

The Licensor shall not use the declarant's confidential information obtained during audits for any purpose other than the audit. The Licensor shall impose confidentiality obligations on employees who handle audit information.

### 6.6 Audit Cost Allocation

- **When audit requirements are met**: The Licensor bears the audit costs
- **When a false declaration is confirmed**: The declarant bears the reasonable costs incurred by the audit

---

## 7. Personal Information and Data Protection

### 7.1 Personal Information Collected

The following personal information is collected in the operation of this system:

- Representative name and title
- Contact person name and email address
- Declarant (electronic signatory) name and title
- Company information (when personal information is included in corporate information)

### 7.2 Purpose of Use

Collected personal information is used only for the following purposes:

- Review and approval of declarations
- Notification of review results
- Annual renewal reminders
- Threshold excess and change notification responses
- Conducting audits
- Compliance with legal disclosure obligations

### 7.3 Third-Party Disclosure

Collected personal information is not disclosed to third parties except in the following cases:

- When the declarant has given consent
- When disclosure is legally required
- When disclosed to professionals (attorneys, accountants, etc.) who have signed non-disclosure agreements for audit assistance

### 7.4 Retention Period

- **Approved declaration information**: **5 years** from the end of the relief period
- **Rejected declaration information**: **3 years** from the rejection notice
- **False declaration-related information**: **7 years** from determination (for litigation purposes)

After the retention period expires, personal information shall be promptly deleted.

### 7.5 Disclosure, Correction, and Deletion Requests

Declarants may request disclosure, correction, or deletion of their personal information. However, the following restrictions apply:

- During the retention period necessary for audits or litigation, deletion requests may not be accommodated
- Deletion of declaration numbers and declaration content is not permitted during the relief period

Requests should be sent to privacy@aquaxis.com.

### 7.6 GDPR and Other Foreign Laws

When receiving declarations from outside Japan (EU, UK, US state of California, etc.), the following measures may be required:

- GDPR: Notification to the DPO (Data Protection Officer), legalization of personal data transfers outside the EU
- CCPA: Prohibition of personal information sales, response to deletion requests

Since declarations from outside Japan are not currently a primary anticipated use case, the policy is to respond individually as needed in operations.

---

## 8. Internal Operations Manual

**Note: This section is an operations guide for internal Licensor personnel only.**

### 8.1 Role Assignments

| Role | Responsible | Key Duties |
|---|---|---|
| Declaration Reception | [Department] | Initial reception, formal review, pointing out deficiencies |
| Review | [Department] | Substantive review, cross-reference with public information, approval decisions |
| Notification | [Department] | Approval/rejection/additional confirmation notifications, annual renewal reminders |
| Audit | [Department/Outsourced] | Audit planning and execution, false declaration investigation |
| Legal | [Department/External Counsel] | Legal response upon false declaration determination, appeal handling |

### 8.2 Checklist from Reception to Notification

**Reception (Target: Same Day)**
- [ ] Confirm form submission
- [ ] Confirm required fields are filled
- [ ] Confirm automatic receipt confirmation email was sent

**Formal Review (Target: Within 3 Business Days)**
- [ ] Check for missing fields or inconsistencies
- [ ] Check for obvious falsifications (format violations, anomalous values)
- [ ] Check past declaration history

**Substantive Review (Target: Within 5 Business Days)**
- [ ] Confirm revenue is within threshold
- [ ] Cross-reference corporate group structure with public information
- [ ] Confirm validity of intended use
- [ ] Check for past false declaration history

**Approval Processing (Target: Within 5 Business Days)**
- [ ] Issue declaration number
- [ ] Set relief period (typically until the end of the declarant's fiscal year)
- [ ] Send approval notification email
- [ ] Register in internal database

### 8.3 Data Management

Declaration information is registered in the internal database with the following structure:

**Required Fields:**
- Declaration number (primary key)
- Organization name
- Representative name, contact person name, contact information
- Most recent fiscal year, annual gross revenue
- Corporate group information
- Declaration date, approval date, relief period expiration date
- Next renewal date
- Status (approved / conditional / rejected / invalidated)

**Access Control:**
- Read: Declaration reception, review, notification, and audit staff
- Update: Review, notification, and audit staff
- Delete: Admin privileges only

### 8.4 Periodic Reviews

The following periodic reviews shall be conducted:

- **Monthly**: Number of declarations, approval rate, and trend analysis of rejection reasons from the previous month
- **Quarterly**: Review of system operations and improvement proposals
- **Annually**: Review of the system itself (thresholds, relief periods, procedures)

### 8.5 Anomaly Detection Indicators

The following anomalies, if detected, should trigger individual investigation:

- Multiple declarations from the same IP address
- Multiple organizations declaring from the same contact information (email, phone)
- Unnatural decrease in revenue (compared to past declarations)
- Unnatural changes in corporate group information
- Inconsistency between contact information and company location

### 8.6 Recommended Automation Scope

For efficient operations, the following automation is recommended:

| Item | Automation Feasibility |
|---|---|
| Form receipt and initial receipt confirmation email | **Recommended for automation** |
| Formal review (required field check) | **Recommended for automation** |
| Cross-reference with public information (National Tax Agency corporate number publication site, etc.) | Semi-automatable |
| Substantive review (financial statement verification, etc.) | Manual |
| Approval notification and declaration number issuance | **Recommended for automation** (after approval) |
| Annual renewal reminder sending | **Must be automated** (reminder) |
| Random selection of audit targets | **Recommended for automation** |

---

## 9. Notification and Communication Templates

### 9.1 Receipt Confirmation Email (Automatic)

```
Subject: [HESTIA] Small Business Relief Self-Declaration Received

Dear [Declarant Name],

Thank you for submitting your small business relief self-declaration for HESTIA.
We have received your declaration as follows:

- Receipt Date and Time: YYYY-MM-DD HH:MM JST
- Declarant: [Organization Name / Individual Name]
- Contact Person: [Contact Person Name]

We will review the content and notify you of the result within 5 business days.
We may request additional information during the review period.

─────────────────────────────────────────────
AQUAXIS TECHNOLOGY
contact@aquaxis.com
```

### 9.2 Approval Notification Email

```
Subject: [HESTIA] Small Business Relief Declaration Approved

Dear [Contact Person Name],

Thank you for submitting your small business relief self-declaration for HESTIA.

We have reviewed your declaration and are pleased to inform you that it has been approved.

- Declaration Number: SBD-YYYY-NNNNN
- Approval Date: YYYY-MM-DD
- Relief Period: YYYY-MM-DD to YYYY-MM-DD
- Applicable License: License A (AGPL-3.0)

During the relief period, you may use this Software under the terms of License A (AGPL-3.0).
Please note that AGPL-3.0's own obligations (publication of modifications, source code
publication upon network interaction, etc.) continue to apply.

Important reminders:
- If your annual gross revenue exceeds the threshold during the relief period, you must
  notify us within 30 days
- We will send an annual renewal reminder 60 days before the relief period expiration date
- Please keep your declaration number safe, as you will need it for future inquiries
  and renewals

─────────────────────────────────────────────
AQUAXIS TECHNOLOGY
contact@aquaxis.com
```

### 9.3 Additional Information Request Email

```
Subject: [HESTIA] Small Business Relief Declaration Content Confirmation

Dear [Contact Person Name],

Regarding the small business relief self-declaration you submitted, we would like
to request additional information on the following points.

- Declaration Number: SBD-YYYY-NNNNN
- Items to Confirm:
  1. [Specific item to confirm]
  2. [Specific item to confirm]

- Response Deadline: YYYY-MM-DD (within 2 weeks)

We apologize for the inconvenience and ask that you respond by the above deadline
at http://aquaxis.com/contact or by replying to this email. If we do not receive
a response by the deadline, your declaration will be placed on hold.

─────────────────────────────────────────────
AQUAXIS TECHNOLOGY
contact@aquaxis.com
```

### 9.4 Rejection Notification Email

```
Subject: [HESTIA] Small Business Relief Declaration Result

Dear [Contact Person Name],

We have carefully reviewed the small business relief self-declaration you submitted.
However, we regret to inform you that we are unable to approve the application of
this system for the following reason(s).

- Declaration Number (provisional): SBD-YYYY-NNNNN
- Reason for Rejection: [Specific reason for rejection]

If you wish to use this Software commercially, please select one of the following:

(1) License B (Reciprocal, Free)
    Output publication obligation applies. For details, see LICENSE.md Chapter 2.

(2) License C (Commercial Subscription)
    Paid. No output publication obligation, with priority support.
    For details, contact http://aquaxis.com/sales.

If you have any questions about this result, please contact contact@aquaxis.com
within 14 days of receiving this notice. We will guide you through the appeal process.

─────────────────────────────────────────────
AQUAXIS TECHNOLOGY
contact@aquaxis.com
```

### 9.5 Annual Renewal Reminder Email (Automatic)

```
Subject: [HESTIA] Small Business Relief Period Expiration Notice

Dear [Contact Person Name],

Thank you for your continued use of HESTIA.

The expiration date of your registered small business relief period is approaching.

- Declaration Number: SBD-YYYY-NNNNN
- Relief Period Expiration Date: YYYY-MM-DD

If you wish to continue the relief period, please complete the annual renewal
declaration by 30 days before the expiration date.

- Renewal Declaration URL: http://aquaxis.com/declaration/renewal

If you do not submit a renewal declaration, License A coverage will expire on
the expiration date. You will then need to choose one of the following:

(1) License B (Reciprocal, Free, Output Publication)
(2) License C (Commercial Subscription, Paid)
(3) Stop using this Software

If you have any questions, please contact contact@aquaxis.com.

─────────────────────────────────────────────
AQUAXIS TECHNOLOGY
contact@aquaxis.com
```

### 9.6 Threshold Excess Confirmation Email (Licensor to Declarant)

```
Subject: [HESTIA] Threshold Excess Confirmation Request

Dear [Contact Person Name],

You are registered under the HESTIA small business relief (Declaration Number:
SBD-YYYY-NNNNN). Based on [source information], we have determined that your
annual gross revenue may exceed the threshold (10,000,000 JPY for Japan-based
entities or 50,000 USD for entities outside Japan).

We would like to confirm the following:

1. The accurate amount of your annual gross revenue for the most recent fiscal year
2. The date the excess became certain, if applicable
3. Your future plans for using this Software

Please respond within 14 days of receiving this email to contact@aquaxis.com.
If we do not receive a response, or if a false declaration is confirmed, the
relief may be retroactively invalidated. We appreciate your honest cooperation.

─────────────────────────────────────────────
AQUAXIS TECHNOLOGY
contact@aquaxis.com
```