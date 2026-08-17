# Contributing to Xytro Labs Web

Thanks for your interest in improving the Xytro Labs websites!

This repository holds the static, public-facing websites and documentation
for Xytro Labs' products. We welcome contributions that:

- Fix typos, broken links, or factual inaccuracies in documentation
- Improve website accessibility, performance, or mobile responsiveness
- Propose clearer copy or better structure for the help/docs pages

## Getting started

These are static HTML/CSS pages — no build step is required to view a page.
Open the relevant `index.html` in a browser or serve the directory locally:

```bash
python3 -m http.server 8000
```

## What we don't accept here

- **Do not commit secrets, API keys, credentials, internal hostnames, or
  private infrastructure details** into this public repository.
- Do not add personal data or proprietary product source code to this repo.
  Product backends live elsewhere and are not published here.
- Only public-facing marketing/documentation content belongs in this repo.

## Making a change

1. Fork the repository.
2. Create a branch: `git checkout -b your-fix`.
3. Make your change and verify the page renders correctly.
4. Commit with a clear message describing the change.
5. Open a pull request against `main`.

## Code of conduct

Be respectful and constructive. Harassment or abuse of any kind is not
tolerated.
