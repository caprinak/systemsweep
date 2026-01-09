// src/hooks/useCleanup.ts
import { useCallback, useEffect, useState } from 'react';
import { useAppStore } from '../stores/appStore';
import * as api from '../utils/api';
import { formatBytes } from '../utils/format';

interface CleanupProgress {
    current: number;
    total: number;
    currentFile: string;
}

export function useCleanup() {
    const {
        scanResults,
        isCleaning,
        isDryRun,
        settings,
        setIsCleaning,
        setCurrentSession,
        addRestorePoint,
        addNotification,
        deselectAllResults,
    } = useAppStore();

    const [cleanupProgress, setCleanupProgress] = useState<CleanupProgress | null>(null);

    useEffect(() => {
        const unsubscribe = api.onCleanupProgress((progress) => {
            setCleanupProgress(progress);
        });

        return () => {
            unsubscribe.then((unsub) => unsub());
        };
    }, []);

    const selectedItems = scanResults.filter((r) => r.isSelected);
    const selectedSize = selectedItems.reduce((acc, r) => acc + r.size, 0);

    const performCleanup = useCallback(async () => {
        if (selectedItems.length === 0) {
            addNotification({
                type: 'warning',
                title: 'No Items Selected',
                message: 'Please select items to clean up',
            });
            return;
        }

        setIsCleaning(true);
        setCleanupProgress({ current: 0, total: selectedItems.length, currentFile: '' });

        try {
            // Create restore point if enabled
            if (settings.createRestorePoints && !isDryRun) {
                const restorePoint = await api.createRestorePoint(
                    selectedItems.map((i) => i.id),
                    `Cleanup session - ${new Date().toLocaleString()}`
                );
                addRestorePoint(restorePoint);
            }

            const session = await api.performCleanup(
                selectedItems.map((i) => i.id),
                isDryRun,
                settings.secureDelete
            );

            setCurrentSession(session);
            deselectAllResults();

            addNotification({
                type: 'success',
                title: isDryRun ? 'Dry Run Complete' : 'Cleanup Complete',
                message: isDryRun
                    ? `Would free ${formatBytes(selectedSize)} from ${selectedItems.length} items`
                    : `Freed ${formatBytes(session.spaceFreed)} from ${session.filesDeleted} files`,
            });
        } catch (error) {
            addNotification({
                type: 'error',
                title: 'Cleanup Failed',
                message: error instanceof Error ? error.message : 'Unknown error occurred',
            });
        } finally {
            setIsCleaning(false);
            setCleanupProgress(null);
        }
    }, [
        selectedItems,
        isDryRun,
        settings,
        setIsCleaning,
        setCurrentSession,
        addRestorePoint,
        addNotification,
        deselectAllResults,
        selectedSize
    ]);

    const cancelCleanup = useCallback(async () => {
        try {
            await api.cancelCleanup();
            setIsCleaning(false);
            setCleanupProgress(null);
            addNotification({
                type: 'info',
                title: 'Cleanup Cancelled',
                message: 'The cleanup operation was cancelled',
            });
        } catch (error) {
            console.error('Failed to cancel cleanup:', error);
        }
    }, [setIsCleaning, addNotification]);

    return {
        isCleaning,
        isDryRun,
        cleanupProgress,
        selectedItems,
        selectedSize,
        performCleanup,
        cancelCleanup,
    };
}
