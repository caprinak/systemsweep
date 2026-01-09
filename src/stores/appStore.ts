// src/stores/appStore.ts
import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type {
    ScanResult,
    DuplicateGroup,
    StartupItem,
    LargeFile,
    SystemHealth,
    CleanupSession,
    RestorePoint,
    ScanProgress,
    Settings,
    Notification,
    CleanupCategory
} from '../types';

interface CategoryStats {
    category: CleanupCategory;
    count: number;
    size: number;
    label: string;
    icon: string;
}

interface AppState {
    // UI State
    sidebarCollapsed: boolean;
    currentView: string;
    isScanning: boolean;
    isCleaning: boolean;
    isDryRun: boolean;

    // Data
    scanResults: ScanResult[];
    duplicates: DuplicateGroup[];
    startupItems: StartupItem[];
    largeFiles: LargeFile[];
    systemHealth: SystemHealth | null;
    currentSession: CleanupSession | null;
    restorePoints: RestorePoint[];
    scanProgress: ScanProgress | null;
    notifications: Notification[];
    categoryStats: CategoryStats[];

    // Settings
    settings: Settings;

    // Actions
    setSidebarCollapsed: (collapsed: boolean) => void;
    setCurrentView: (view: string) => void;
    setIsScanning: (scanning: boolean) => void;
    setIsCleaning: (cleaning: boolean) => void;
    setIsDryRun: (dryRun: boolean) => void;
    setScanResults: (results: ScanResult[]) => void;
    setDuplicates: (duplicates: DuplicateGroup[]) => void;
    setStartupItems: (items: StartupItem[]) => void;
    setLargeFiles: (files: LargeFile[]) => void;
    setSystemHealth: (health: SystemHealth) => void;
    setCurrentSession: (session: CleanupSession | null) => void;
    addRestorePoint: (point: RestorePoint) => void;
    setScanProgress: (progress: ScanProgress | null) => void;
    addNotification: (notification: Omit<Notification, 'id' | 'timestamp' | 'read'>) => void;
    markNotificationRead: (id: string) => void;
    clearNotifications: () => void;
    updateSettings: (settings: Partial<Settings>) => void;
    toggleResultSelection: (id: string) => void;
    selectAllResults: (category?: CleanupCategory) => void;
    deselectAllResults: () => void;
    setCategoryStats: (stats: CategoryStats[]) => void;
    reset: () => void;
}

const defaultSettings: Settings = {
    theme: 'system',
    autoScan: false,
    scanInterval: 7,
    dryRunDefault: true,
    createRestorePoints: true,
    maxRestorePoints: 10,
    secureDelete: false,
    notifications: true,
    minimizeToTray: true,
    startWithSystem: false,
    language: 'en',
    excludedPaths: [],
    largeFileThreshold: 100 * 1024 * 1024, // 100MB
    oldFileThreshold: 90, // days
};

export const useAppStore = create<AppState>()(
    persist(
        (set, get) => ({
            // Initial UI State
            sidebarCollapsed: false,
            currentView: 'dashboard',
            isScanning: false,
            isCleaning: false,
            isDryRun: true,

            // Initial Data
            scanResults: [],
            duplicates: [],
            startupItems: [],
            largeFiles: [],
            systemHealth: null,
            currentSession: null,
            restorePoints: [],
            scanProgress: null,
            notifications: [],
            categoryStats: [],

            // Settings
            settings: defaultSettings,

            // Actions
            setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),

            setCurrentView: (view) => set({ currentView: view }),

            setIsScanning: (scanning) => set({ isScanning: scanning }),

            setIsCleaning: (cleaning) => set({ isCleaning: cleaning }),

            setIsDryRun: (dryRun) => set({ isDryRun: dryRun }),

            setScanResults: (results) => set({ scanResults: results }),

            setDuplicates: (duplicates) => set({ duplicates }),

            setStartupItems: (items) => set({ startupItems: items }),

            setLargeFiles: (files) => set({ largeFiles: files }),

            setSystemHealth: (health) => set({ systemHealth: health }),

            setCurrentSession: (session) => set({ currentSession: session }),

            addRestorePoint: (point) => set((state) => {
                const points = [point, ...state.restorePoints];
                if (points.length > state.settings.maxRestorePoints) {
                    points.pop();
                }
                return { restorePoints: points };
            }),

            setScanProgress: (progress) => set({ scanProgress: progress }),

            addNotification: (notification) => set((state) => ({
                notifications: [
                    {
                        ...notification,
                        id: crypto.randomUUID(),
                        timestamp: new Date().toISOString(),
                        read: false,
                    },
                    ...state.notifications,
                ].slice(0, 50),
            })),

            markNotificationRead: (id) => set((state) => ({
                notifications: state.notifications.map((n) =>
                    n.id === id ? { ...n, read: true } : n
                ),
            })),

            clearNotifications: () => set({ notifications: [] }),

            updateSettings: (newSettings) => set((state) => ({
                settings: { ...state.settings, ...newSettings },
            })),

            toggleResultSelection: (id) => set((state) => ({
                scanResults: state.scanResults.map((r) =>
                    r.id === id ? { ...r, isSelected: !r.isSelected } : r
                ),
            })),

            selectAllResults: (category) => set((state) => ({
                scanResults: state.scanResults.map((r) =>
                    category ? (r.category === category ? { ...r, isSelected: true } : r) : { ...r, isSelected: true }
                ),
            })),

            deselectAllResults: () => set((state) => ({
                scanResults: state.scanResults.map((r) => ({ ...r, isSelected: false })),
            })),

            setCategoryStats: (stats) => set({ categoryStats: stats }),

            reset: () => set({
                scanResults: [],
                duplicates: [],
                largeFiles: [],
                currentSession: null,
                scanProgress: null,
                isScanning: false,
                isCleaning: false,
            }),
        }),
        {
            name: 'desktop-cleaner-storage',
            partialize: (state) => ({
                settings: state.settings,
                restorePoints: state.restorePoints,
                sidebarCollapsed: state.sidebarCollapsed,
            }),
        }
    )
);
