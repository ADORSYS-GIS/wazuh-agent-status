export interface ThemeConfig {
    primary_color: string;
    secondary_color: string;
    dark_mode: boolean;
    bg?: string;
    sidebar_bg?: string;
    text?: string;
    text_dim?: string;
}

export interface BrandConfig {
    name: string;
    company: string;
    managed_by?: string;
    logo_path: string;
    theme: ThemeConfig;
}

export interface FeaturesConfig {
    self_healing: boolean;
    log_streaming: boolean;
    os_updates_check: boolean;
}

export interface AppConfig {
    brand: BrandConfig;
    features: FeaturesConfig;
}

// Removed UpdateInfo — version status is now a string from server

export type View = "status" | "logs" | "updates" | "settings";
