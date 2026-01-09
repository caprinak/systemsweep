// src/hooks/useScanner.ts
import { useCallback, useEffect } from 'react';
import { useAppStore } from '../stores/appStore';
import * as api from '../utils/api';
import type { CleanupCategory } from '../types';
import { formatBytes } from '../utils/format';

export function useScanner() {
    const {
        isScanning,
        scanProgress,
        scanResults,
        setIsScanning,
        setScanProgress,
        setScanResults,
        setCategoryStats,
        addNotification,
    } = useAppStore();

    useEffect(() => {
        const unsubscribeProgress = api.onScanProgress((progress) => {
            setScanProgress(progress);
        });

        const unsubscribeComplete = api.onScanComplete((results) => {
            setScanResults(results);
            setIsScanning(false);
            setScanProgress(null);

            // Calculate category stats
            const statsMap = new Map<CleanupCategory, { count: number; size: number }>();
            results.forEach((result) => {
                const existing = statsMap.get(result.category) || { count: 0, size: 0 };
                statsMap.set(result.category, {
                    count: existing.count + 1,
                    size: existing.size + result.size,
                });
            });

            const categoryLabels: Record<CleanupCategory, { label: string; icon: string }> = {
                system_cache: { label: 'System Cache', icon: 'database' },
                browser_cache: { label: 'Browser Cache', icon: 'globe' },
                temp_files: { label: 'Temporary Files', icon: 'file-x' },
                log_files: { label: 'Log Files', icon: 'file-text' },
                thumbnails: { label: 'Thumbnails', icon: 'image' },
                duplicates: { label: 'Duplicates', icon: 'copy' },
                large_files: { label: 'Large Files', icon: 'hard-drive' },
                old_files: { label: 'Old Files', icon: 'clock' },
                trash: { label: 'Trash', icon: 'trash-2' },
                downloads: { label: 'Downloads', icon: 'download' },
            };

            const stats = Array.from(statsMap.entries()).map(([category, data]) => ({
                category,
                count: data.count,
                size: data.size,
                ...categoryLabels[category],
            }));

            setCategoryStats(stats);

            addNotification({
                type: 'success',
                title: 'Scan Complete',
                message: `Found ${results.length} items totaling ${formatBytes(
                    results.reduce((acc, r) => acc + r.size, 0)
                )}`,
            });
        });

        return () => {
            unsubscribeProgress.then((unsub) => unsub());
            unsubscribeComplete.then((unsub) => unsub());
        };
    }, []);

    const startScan = useCallback(
        async (categories: CleanupCategory[]) => {
            setIsScanning(true);
            setScanProgress({
                phase: 'initializing',
                currentPath: '',
                filesScanned: 0,
                issuesFound: 0,
                estimatedTime: 0,
                progress: 0,
            });

            try {
                await api.startScan(categories);
            } catch (error) {
                setIsScanning(false);
                setScanProgress(null);
                addNotification({
                    type: 'error',
                    title: 'Scan Failed',
                    message: error instanceof Error ? error.message : 'Unknown error occurred',
                });
            }
        },
        [setIsScanning, setScanProgress, addNotification]
    );

    const cancelScan = useCallback(async () => {
        try {
            await api.cancelScan();
            setIsScanning(false);
            setScanProgress(null);
            addNotification({
                type: 'info',
                title: 'Scan Cancelled',
                message: 'The scan was cancelled by user',
            });
        } catch (error) {
            console.error('Failed to cancel scan:', error);
        }
    }, [setIsScanning, setScanProgress, addNotification]);

    return {
        isScanning,
        scanProgress,
        scanResults,
        startScan,
        cancelScan,
    };
}
