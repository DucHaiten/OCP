# Trademark OCP v1.0

This document explains the OCP trademark and branding policy in user-readable language for users, contributors, and parties who may want to distribute derivative builds.

It does not replace the root source file:
- `TRADEMARK.md`

This file mainly helps answer practical questions such as:
- When is a build considered an official OCP build?
- Can a fork say that it is based on OCP?
- Can someone use the OCP name or logo for a separate release?

## Short Summary

If you only need the quick version:

1. Not every build using OCP source is an official build.
2. Only a build inside the official release scope and the official trust chain should be called an `Official OCP Build`.
3. A fork is allowed to say clearly that it is based on OCP.
4. A fork must not present itself as if it were the official OCP build.

## 1) What is this trademark policy for?

Trademark policy does not decide whether you may use the source code.  
That question belongs to the license.

Trademark policy controls a different question:
- how you may present your build or product to users.

In simple terms:
- the license answers “may I use the code?”;
- the trademark policy answers “may I present this build as the official OCP build?”.

## 2) When is a build considered official?

A build should only be considered an `Official OCP Build` when:
- it belongs to the official project release;
- it is inside the signed artifact scope of that release;
- it appears in the official release manifest;
- it belongs to the trust chain and signature chain published by the project.

If those conditions are missing, the safe understanding is:
- it is not an official build.

Very important point:
- just because a build works,
- or because it uses the same source,
- or because the interface looks identical,

does not mean it is allowed to call itself official.

## 3) Can forks exist and publish their own builds?

Yes.

A fork or community build is allowed to:
- exist;
- publish a separate build;
- say clearly that it is based on OCP;
- describe its origin honestly.

Examples of acceptable wording:
- `based on OCP`
- `fork of OCP`
- `community build derived from OCP`

## 4) What must a fork not do?

A fork or derivative build should not:
- call itself `Official OCP Build`;
- call itself `Official OCP`;
- present itself as if it were an official project release;
- use branding in a way that makes it difficult for an ordinary user to distinguish the fork from the official build.

In short:
- it is allowed to describe the origin;
- it is not allowed to impersonate the official build.

## 5) May the name “OCP” be used?

Yes, but it must be used correctly.

Allowed:
- referencing OCP in order to describe project origin;
- stating that your product is based on OCP;
- explaining the relationship between your build and the upstream project.

Not recommended:
- naming or marketing your build in a way that makes users think it is official;
- using `OCP` as the primary brand of a fork without making the fork or community-build status clear.

## 6) May the OCP logo or branding be used?

The current safe understanding is:
- do not use the OCP logo or branding in a way that makes a fork or separate build look official;
- do not repackage another build and present it with official-looking identity so that users think it is part of the official release chain.

If you only need to describe origin, the safest approach is:
- use clear written wording;
- avoid mimicking the official branding.

## 7) How is the official VSCode extension identified?

At the v1.0 state, the VSCode extension should only be considered official when:
- it belongs to the official project release;
- it is inside the signed release scope;
- the publisher identity matches the official release chain.

In current public-facing docs, the official identity is:
- publisher: `ocp`

That means:
- an extension published by a different publisher should not be presented as the official OCP extension unless it truly belongs to the official release chain.

## 8) How does trademark policy relate to the trust chain?

In OCP, trademark does not stand apart from release integrity.

A build is not official merely because:
- the filename looks similar,
- the icon looks similar,
- or the content is close.

It must also:
- appear in the official release manifest;
- carry signatures that fit the official trust chain.

That is why users should look at release evidence, not just names.

## 9) How should end users check quickly?

If you only want a quick check of whether a build is official, ask:

1. Is this build part of the project's official release?
2. Is it inside the official manifest and signature chain?
3. Is it claiming to be official without being able to prove that claim?

If you cannot answer those three questions with confidence, the safe reading is:
- this is not yet a proven official build.

## 10) What does this file not do?

This file does not:
- ban forks;
- ban community builds;
- replace the license;
- replace the release verification guide;
- grant anyone the right to use the brand as if they were the official build.

It does one thing:
- it makes the boundary between the official build and derivative builds clearer.

## 11) Conclusion

The simplest reading is:
- you may fork and build from OCP;
- you may say clearly that your build is based on OCP;
- but only a build inside the official release and the official trust chain should be called an `Official OCP Build`.
