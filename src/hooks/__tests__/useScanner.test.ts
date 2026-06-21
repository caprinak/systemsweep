import { vi, describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useScanner } from '../useScanner';
import { useAppStore } from '../../stores/appStore';
import * as api from '../../utils/api';

// Mock the API layer entirely
vi.mock('../../utils/api', () => {
    let progressCallback: any = null;
    let completeCallback: any = null;
    
    return {
        startScan: vi.fn().mockResolvedValue('scan_id'),
        cancelScan: vi.fn().mockResolvedValue(undefined),
        onScanProgress: vi.fn().mockImplementation((cb) => {
            progressCallback = cb;
            return Promise.resolve(vi.fn());
        }),
        onScanComplete: vi.fn().mockImplementation((cb) => {
            completeCallback = cb;
            return Promise.resolve(vi.fn());
        }),
        __triggerProgress: (data: any) => {
            if (progressCallback) progressCallback(data);
        },
        __triggerComplete: (data: any) => {
            if (completeCallback) completeCallback(data);
        }
    };
});

describe('useScanner hook', () => {
    beforeEach(() => {
        useAppStore.getState().reset();
        useAppStore.getState().clearNotifications();
        vi.clearAllMocks();
    });

    it('should initialize scanner listeners on mount', () => {
        renderHook(() => useScanner());
        expect(api.onScanProgress).toHaveBeenCalled();
        expect(api.onScanComplete).toHaveBeenCalled();
    });

    it('should trigger startScan API and transition states correctly', async () => {
        const { result } = renderHook(() => useScanner());

        expect(result.current.isScanning).toBe(false);

        await act(async () => {
            await result.current.startScan(['temp_files', 'log_files']);
        });

        expect(result.current.isScanning).toBe(true);
        expect(api.startScan).toHaveBeenCalledWith(['temp_files', 'log_files']);
        expect(useAppStore.getState().scanProgress?.phase).toBe('initializing');
    });

    it('should handle progress events pushed from backend', async () => {
        renderHook(() => useScanner());

        const progressData = {
            phase: 'scanning',
            currentPath: 'C:\\temp\\file.txt',
            filesScanned: 42,
            issuesFound: 5,
            estimatedTime: 120,
            progress: 50,
        };

        await act(async () => {
            (api as any).__triggerProgress(progressData);
        });

        expect(useAppStore.getState().scanProgress).toEqual(progressData);
    });

    it('should process results and calculate statistics on scan completion', async () => {
        const { result } = renderHook(() => useScanner());

        await act(async () => {
            await result.current.startScan(['temp_files']);
        });

        expect(result.current.isScanning).toBe(true);

        const mockResults = [
            { id: '1', path: 'file1.tmp', size: 1024, category: 'temp_files' },
            { id: '2', path: 'file2.tmp', size: 2048, category: 'temp_files' },
        ];

        await act(async () => {
            (api as any).__triggerComplete(mockResults);
        });

        expect(result.current.isScanning).toBe(false);
        expect(result.current.scanResults).toEqual(mockResults);
        
        // Category stats should be aggregated
        const stats = useAppStore.getState().categoryStats;
        expect(stats.length).toBe(1);
        expect(stats[0].category).toBe('temp_files');
        expect(stats[0].count).toBe(2);
        expect(stats[0].size).toBe(3072);
        
        // Success notification should be added
        const notifications = useAppStore.getState().notifications;
        expect(notifications.length).toBe(1);
        expect(notifications[0].title).toBe('Scan Complete');
    });

    it('should trigger cancelScan API and reset states on cancellation', async () => {
        const { result } = renderHook(() => useScanner());

        await act(async () => {
            await result.current.startScan(['temp_files']);
        });

        expect(result.current.isScanning).toBe(true);

        await act(async () => {
            await result.current.cancelScan();
        });

        expect(result.current.isScanning).toBe(false);
        expect(api.cancelScan).toHaveBeenCalled();
        expect(result.current.scanProgress).toBeNull();
    });
});
