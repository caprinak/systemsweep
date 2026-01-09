// src/hooks/useSystemHealth.ts
import { useCallback, useEffect } from 'react';
import { useAppStore } from '../stores/appStore';
import * as api from '../utils/api';

export function useSystemHealth() {
    const { systemHealth, setSystemHealth } = useAppStore();

    const fetchHealth = useCallback(async () => {
        try {
            const health = await api.getSystemHealth();
            setSystemHealth(health);
        } catch (error) {
            console.error('Failed to fetch system health:', error);
        }
    }, [setSystemHealth]);

    useEffect(() => {
        fetchHealth();
        const interval = setInterval(fetchHealth, 30000); // Refresh every 30 seconds
        return () => clearInterval(interval);
    }, [fetchHealth]);

    return { systemHealth, refreshHealth: fetchHealth };
}
