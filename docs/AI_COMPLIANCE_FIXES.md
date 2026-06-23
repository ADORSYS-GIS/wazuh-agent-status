# AI Compliance Fixes & Remediation

The Wazuh Agent Status application integrates AI-powered remediation advice to help users resolve failed **Security Configuration Assessment (SCA)** compliance checks.

This document describes how the AI remediation feature works, its technical architecture, command execution mechanics, and security controls.

---

## 1. Overview of the Fix Workflow

When a compliance check fails (e.g., password expiration policy, unused services enabled), the UI offers a **"Fix with AI"** option.

```
┌──────────────────────┐     1. Request Fix     ┌────────────────────────┐
│  ComplianceView UI   │ ─────────────────────► │  Tauri Backend (Rust)  │
│  (React Frontend)    │ ◄───────────────────── │  (AI Client Handler)   │
└──────────────────────┘     4. Parse & Display └────────────────────────┘
           │                                                 │
           │ 2. Sudo Cmd Exec (with Password)                │ 3. Call AI API
           └─────────────────────────────────────────────────┼────────────────► [ AI Provider ]
                                                             │                  (Gemini/DeepSeek/etc.)
                                                             ▼
                                                    ┌─────────────────┐
                                                    │ Host System OS  │
                                                    │ (Local Bash/sh) │
                                                    └─────────────────┘
```

1. **AI Fix Suggestion**: The frontend sends check metadata (such as category, title, remediation guidelines, and target OS) to the Tauri Rust backend.
2. **API Request**: The backend forwards the request to the configured AI Provider (Gemini, DeepSeek, Custom OpenAI-compatible endpoint, etc.).
3. **Markdown Parsing**: The AI returns a detailed remediation plan in Markdown. The frontend parses the markdown on-the-fly to isolate risk levels, impact analysis, description text, and actual executable commands.
4. **Interactive Fix Cards**: The commands are displayed in terminal-like cards. Sudo-requiring commands are highlighted, and non-interactive scriptable actions can be executed directly from the application.
5. **SCA Rescan**: To confirm the issue is fixed, users can trigger an immediate agent-level compliance scan from the UI.

---

## 2. Command Execution Mechanics

Executing terminal commands from an unprivileged desktop application requires careful shell session handling:

### Standard Command Execution

For standard (non-sudo) actions, the Tauri Rust backend executes commands via a child process:

- On **Linux/macOS**: Invokes `sh -c "<command>"`.
- On **Windows**: Invokes `powershell.exe -NoProfile -NonInteractive -Command "<command>"`.
  Stdout and stderr are captured and returned to the UI terminal view in real-time.

### Sudo Command Execution (`sudo -S`)

Many security remediations require root privileges. Standard `sudo <command>` calls wait for password input on `stderr`/`tty`, which causes UI applications to hang indefinitely.
To resolve this:

- The app requests the user's sudo password in the UI.
- The backend executes: `echo <password> | sudo -S -p '' <command>`.
- The `-S` flag forces `sudo` to read the password from standard input, and `-p ''` silences any custom password prompts, preventing execution hangs.

### Force SCA Rescan

Wazuh does not have a CLI command or API endpoint to trigger a Security Configuration Assessment (SCA) scan on demand. The only way to force an immediate scan is to restart the agent service.

- The **"Trigger SCA Rescan"** action executes a service restart:
  `echo <password> | sudo -S -p '' systemctl restart wazuh-agent`
- Since the Wazuh agent is configured with `<scan_on_start>yes</scan_on_start>` by default, the service restart forces a fresh compliance scan to run immediately and update the manager.

---

## 3. Security Controls & Architecture

Running shell commands suggested by an AI requires strict security boundaries to prevent malicious input execution or credential leaks.

### Password Privacy

- **RAM-Only State**: The user's sudo password is saved only in React state memory (`useState`). It is **never** written to the disk, configuration files, cache, or logs.
- **Shared Password Session**: Sudo passwords are shared across the execution session inside the modal. Once typed in one step, it auto-fills the others.
- **Wiped on Close**: When the fix modal is closed, the password state is immediately reset to `""` (empty string) to clear it from memory.
- **No AI Access**: The password is never sent to the AI API or transmitted over the network. It remains strictly local to the React client and the Tauri IPC bridge.

### Interactive Editor Prevention

AI models occasionally suggest opening files in interactive CLI editors (e.g., `sudo nano /etc/pam.d/su` or `vim`). Running these inside an automated shell child process will hang the execution forever.

1. **Prompt Containment**: The backend system prompt instructs the AI model to _never_ suggest interactive tools, requiring scriptable equivalents instead (e.g., using `sed`, `tee`, `echo`, or file redirection).
2. **Frontend Interceptor**: The frontend uses a regex pattern detector (`/\b(nano|vim?|emacs|gedit|kate)\b/`) to scan suggested commands. If an interactive editor command is detected, the **Execute** button is disabled, and a warning banner advises the user to run it manually in their system terminal.

---

## 4. Configuration

To enable AI remediation:

1. Navigate to the **Settings** tab in the Wazuh Agent Status app.
2. Select your preferred **AI Provider** (e.g. Gemini, OpenAI, Anthropic, DeepSeek, or custom endpoint).
3. Provide the required **API Key** and **Model Name** (e.g., `deepseek-chat`, `gemini-1.5-flash`).
4. Click **Test Connection** to verify connection. Once saved, the "Fix with AI" buttons will activate on the Compliance tab.
