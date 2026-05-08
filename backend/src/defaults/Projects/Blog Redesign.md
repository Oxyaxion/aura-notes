---
title: Blog Redesign
tags:
  - project
  - web
  - design
type: project
status: active
priority: medium
area: pro
date: 2026-04-01
due: 2026-06-30
project: Blog
---

# Blog Redesign

Rebuild the personal blog with a focus on readability and load speed.

## Goals

- Static site — no CMS, no JS framework
- Sub-1s load time on mobile
- Dark mode support
- RSS feed

## Stack

- **Generator**: Zola (Rust, single binary, fast)
- **Hosting**: Cloudflare Pages (free tier)
- **Domain**: existing

## Tasks

- [x] Choose generator
- [x] Pick base theme
- [ ] Migrate old posts (12 articles)
- [ ] Custom CSS pass
- [ ] Set up deploy pipeline
- [ ] Redirect old URLs
