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
    server_addr: string;
    ca_cert_path: string;
    client_cert_path: string;
    client_key_path: string;
    brand: BrandConfig;
    features: FeaturesConfig;
}

export type View = "status" | "updates" | "settings";
