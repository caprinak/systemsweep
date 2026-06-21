import { describe, it, expect, beforeEach } from 'vitest';
import { useAppStore } from '../appStore';

describe('appStore', () => {
    beforeEach(() => {
        useAppStore.getState().reset();
        useAppStore.getState().clearNotifications();
    });

    it('should have correct default state', () => {
        const state = useAppStore.getState();
        expect(state.isScanning).toBe(false);
        expect(state.scanResults).toEqual([]);
        expect(state.notifications).toEqual([]);
        expect(state.settings.theme).toBe('system');
        expect(state.settings.largeFileThreshold).toBe(100 * 1024 * 1024);
    });

    it('should handle settings updates', () => {
        useAppStore.getState().updateSettings({ theme: 'dark', secureDelete: true });
        const state = useAppStore.getState();
        expect(state.settings.theme).toBe('dark');
        expect(state.settings.secureDelete).toBe(true);
    });

    it('should manage notifications', () => {
        useAppStore.getState().addNotification({
            type: 'success',
            title: 'Scan Done',
            message: 'All files checked',
        });

        let state = useAppStore.getState();
        expect(state.notifications.length).toBe(1);
        expect(state.notifications[0].title).toBe('Scan Done');
        expect(state.notifications[0].read).toBe(false);

        const id = state.notifications[0].id;
        useAppStore.getState().markNotificationRead(id);
        state = useAppStore.getState();
        expect(state.notifications[0].read).toBe(true);

        useAppStore.getState().clearNotifications();
        state = useAppStore.getState();
        expect(state.notifications.length).toBe(0);
    });

    it('should manage selection status of scan results', () => {
        const mockResults = [
            { id: '1', name: 'a.tmp', size: 100, category: 'temp_files' as any, isSelected: false },
            { id: '2', name: 'b.log', size: 200, category: 'log_files' as any, isSelected: false },
            { id: '3', name: 'c.tmp', size: 150, category: 'temp_files' as any, isSelected: false },
        ];
        useAppStore.getState().setScanResults(mockResults as any);

        let state = useAppStore.getState();
        expect(state.scanResults.length).toBe(3);

        // toggle one
        useAppStore.getState().toggleResultSelection('1');
        state = useAppStore.getState();
        expect(state.scanResults.find(r => r.id === '1')?.isSelected).toBe(true);
        expect(state.scanResults.find(r => r.id === '2')?.isSelected).toBe(false);

        // select all for a specific category
        useAppStore.getState().selectAllResults('temp_files' as any);
        state = useAppStore.getState();
        expect(state.scanResults.find(r => r.id === '1')?.isSelected).toBe(true);
        expect(state.scanResults.find(r => r.id === '2')?.isSelected).toBe(false);
        expect(state.scanResults.find(r => r.id === '3')?.isSelected).toBe(true);

        // select all overall
        useAppStore.getState().selectAllResults();
        state = useAppStore.getState();
        expect(state.scanResults.every(r => r.isSelected)).toBe(true);

        // deselect all
        useAppStore.getState().deselectAllResults();
        state = useAppStore.getState();
        expect(state.scanResults.every(r => !r.isSelected)).toBe(true);
    });
});
