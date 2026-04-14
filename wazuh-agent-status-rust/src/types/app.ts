export interface ThemeConfig {
    primary_color: string;
    secondary_color: string;
    dark_mode: boolean;
}

export interface BrandConfig {
    name: string;
    company: string;
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

export interface UpdateInfo {
    current_version: string;
    latest_version: string;
    update_available: boolean;
    download_url: string;
}

export type View = "status" | "updates" | "settings";
