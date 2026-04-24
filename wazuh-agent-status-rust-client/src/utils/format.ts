/**
 * Formats bytes to a human-readable string (e.g., 1024 -> "1.0 KB").
 */
export function formatBytes(bytes: number, decimals: number = 1): string {
    if (bytes === 0) return '0 B';

    const k = 1024;
    const dm = Math.max(0, decimals);
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return `${Number.parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}
