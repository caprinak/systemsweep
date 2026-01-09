// src/types/index.ts
export interface ScanResult {
    id: string;
    path: string;
    name: string;
    size: number;
    fileType: FileType;
    category: CleanupCategory;
    lastAccessed: string;
    lastModified: string;
    isSelected: boolean;
    risk: RiskLevel;
    description: string;
}

export type FileType =
    | 'cache'
    | 'temp'
    | 'log'
    | 'thumbnail'
    | 'duplicate'
    | 'large'
    | 'old'
    | 'browser'
    | 'system'
    | 'trash';

export type CleanupCategory =
    | 'system_cache'
    | 'browser_cache'
    | 'temp_files'
    | 'log_files'
    | 'thumbnails'
    | 'duplicates'
    | 'large_files'
    | 'old_files'
    | 'trash'
    | 'downloads';

export type RiskLevel = 'safe' | 'low' | 'medium' | 'high';

export interface DuplicateGroup {
    hash: string;
    files: DuplicateFile[];
    totalSize: number;
    potentialSavings: number;
}

export interface DuplicateFile {
    id: string;
    path: string;
    name: string;
    size: number;
    modified: string;
    isOriginal: boolean;
    isSelected: boolean;
}

export interface StartupItem {
    id: string;
    name: string;
    path: string;
    enabled: boolean;
    impact: 'low' | 'medium' | 'high';
    type: 'app' | 'service' | 'scheduled';
    description?: string;
}

export interface LargeFile {
    id: string;
    path: string;
    name: string;
    size: number;
    extension: string;
    lastAccessed: string;
    isSelected: boolean;
}

export interface SystemHealth {
    diskUsage: DiskUsage;
    memoryUsage: MemoryUsage;
    cleanableSpace: number;
    lastScan: string | null;
    issuesFound: number;
}

export interface DiskUsage {
    total: number;
    used: number;
    free: number;
    mountPoint: string;
}

export interface MemoryUsage {
    total: number;
    used: number;
    free: number;
    cached: number;
}

export interface CleanupSession {
    id: string;
    startTime: string;
    endTime?: string;
    filesScanned: number;
    filesDeleted: number;
    spaceFreed: number;
    status: 'scanning' | 'cleaning' | 'completed' | 'cancelled' | 'error';
    items: CleanupItem[];
}

export interface CleanupItem {
    id: string;
    path: string;
    size: number;
    status: 'pending' | 'deleted' | 'failed' | 'skipped';
    error?: string;
    backupPath?: string;
}

export interface RestorePoint {
    id: string;
    timestamp: string;
    description: string;
    itemCount: number;
    totalSize: number;
    items: RestoreItem[];
}

export interface RestoreItem {
    originalPath: string;
    backupPath: string;
    size: number;
    restored: boolean;
}

export interface ScanProgress {
    phase: 'initializing' | 'scanning' | 'analyzing' | 'completed';
    currentPath: string;
    filesScanned: number;
    issuesFound: number;
    estimatedTime: number;
    progress: number;
}

export interface Settings {
    theme: 'light' | 'dark' | 'system';
    autoScan: boolean;
    scanInterval: number;
    dryRunDefault: boolean;
    createRestorePoints: boolean;
    maxRestorePoints: number;
    secureDelete: boolean;
    notifications: boolean;
    minimizeToTray: boolean;
    startWithSystem: boolean;
    language: string;
    excludedPaths: string[];
    largeFileThreshold: number;
    oldFileThreshold: number;
}

export interface Notification {
    id: string;
    type: 'info' | 'success' | 'warning' | 'error';
    title: string;
    message: string;
    timestamp: string;
    read: boolean;
}
