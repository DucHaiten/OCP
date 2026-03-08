# Governance OCP v1.0

This document explains how OCP is governed at the public-facing level:
- who has decision authority;
- which changes require tight control;
- how users and contributors should understand the project's authority model.

This file is written so readers can understand the governance model of the project.  
It does not replace the root source files such as `GOVERNANCE.md`, `CODEOWNERS`, or the rules already locked in the plan and contracts.

## Short Summary

If you only need the quick version:

1. OCP is currently coordinated as an owner-led project.
2. Not every contributor has authority over policy, release direction, or licensing direction.
3. Large changes must go through plan gates, tests, and evidence; they are not decided by intuition alone.
4. “Official build” status and official release authority must be tied to the trust chain; a fork does not get that status automatically.

## 1) What is governance for in OCP?

Governance here is not abstract theory.

It exists to answer practical questions such as:
- who can finalize a release;
- who can change policy;
- who can change licensing or commercial direction;
- how contributor changes are handled;
- what counts as an official release.

Without clear governance, a project like this quickly runs into problems such as:
- disputes about who has decision authority;
- confusion between contribution and final ownership of strategic decisions;
- confusion between fork/community builds and official builds;
- difficulty keeping docs, legal policy, release policy, and trust chain aligned.

## 2) How should the current OCP model be understood?

Based on the current repo state and the active plan, OCP is currently an:
- owner-led project;
- with tightly held release and policy direction;
- with contributions accepted through a clear workflow, but without contributions automatically carrying strategic decision power.

In simpler terms:
- the project can accept contributions,
- but final decisions on release, legal direction, governance, and commercial direction are not automatically shared equally across everyone who submits a PR.

## 3) Who has authority over major decisions?

At the public-facing level, readers should understand that:
- the owner or core release authority of the project has the power to finalize:
  - official releases,
  - licensing policy,
  - commercial direction,
  - governance changes,
  - trust-root and official build direction.

This does not mean contributors are unimportant.

It means:
- technical contributions and community feedback are inputs;
- final decisions on strategic and legal issues require clear owner authority.

## 4) What rights do contributors have, and what rights do they not have?

### 4.1) What contributors can do

Contributors can:
- propose changes to code, docs, and contracts;
- report bugs and propose improvements;
- discuss design, UX, docs, or workflow;
- help improve project quality.

### 4.2) What contributors do not automatically have

Contributors do not automatically have the right to:
- finalize an official release;
- change the licensing model;
- change commercial policy;
- label a build as official;
- alter the governance direction without going through the owner-controlled process.

In short:
- sending a contribution is not the same thing as gaining authority to run the project.

## 5) Which changes must be tightly controlled?

The following changes should be treated as sensitive and should not be handled like ordinary small edits:

- the licensing model;
- commercial policy;
- CLA and contribution policy;
- trust-root and release signing;
- the definition of an official build;
- branding and trademark rules;
- release gate status;
- root governance and ownership files.

For OCP, changes like these should follow a path with:
- a clear plan,
- evidence,
- appropriate tests when the area is machine-checkable,
- a proper closeout.

## 6) What does “official build” mean in governance terms?

From the governance side, “official build” should not be understood as:
- any build that looks like OCP;
- any build that someone else compiled from the same source;
- any fork that keeps a similar name.

The safe understanding is:
- an official build must be tied to the project's release authority;
- an official build must fit the project's trust chain and release policy;
- fork or community builds may exist, but they should not be presented as official unless they actually belong to the official release chain.

More detail about name, logo, and branding belongs in:
- `TRADEMARK.md`
- `docs/en/legal/trademark.md`

## 7) How does governance relate to the plan and gates?

In OCP, the plan is not just a work note.

It is part of practical governance because it locks:
- scope,
- gates,
- `DONE` criteria,
- evidence,
- relevant tests.

That means:
- no one should declare a gate complete without enough evidence;
- no one should silently change the design or lower the bar without writing down why;
- the project's governance is tightly tied to a contract-first, evidence-backed rule set.

In simple terms:
- major technical decisions must leave evidence behind;
- an official release must not be finalized by informal verbal agreement alone.

## 8) If there is disagreement, what principle should be used?

At the current public-doc level, the safe reading is:

1. Prioritize the already locked sources of truth:
   - contracts,
   - the current plan,
   - root legal and governance files,
   - release evidence.
2. If something is only a proposal and is not yet locked in those sources, it is not yet official policy.
3. If an explanation conflicts with a locked source, the locked source wins.

This prevents situations where:
- one person says one thing,
- a PR says another,
- the README says something else,
- and nobody can tell what is actually official.

## 9) How much governance does an end user need to understand?

Most users do not need to study governance deeply.

But if you are:
- a long-term contributor,
- someone evaluating adoption for an organization,
- someone concerned with legal or release legitimacy,
- or someone who wants to know who really has authority over project direction,

then this file helps you understand:
- the project is not a model where everyone who contributes has equal governing authority;
- release and policy direction are explicitly controlled;
- official status must be tied to authority and trust chain.

## 10) What does this file not replace?

`docs/en/legal/governance.md` does not replace:
- `GOVERNANCE.md` at the root;
- `CODEOWNERS`;
- `TRADEMARK.md`;
- the plan and contracts that lock project decisions;
- release evidence and signoff records.

It is only an explanation document to help readers understand how the project operates at the governance level.

## 11) Conclusion

The shortest safe understanding is:
- OCP currently has clear owner authority;
- contribution is not the same thing as governance authority;
- release, licensing, commercial direction, and official-build status must go through explicit governance;
- if something is not locked in the sources of truth and backed by evidence, it should not yet be treated as an official project decision.
