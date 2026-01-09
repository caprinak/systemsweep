// src/utils/api.ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
    ScanResult,
    DuplicateGroup,
    StartupItem,
    SystemHealth,
    CleanupSession,
    RestorePoint,
    ScanProgress,
    LargeFile,
    CleanupCategory
} from '../types';

// System Health
export async function getSystemHealth(): Promise<SystemHealth> {
    return invoke('get_system_health');
}

// Scanning
export async function startScan(categories: CleanupCategory[]): Promise<string> {
    return invoke('start_scan', { categories });
}

export async function cancelScan(): Promise<void> {
    return invoke('cancel_scan');
}

export function onScanProgress(callback: (progress: ScanProgress) => void) {
    return listen<ScanProgress>('scan-progress', (event) => {
        callback(event.payload);
    });
}

export function onScanComplete(callback: (results: ScanResult[]) => void) {
    return listen<ScanResult[]>('scan-complete', (event) => {
        callback(event.payload);
    });
}

// Cleanup
export async function performCleanup(
    items: string[],
    dryRun: boolean,
    secureDelete: boolean
): Promise<CleanupSession> {
    return invoke('perform_cleanup', { items, dryRun, secureDelete });
}

export async function cancelCleanup(): Promise<void> {
    return invoke('cancel_cleanup');
}

export function onCleanupProgress(
    callback: (progress: { current: number; total: number; currentFile: string }) => void
) {
    return listen('cleanup-progress', (event) => {
        callback(event.payload as any);
    });
}

// Duplicates
export async function findDuplicates(paths?: string[]): Promise<DuplicateGroup[]> {
    return invoke('find_duplicates', { paths });
}

export async function deleteDuplicates(
    fileIds: string[],
    keepOriginals: boolean
): Promise<{ deleted: number; freed: number }> {
    return invoke('delete_duplicates', { fileIds, keepOriginals });
}

// Large Files
export async function findLargeFiles(
    minSize: number,
    paths?: string[]
): Promise<LargeFile[]> {
    return invoke('find_large_files', { minSize, paths });
}

// Old Files
export async function findOldFiles(
    maxAge: number,
    paths?: string[]
): Promise<ScanResult[]> {
    return invoke('find_old_files', { maxAge, paths });
}

// Startup Items
export async function getStartupItems(): Promise<StartupItem[]> {
    return invoke('get_startup_items');
}

export async function toggleStartupItem(
    id: string,
    enabled: boolean
): Promise<boolean> {
    return invoke('toggle_startup_item', { id, enabled });
}

export async function deleteStartupItem(id: string): Promise<boolean> {
    return invoke('delete_startup_item', { id });
}

// Restore Points
export async function getRestorePoints(): Promise<RestorePoint[]> {
    return invoke('get_restore_points');
}

export async function createRestorePoint(
    items: string[],
    description: string
): Promise<RestorePoint> {
    return invoke('create_restore_point', { items, description });
}

export async function restoreFromPoint(
    pointId: string,
    itemIds?: string[]
): Promise<{ restored: number; failed: number }> {
    return invoke('restore_from_point', { pointId, itemIds });
}

export async function deleteRestorePoint(pointId: string): Promise<boolean> {
    return invoke('delete_restore_point', { pointId });
}

// File Operations
export async function openInExplorer(path: string): Promise<void> {
    return invoke('open_in_explorer', { path });
}

export async function moveToTrash(paths: string[]): Promise<number> {
    return invoke('move_to_trash', { paths });
}

export async function secureDelete(paths: string[]): Promise<number> {
    return invoke('secure_delete', { paths });
}

// Settings
export async function getExcludedPaths(): Promise<string[]> {
    return invoke('get_excluded_paths');
}

export async function addExcludedPath(path: string): Promise<void> {
    return invoke('add_excluded_path', { path });
}

export async function removeExcludedPath(path: string): Promise<void> {
    return invoke('remove_excluded_path', { path });
}

// Scheduling
export async function scheduleCleanup(
    interval: 'daily' | 'weekly' | 'monthly',
    time: string,
    categories: CleanupCategory[]
): Promise<void> {
    return invoke('schedule_cleanup', { interval, time, categories });
}

export async function cancelScheduledCleanup(): Promise<void> {
    return invoke('cancel_scheduled_cleanup');
}

// Analytics
export async function getCleanupHistory(): Promise<CleanupSession[]> {
    return invoke('get_cleanup_history');
}

export async function getStorageAnalytics(): Promise<{
    byCategory: { category: string; size: number }[];
    byType: { type: string; size: number }[];
    timeline: { date: string; freed: number }[];
}> {
    return invoke('get_storage_analytics');
}
