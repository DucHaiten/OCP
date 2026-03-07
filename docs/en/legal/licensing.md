# Licensing OCP-OCL v1.0

This document is for readers who want quick, practical answers to questions such as:
- Can I use OCP-OCL through the open-source route?
- When do I need to discuss a commercial path?
- If I contribute to the project, how does that affect later usage and release rights?

This is an explanation document for users to read.  
It does not replace the `LICENSE` file, a separate commercial agreement, or legal advice.

## Short Summary

If you only need the quick conclusion, these are the three most important points:

1. The open-source side of OCP-OCL uses `AGPL-3.0-only`.
2. The project follows a `dual-license` model, which means there may be a separate commercial path in addition to the open-source path.
3. If you contribute to the project, you must go through the CLA flow required by current policy.

## 1) What license does OCP-OCL use?

The official open-source license of OCP-OCL is:
- `AGPL-3.0-only`

This point needs to be understood clearly:
- OCP-OCL does not use a custom-made license instead of AGPL.
- If you see descriptions such as `GGPL`, `governed GPL`, or similar wording elsewhere, treat them as shorthand for the project's philosophy or governance style, not as a legal license name that replaces `AGPL-3.0-only`.

If there is any difference between an explanation and the original license text, the `LICENSE` file at the root of the repo is the primary legal source for the open-source path.

## 2) What does “dual-license” mean here?

`Dual-license` does not mean you can mix two licenses however you want.

It means the project has two distinct licensing paths:

### 2.1) Open-source path
You use OCP-OCL under `AGPL-3.0-only`.

### 2.2) Commercial path
In some situations, the project may provide a separate commercial license or separate commercial terms outside the open-source path.

In plain language:
- if you use the OSS path, you must comply with `AGPL-3.0-only`;
- if you need different terms, you must not assume them on your own and instead must go through the separate commercial path.

## 3) When can I use the open-source path?

In general, you can consider the OSS path if:
- you accept using OCP-OCL under `AGPL-3.0-only`;
- you accept the obligations that come with that license;
- you do not need a separate legal exception;
- you do not need separately committed support, SLA, warranty, or indemnity terms.

Common situations that often fit the OSS path:
- learning, research, and technical evaluation;
- internal evaluation before deciding on broader use;
- use in an open-source project that is compatible with the license;
- contributing back to the ecosystem through the project's workflow.

Important point:
- the project does not automatically grant exceptions just because you are an individual, a small team, a startup, or “only trying it out”.
- if you plan to use OCP-OCL through the OSS path in an important context, you should read the `LICENSE` file carefully yourself.

## 4) When should I move to the commercial path?

You should discuss the commercial path if one or more of the following applies:
- you want to use or distribute OCP-OCL in a proprietary model;
- you need separate legal terms for an organization or company;
- you need committed support, SLA, warranty, indemnity, or procurement paperwork;
- you need a clearly documented licensing position for legal, audit, or internal procurement review;
- you need an exception that does not exist in the OSS path.

The detailed explanation for that path is in:
- `docs/en/legal/commercial.md`

If that file is not yet complete in a given snapshot, the safe reading is:
- the commercial path is not yet fully described at the public-doc level;
- you should not infer anything beyond what has already been stated explicitly.

## 5) What is the project not claiming right now?

To avoid misunderstanding, you should not interpret OCP-OCL in any of the following ways unless a separate policy says so clearly:
- “free for small users, mandatory payment for large users”;
- “internal use always means you do not need to care about the license”;
- “every organization must buy a commercial license”;
- “every enterprise use case is forbidden unless money has already been paid”.

Statements like these only have value when they are written explicitly in legal text or in a separate commercial policy.

## 6) What if I want to contribute to the project?

Under current policy:
- a CLA is required;
- the locked CLA type is `license_grant`;
- the CLA scope applies to:
  - code,
  - docs,
  - contracts.

In practical terms, this is kept in place so that the project can:
- continue releasing the open-source side consistently;
- while also retaining the ability to operate a lawful dual-license model.

In short:
- if a contribution does not go through the proper CLA flow, it does not fit the licensing model that the project has locked.

If you want to contribute, read:
- `CONTRIBUTING.md`

## 7) Which file should I read if I need a firmer answer?

Each file serves a different purpose:

### `LICENSE`
This is the original OSS license text.  
If you need to know the legal obligations of the open-source path, this is the most important file.

### `docs/en/legal/licensing.md`
This is the file you are reading now.  
It helps you understand the project's overall licensing model in more readable language.

### `docs/en/legal/commercial.md`
This file explains the commercial path, the kinds of situations that need commercial terms, and the safe way to understand that path.

### `docs/en/legal/governance.md`
This file explains who has authority to decide large changes around release, policy, governance, and licensing direction.

### `CONTRIBUTING.md`
This file explains how to contribute to the project.  
It does not replace the license or the CLA, but it matters if you plan to submit contributions.

## 8) Limits of this document

This document is written to:
- reduce misunderstanding;
- help users and organizations make an initial decision faster;
- avoid arguments like “I thought the project used one license, but it actually used another”.

But this file is not:
- a commercial agreement;
- a replacement for `LICENSE`;
- legal advice.

If you are making a decision with serious legal or commercial consequences, you should:
1. read `LICENSE`,
2. read the commercial document when it is complete,
3. and involve your own legal team if needed.

## 9) Conclusion

The simplest safe reading is:
- OCP-OCL has an open-source path under `AGPL-3.0-only`;
- the project keeps a `dual-license` model;
- contribution through the CLA flow is a mandatory policy requirement;
- if you need terms that differ from the OSS path, you must go through the separate commercial path and must not infer those terms on your own.
