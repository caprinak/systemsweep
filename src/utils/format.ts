// src/utils/format.ts
export function formatBytes(bytes: number, decimals = 2): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`;
}

export function formatNumber(num: number): string {
    return new Intl.NumberFormat().format(num);
}

export function formatDate(date: string | Date): string {
    return new Intl.DateTimeFormat('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    }).format(new Date(date));
}

export function formatRelativeTime(date: string | Date): string {
    const now = new Date();
    const then = new Date(date);
    const diffMs = now.getTime() - then.getTime();
    const diffSecs = Math.floor(diffMs / 1000);
    const diffMins = Math.floor(diffSecs / 60);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffDays > 30) {
        return formatDate(date);
    } else if (diffDays > 0) {
        return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
    } else if (diffHours > 0) {
        return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
    } else if (diffMins > 0) {
        return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
    } else {
        return 'Just now';
    }
}

export function formatDuration(seconds: number): string {
    if (seconds < 60) {
        return `${seconds}s`;
    } else if (seconds < 3600) {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins}m ${secs}s`;
    } else {
        const hours = Math.floor(seconds / 3600);
        const mins = Math.floor((seconds % 3600) / 60);
        return `${hours}h ${mins}m`;
    }
}

export function formatPercentage(value: number, total: number): string {
    if (total === 0) return '0%';
    return `${((value / total) * 100).toFixed(1)}%`;
}

export function truncatePath(path: string, maxLength = 50): string {
    if (path.length <= maxLength) return path;
    const parts = path.split(/[/\\]/);
    if (parts.length <= 3) return path;
    return `${parts[0]}/.../${parts.slice(-2).join('/')}`;
}

export function getFileExtension(filename: string): string {
    const parts = filename.split('.');
    return parts.length > 1 ? parts.pop()?.toLowerCase() || '' : '';
}

export function getFileIcon(extension: string): string {
    const iconMap: Record<string, string> = {
        // Images
        jpg: 'image',
        jpeg: 'image',
        png: 'image',
        gif: 'image',
        svg: 'image',
        webp: 'image',
        // Documents
        pdf: 'file-text',
        doc: 'file-text',
        docx: 'file-text',
        txt: 'file-text',
        // Archives
        zip: 'archive',
        rar: 'archive',
        '7z': 'archive',
        tar: 'archive',
        gz: 'archive',
        // Video
        mp4: 'video',
        avi: 'video',
        mkv: 'video',
        mov: 'video',
        // Audio
        mp3: 'music',
        wav: 'music',
        flac: 'music',
        // Code
        js: 'code',
        ts: 'code',
        py: 'code',
        rs: 'code',
        // Default
        default: 'file',
    };
    return iconMap[extension] || iconMap.default;
}
