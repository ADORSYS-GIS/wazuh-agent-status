/**
 * ─── Brand Module ────────────────────────────────────────────────────────────
 *
 * Single source of truth for all branding/theming.
 *
 * **How to customise for a new company/brand:**
 * 1. Edit `app_config.json` → `brand` section (primary_color, logo_path, etc.)
 * 2. Place the logo file in `public/` and point `logo_path` to it
 * 3. Set `dark_mode: false` for light theme (colors adapt automatically)
 *
 * All CSS custom properties used across the app are computed in one place below.
 */

import type { BrandConfig } from "../types/app";

// ─── Default palettes ────────────────────────────────────────────────────────

type ThemeMode = "dark" | "light";

type ThemeDefaults = {
  bg: string;
  sidebarBg: string;
  cardBg: string;
  text: string;
  textDim: string;
  border: string;
  success: string;
  error: string;
  warning: string;
  overlayWhite05: string;
  overlayWhite08: string;
  overlayWhite10: string;
  overlayWhite12: string;
  overlayWhite20: string;
  overlayBlack30: string;
  overlayBlack40: string;
  overlayBlack50: string;
  tooltipBg: string;
  tooltipColor: string;
  terminalBg: string;
  terminalBorder: string;
  terminalLineColor: string;
  progressTrackBg: string;
  scrollbarThumb: string;
  shimmerColor: string;
  cardHoverBg: string;
  titlebarHeight: string;
  sidebarWidth: string;
  navItemSize: string;
};

const THEME_DEFAULTS: Record<ThemeMode, ThemeDefaults> = {
  dark: {
    bg: "#0b1120",
    sidebarBg: "#111827",
    cardBg: "rgba(17, 24, 39, 0.7)",
    text: "#f8fafc",
    textDim: "#94a3b8",
    border: "rgba(255, 255, 255, 0.08)",
    success: "#10b981",
    error: "#f43f5e",
    warning: "#fbbf24",
    // Overlay / effect colours (var(--overlay-*) used in App.css)
    overlayWhite05: "rgba(255, 255, 255, 0.05)",
    overlayWhite08: "rgba(255, 255, 255, 0.08)",
    overlayWhite10: "rgba(255, 255, 255, 0.10)",
    overlayWhite12: "rgba(255, 255, 255, 0.12)",
    overlayWhite20: "rgba(255, 255, 255, 0.20)",
    overlayBlack30: "rgba(0, 0, 0, 0.30)",
    overlayBlack40: "rgba(0, 0, 0, 0.40)",
    overlayBlack50: "rgba(0, 0, 0, 0.50)",
    // Misc
    tooltipBg: "#1e293b",
    tooltipColor: "#ffffff",
    terminalBg: "#0a0a0f",
    terminalBorder: "#1e1e2a",
    terminalLineColor: "#8b8b9e",
    progressTrackBg: "rgba(255, 255, 255, 0.05)",
    scrollbarThumb: "rgba(255, 255, 255, 0.1)",
    shimmerColor: "rgba(255, 255, 255, 0.05)",
    cardHoverBg: "rgba(30, 41, 59, 0.8)",
    // Layout
    titlebarHeight: "32px",
    sidebarWidth: "64px",
    navItemSize: "40px",
  },
  light: {
    bg: "#f1f5f9",
    sidebarBg: "#ffffff",
    cardBg: "rgba(255, 255, 255, 0.85)",
    text: "#0f172a",
    textDim: "#64748b",
    border: "rgba(0, 0, 0, 0.10)",
    success: "#16a34a",
    error: "#dc2626",
    warning: "#d97706",
    // Overlay / effect colours — dark overlays on light backgrounds
    overlayWhite05: "rgba(0, 0, 0, 0.04)",
    overlayWhite08: "rgba(0, 0, 0, 0.06)",
    overlayWhite10: "rgba(0, 0, 0, 0.08)",
    overlayWhite12: "rgba(0, 0, 0, 0.10)",
    overlayWhite20: "rgba(0, 0, 0, 0.12)",
    overlayBlack30: "rgba(0, 0, 0, 0.15)",
    overlayBlack40: "rgba(0, 0, 0, 0.20)",
    overlayBlack50: "rgba(0, 0, 0, 0.25)",
    // Misc
    tooltipBg: "#1e293b",
    tooltipColor: "#ffffff",
    terminalBg: "#f8fafc",
    terminalBorder: "#e2e8f0",
    terminalLineColor: "#475569",
    progressTrackBg: "rgba(0, 0, 0, 0.06)",
    scrollbarThumb: "rgba(0, 0, 0, 0.15)",
    shimmerColor: "rgba(0, 0, 0, 0.04)",
    cardHoverBg: "rgba(255, 255, 255, 0.95)",
    // Layout
    titlebarHeight: "32px",
    sidebarWidth: "64px",
    navItemSize: "40px",
  },
};

// ─── CSS variable computation ────────────────────────────────────────────────

/**
 * Compute ALL CSS custom properties that the app uses, derived from the brand
 * configuration. This is the **single source of truth** for every colour and
 * dimension consumed by App.css and its children.
 */
export function computeBrandCSS(brand: BrandConfig): Record<string, string> {
  const { primary_color, secondary_color, dark_mode, bg, sidebar_bg, text, text_dim } = brand.theme;

  const glow = toRgba(primary_color, 0.6);
  const metallic = `linear-gradient(135deg, ${primary_color}, #ffffff44, ${primary_color})`;

  const P = (hex: string, a: number) => toRgba(hex, a);

  const mode = THEME_DEFAULTS[dark_mode ? "dark" : "light"];

  return {
    // ── Core brand colours ──────────────────────────────────────────────
    "--primary": primary_color,
    "--primary-glow": glow,
    "--primary-metallic": metallic,
    "--secondary": secondary_color,

    // ── Surface & text (config values override mode defaults) ────────────
    "--bg": bg ?? mode.bg,
    "--sidebar-bg": sidebar_bg ?? mode.sidebarBg,
    "--card-bg": mode.cardBg,
    "--text": text ?? mode.text,
    "--text-dim": text_dim ?? mode.textDim,
    "--border": mode.border,

    // ── Semantic colours ────────────────────────────────────────────────
    "--success": mode.success,
    "--error": mode.error,
    "--warning": mode.warning,

    // ── Overlay / effect colours (consume via var(--overlay-*) in CSS) ───
    "--overlay-white-05": mode.overlayWhite05,
    "--overlay-white-08": mode.overlayWhite08,
    "--overlay-white-10": mode.overlayWhite10,
    "--overlay-white-12": mode.overlayWhite12,
    "--overlay-white-20": mode.overlayWhite20,
    "--overlay-black-30": mode.overlayBlack30,
    "--overlay-black-40": mode.overlayBlack40,
    "--overlay-black-50": mode.overlayBlack50,

    // ── Component-specific ──────────────────────────────────────────────
    "--text-on-primary": isLightColor(primary_color) ? "#0f172a" : "#ffffff",
    "--tooltip-bg": mode.tooltipBg,
    "--tooltip-color": mode.tooltipColor,
    "--terminal-bg": mode.terminalBg,
    "--terminal-border": mode.terminalBorder,
    "--terminal-line-color": mode.terminalLineColor,
    "--progress-track-bg": mode.progressTrackBg,
    "--scrollbar-thumb": mode.scrollbarThumb,
    "--shimmer-color": mode.shimmerColor,
    "--card-hover-bg": mode.cardHoverBg,

    // ── Primary-alpha helpers (driven by primary_color) ─────────────────
    "--primary-alpha-04": P(primary_color, 0.04),
    "--primary-alpha-08": P(primary_color, 0.08),
    "--primary-alpha-10": P(primary_color, 0.1),
    "--primary-alpha-15": P(primary_color, 0.15),
    "--primary-alpha-20": P(primary_color, 0.2),
    "--primary-alpha-30": P(primary_color, 0.3),
    "--primary-alpha-40": P(primary_color, 0.4),

    // ── Layout ──────────────────────────────────────────────────────────
    "--titlebar-height": mode.titlebarHeight,
    "--sidebar-width": mode.sidebarWidth,
    "--nav-item-size": mode.navItemSize,
    "--transition-fast": "0.2s ease",
    "--transition-smooth": "0.3s cubic-bezier(0.16, 1, 0.3, 1)",
  };
}

/**
 * Return the URL path for the brand logo.
 * Logo should be placed in `public/` and referenced by filename in `logo_path`.
 */
export function getBrandLogoUrl(brand: BrandConfig): string {
  return `/${brand.logo_path.replace(/^\//, "")}`;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/**
 * Parse a 3- or 6-digit hex colour into [r, g, b] components.
 * Returns null if the input is not a recognised format.
 */
function hexToRgb(hex: string): [number, number, number] | null {
  const clean = hex.replace("#", "");

  if (clean.length === 3) {
    return [
      Number.parseInt(clean[0] + clean[0], 16),
      Number.parseInt(clean[1] + clean[1], 16),
      Number.parseInt(clean[2] + clean[2], 16),
    ];
  }
  if (clean.length === 6) {
    return [
      Number.parseInt(clean.slice(0, 2), 16),
      Number.parseInt(clean.slice(2, 4), 16),
      Number.parseInt(clean.slice(4, 6), 16),
    ];
  }
  return null;
}

/**
 * Returns `true` if a hex colour is perceptually light (text-on-primary
 * should switch to dark for readability).
 */
function isLightColor(hex: string): boolean {
  const rgb = hexToRgb(hex);
  if (!rgb) return false;
  // Relative luminance (sRGB coefficients)
  const luminance = 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2];
  return luminance > 160;
}

function toRgba(hex: string, alpha: number): string {
  const rgb = hexToRgb(hex);
  if (!rgb) return `rgba(0, 170, 255, ${alpha})`;
  return `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, ${alpha})`;
}
