# 🗺️ Master Roadmap: Strategic Vision & Justifications

This roadmap details the transformation of **Wazuh Agent Status** into a premium security utility. Each feature is explained in both simple terms and professional business contexts.

---

## 🥇 Phase 1: Trust & Security Hardening

### 🔒 Security Hardening (Making it Private & Authorized)

- **Simple View**: We are locking the "internal phone line" between our app components.
- **Technical Logic**: We will implement **TLS Encryption** for transport and **JWT/Token-based Authentication** for commands.
- **Why We Need It**: In an enterprise, security tools are prime targets. Without these, a malicious script could easily "pause" your security monitoring without you knowing.
- **Market Value**: This allows you to sell the tool as **"Zero-Trust Compliant."**
- **⚡ Rust Edge**: Rust's `rustls` and `jsonwebtoken` crates provide industry-leading performance and safety for encrypted handshakes, ensuring zero-trust enforcement without slowing down the workstation.

### 🔏 Code Signing (The Seal of Quality)

- **Simple View**: Getting a digital "Official Seal" so your computer trusts the app.
- **Technical Logic**: Signing binaries with an **Extended Validation (EV) Certificate** and performing **macOS Notarization**.
- **Why We Need It**: Avoids Windows SmartScreen or macOS Gatekeeper blocks that make external software look like malware.
- **Market Value**: Essential for **Professionalism and Trust**. Customers won't install a security tool that their OS flags as "dangerous."
- **⚡ Rust Edge**: The `cargo-dist` toolchain automates binary signing and notarization for Windows and macOS, ensuring a "Green Shield" trust score on every release.

---

## 🥈 Phase 2: Enterprise Operations

### 📦 Silent Installers (One-Click Mass Deployment)

- **Simple View**: Making it easy for IT managers to install this on 1,000 PCs at once.
- **Technical Logic**: Developing native **MSI (Windows)** and **PKG (macOS)** installers with support for silent flags (`/quiet`, `/qn`).
- **Why We Need It**: Modern IT departments use tools like **Microsoft Intune** or **Jamf**. They require standard packages that don't need human interaction.
- **Market Value**: This is a **Scalability** feature. It makes your tool "ready for big business."
- **⚡ Rust Edge**: `cargo-dist` generates native `.msi` (Windows) and `.pkg` (macOS) installers directly as part of the build pipeline, reducing deployment overhead.

---

## 🥉 Phase 3: Serviceability & Diagnostics

### 📜 Integrated Log Streamer (The "Black Box")

- **Simple View**: A window that shows exactly what the security agent is doing.
- **Technical Logic**: A real-time tail of the `ossec.log` with filtering and keyword highlighting (e.g., ERROR, WARNING).
- **Why We Need It**: Troubleshooting connection issues currently requires manually searching system folders. This puts the answers at the user's fingertips.
- **Market Value**: Reduces **Support Costs** (MTTR - Mean Time To Resolution) by empowering users and local admins to fix issues themselves.
- **⚡ Rust Edge**: Rust's zero-copy asynchronous I/O allows us to stream large log files (`ossec.log`) in real-time with almost **zero CPU overhead**, making the viewer lightweight and responsive.

### 🩹 Self-Healing (The Auto-Fix)

- **Simple View**: If the security agent stops, the app automatically brings it back up.
- **Technical Logic**: A background monitoring loop that detects `service stopped` states and triggers a `service start` command using backoff logic.
- **Why We Need It**: Minimizes protection gaps. You shouldn't have to wait for an IT ticket to be resolved to stay protected.
- **Market Value**: Provides **"High Availability"** of the security posture.
- **⚡ Rust Edge**: Rust's deterministic memory management ensures that the background monitoring loop is extremely stable and resilient, even in low-memory situations.

---

## 🥈 Phase 4: UX & Managed Services (Personalized Experience)

### 🎨 White-Labeling & Identity Branding (Custom Themes)

- **The Feature**: Ability to fully customize the tray app's appearance for different clients.
- **What can be customized**:
  - **Custom Logo**: Use the client's own corporate logo instead of the default Wazuh one.
  - **Company Name**: Show "Protected by [Your Company Name]" in the tray tooltip.
  - **Custom Themes**: Support for Dark/Light modes and brand-specific accent colors.
  - **Splash Screen**: Add a small, professional "Starting..." splash screen with the MSP's identity.
- **Benefit**: This is a **massive "MSP-Ready" feature**. It allows security providers to build their own brand and show daily value to their clients right on their desktops.

### ✅ Compliance Dashboard Overlay

- **Simple View**: A simple checklist showing if your computer follows the company's security rules.
- **Technical Logic**: Integrating with Wazuh's SCA (Security Configuration Assessment) results to show a "Health Score."
- **Market Value**: It turns a passive tool into an **Interactive Compliance Hub**, which is a huge selling point for CISOs.

---

## 🌟 Phase 5: Extended System Hygiene (Tray Icon Expansion)

We are expanding the tray icon's scope to monitor the overall "Health" of the workstation, beyond just the Wazuh Agent.

### 🔄 OS Update Monitoring (Keep the Foundation Secure)

- **The Feature**: The tray icon will notify you if your Operating System (Linux, Windows, or macOS) has pending security updates.
- **Feasibility**:
  - **Monitoring**: Fully feasible. We can run background checks (e.g., `apt list --upgradable`) and show an "Update Available" badge on the icon.
  - **Action**: We can provide a "Trigger Update" button that launches the system's native update tool or runs a background script.
- **Benefit**: Outdated OS versions are the #1 way hackers get in. Monitoring this directly alongside Wazuh provides total peace of mind.

### 🛡️ Extended Security Indicators

- **VPN Status**: A simple indicator showing if your secure work tunnel is active.
- **Disk Encryption**: A green lock icon if **BitLocker** (Windows) or **FileVault** (macOS) is active.
- **Firewall Status**: A quick check to see if the system firewall is blocking unauthorized traffic.
- **Why This Works**: It transforms the app into a **Single Source of Truth** for the machine's security state.

---

## 🚀 Phase 6: Resource Guard & Performance Optimization

We are adding an intelligent "Resource Guard" to ensure that your security tools don't slow down your workstation.

### 📉 Resource Guard (Monitoring & Auto-Fixing)

- **The Feature**: The application will monitor the computer's CPU and Memory usage in real-time.
- **The "Auto-Fix" Strategy**:
  - **Detection**: If a process (like a web browser or a background task) starts using too much CPU, the app will flag it.
  - **Soft-Fix (Normalization)**: On Linux and macOS, the app can automatically "lower the priority" of the greedy process (using `renice`). This allows other important tasks to run smoothly without killing the user's work.
  - **Notification**: The tray icon will notify the user: _"Google Chrome is using 95% CPU. I've optimized it for you!"_
- **⚡ Rust Edge**: This is where Rust truly shines. Using the `sysinfo` crate, we can monitor system metrics with **microsecond precision** and **near-zero RAM usage**. Rust's direct access to system calls ensures "Soft-Fixes" are executed with no lag.
- **Feasibility**: 100% feasible using background process monitoring libraries.
- **Benefit**: Users often hate security apps because they "slow down the machine." By adding a "Performance Optimizer," you turn the app into something users **want** to keep running.
