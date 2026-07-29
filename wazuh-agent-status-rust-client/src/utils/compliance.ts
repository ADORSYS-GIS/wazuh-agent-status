// ─── Markdown Parser ──────────────────────────────────────────────────────────

export interface MarkdownChunk {
  type: "text" | "heading2" | "heading3" | "step" | "list_item" | "code_block";
  content: string;
  language?: string;
}

interface ParseState {
  riskLevel: string | null;
  impact: string | null;
  currentSection: "risk" | "impact" | null;
}

const CODE_FENCE_RE = /```(bash|sh|powershell|cmd|shell|zsh)?\n([\s\S]*?)```/;

function parseCodeBlock(part: string): MarkdownChunk {
  const match = CODE_FENCE_RE.exec(part);
  if (match) {
    return { type: "code_block", content: match[2].trim(), language: match[1] || "bash" };
  }
  return { type: "code_block", content: part.replace(/```/g, "").trim(), language: "bash" };
}

function parseTextLine(trimmed: string, chunks: MarkdownChunk[], state: ParseState): void {
  if (trimmed.startsWith("## Risk Level")) {
    state.currentSection = "risk";
    return;
  }
  if (trimmed.startsWith("## Impact")) {
    state.currentSection = "impact";
    return;
  }
  if (trimmed.startsWith("## ")) {
    state.currentSection = null;
    chunks.push({ type: "heading2", content: trimmed.slice(3) });
    return;
  }
  if (trimmed.startsWith("### ")) {
    state.currentSection = null;
    chunks.push({ type: "heading3", content: trimmed.slice(4) });
    return;
  }
  if (/^\d+\.\s/.test(trimmed)) {
    state.currentSection = null;
    chunks.push({ type: "step", content: trimmed });
    return;
  }
  if (trimmed.startsWith("- ") || trimmed.startsWith("* ")) {
    state.currentSection = null;
    chunks.push({ type: "list_item", content: trimmed.replace(/^[-*]\s+/, "") });
    return;
  }
  if (state.currentSection === "risk") {
    state.riskLevel = trimmed;
  } else if (state.currentSection === "impact") {
    state.impact = (state.impact ? state.impact + " " : "") + trimmed;
  } else {
    chunks.push({ type: "text", content: trimmed });
  }
}

export function parseMarkdownIntoChunks(markdown: string) {
  const chunks: MarkdownChunk[] = [];
  const state: ParseState = { riskLevel: null, impact: null, currentSection: null };

  if (!markdown) return { chunks, riskLevel: state.riskLevel, impact: state.impact };

  for (const part of markdown.split(/(```[\s\S]*?```)/g)) {
    if (part.startsWith("```")) {
      chunks.push(parseCodeBlock(part));
    } else {
      for (const line of part.split("\n")) {
        const trimmed = line.trim();
        if (trimmed) parseTextLine(trimmed, chunks, state);
      }
    }
  }

  return { chunks, riskLevel: state.riskLevel, impact: state.impact };
}

// ─── Score Helpers ────────────────────────────────────────────────────────────

export function scoreColor(score: number): string {
  if (score >= 80) return "var(--success)";
  if (score >= 50) return "var(--warning)";
  return "var(--error)";
}

export function scoreLabel(score: number): string {
  if (score >= 80) return "Good";
  if (score >= 50) return "Fair";
  return "Poor";
}

// ─── Command Helpers ─────────────────────────────────────────────────────────

const WINDOWS_ADMIN_VERBS = [
  "Restart-Service", "Start-Service", "Stop-Service", "Set-Service",
  "New-Service", "Remove-Service",
  "Install-WindowsFeature", "Uninstall-WindowsFeature",
  "Add-WindowsFeature", "Remove-WindowsFeature",
  "Set-ItemProperty", "Remove-ItemProperty", "New-ItemProperty",
  "Set-ExecutionPolicy",
  "New-Item", "Remove-Item", "Set-Item",
  "New-LocalUser", "Remove-LocalUser", "Set-LocalUser",
  "Add-LocalGroupMember", "Remove-LocalGroupMember",
  "Enable-WindowsOptionalFeature", "Disable-WindowsOptionalFeature",
  "Set-NetFirewallRule", "New-NetFirewallRule", "Remove-NetFirewallRule",
  "Set-NetIPAddress", "New-NetIPAddress", "Remove-NetIPAddress",
  "Set-NetAdapter", "Disable-NetAdapter", "Enable-NetAdapter",
  "Set-NetConnectionProfile",
  "repadmin", "gpupdate", "gpresult", "secedit",
  "Reg", "reg", "sc.exe", "sc",
  "icacls", "takeown",
];

const ADMIN_VERB_PATTERN = WINDOWS_ADMIN_VERBS.map((v) =>
  v.replace(/[.*+?^${}()|[\]\\]/g, String.raw`\$&`)
).join("|");
const ADMIN_VERB_RE = new RegExp(
  `(?:^|[\\s|;&'"/])(${ADMIN_VERB_PATTERN})(?=[\\s|;&'"/]|$)`
);

export const commandNeedsSudo = (cmd: string) => {
  if (/(?:^|\s)sudo(?:\s|$)/.test(cmd)) return true;
  return ADMIN_VERB_RE.test(cmd.trim());
};

export const commandIsInteractive = (cmd: string) =>
  /\b(nano|vim?|emacs|gedit|kate|mousepad|xed|less|more|most|htop|top|man|watch|tail\s+-f)\b/.test(cmd);

export const commandIsDestructive = (cmd: string) =>
  /\brm\s+-[a-zA-Z]*[rf]/.test(cmd) ||
  /\bdd\s+(if|of)=/.test(cmd) ||
  /\bmkfs\b/.test(cmd) ||
  /\bshred\b/.test(cmd) ||
  cmd.includes(":(){ :|:& };:");

// ─── Relative Time ────────────────────────────────────────────────────────────

export function formatRelativeTime(date: Date): string {
  const diff = Date.now() - date.getTime();
  const secs = Math.floor(diff / 1000);
  if (secs < 5) return "just now";
  if (secs < 60) return `${secs}s ago`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return mins === 1 ? "1m ago" : `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return hours === 1 ? "1h ago" : `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return days === 1 ? "1d ago" : `${days}d ago`;
}
