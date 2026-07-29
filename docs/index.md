---
layout: default
title: Home
nav_order: 1
---

# Wazuh Agent Status Documentation

Wazuh Agent Status is a self-hosted desktop application built with Rust and Tauri that provides real-time visibility and control over your local Wazuh security agent. Read these docs to install, operate, extend, and contribute to the system.

<div class="hero-buttons">
  <a href="features/index.html" class="btn-hero btn-hero-primary">Get Started &rarr;</a>
  <a href="architecture/index.html" class="btn-hero btn-hero-outline">How it works</a>
</div>

## Features

<p class="section-desc">Explore the detailed guides on everything from real-time log streaming to automated AI-powered remediation.</p>

<div class="docs-grid">
  <a href="features/ai-remediation.html" class="docs-card">
    <div class="docs-card-title">AI-Powered Remediation</div>
    <p class="docs-card-desc">Automatically fix compliance failures using AI-generated shell commands.</p>
  </a>

  <a href="features/compliance-dashboard.html" class="docs-card">
    <div class="docs-card-title">Compliance Dashboard</div>
    <p class="docs-card-desc">Review your system's security posture and specific SCA check results.</p>
  </a>

  <a href="features/log-streaming.html" class="docs-card">
    <div class="docs-card-title">Log Streaming</div>
    <p class="docs-card-desc">Stream and filter your local Wazuh agent logs in real-time.</p>
  </a>

  <a href="features/auto-update.html" class="docs-card">
    <div class="docs-card-title">Auto Updates</div>
    <p class="docs-card-desc">Keep your client up-to-date with integrated release channel management.</p>
  </a>
</div>

## Architecture

<p class="section-desc">The ideas that make Wazuh Agent Status work: client-server isolation, security boundaries, and high-performance Rust design.</p>

<div class="docs-grid">
  <a href="architecture/architecture.html" class="docs-card">
    <div class="docs-card-title">System Design</div>
    <p class="docs-card-desc">Detailed design of the tray client and privileged background service.</p>
  </a>

  <a href="roadmap.html" class="docs-card">
    <div class="docs-card-title">Master Roadmap</div>
    <p class="docs-card-desc">Strategic vision, technical justifications, and upcoming features.</p>
  </a>

  <a href="architecture/index.html" class="docs-card">
    <div class="docs-card-title">ADRs</div>
    <p class="docs-card-desc">Architectural Decision Records documenting major technical choices.</p>
  </a>
  
  <a href="https://github.com/ADORSYS-GIS/wazuh-agent-status" class="docs-card">
    <div class="docs-card-title">Source Code</div>
    <p class="docs-card-desc">View the repository on GitHub and contribute to the project.</p>
  </a>
</div>
