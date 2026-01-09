// src/hooks/useTheme.ts
import { useEffect } from 'react';
import { useAppStore } from '../stores/appStore';

export function useTheme() {
    const { settings, updateSettings } = useAppStore();

    useEffect(() => {
        const root = window.document.documentElement;

        if (settings.theme === 'system') {
            const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
            const handleChange = () => {
                root.classList.toggle('dark', mediaQuery.matches);
            };
            handleChange();
            mediaQuery.addEventListener('change', handleChange);
            return () => mediaQuery.removeEventListener('change', handleChange);
        } else {
            root.classList.toggle('dark', settings.theme === 'dark');
        }
    }, [settings.theme]);

    const setTheme = (theme: 'light' | 'dark' | 'system') => {
        updateSettings({ theme });
    };

    return { theme: settings.theme, setTheme };
}
