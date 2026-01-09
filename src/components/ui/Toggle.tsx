// src/components/ui/Toggle.tsx
import { clsx } from 'clsx';
import React from 'react';

interface ToggleProps {
    checked: boolean;
    onChange: (checked: boolean) => void;
    disabled?: boolean;
    size?: 'sm' | 'md' | 'lg';
    label?: string;
}

export function Toggle({
    checked,
    onChange,
    disabled = false,
    size = 'md',
    label,
}: ToggleProps) {
    const sizes = {
        sm: { track: 'w-8 h-4', thumb: 'w-3 h-3', translate: 'translate-x-4' },
        md: { track: 'w-11 h-6', thumb: 'w-5 h-5', translate: 'translate-x-5' },
        lg: { track: 'w-14 h-7', thumb: 'w-6 h-6', translate: 'translate-x-7' },
    };

    const { track, thumb, translate } = sizes[size];

    return (
        <label className="inline-flex items-center gap-3 cursor-pointer">
            <button
                type="button"
                role="switch"
                aria-checked={checked}
                disabled={disabled}
                onClick={() => onChange(!checked)}
                className={clsx(
                    'relative inline-flex shrink-0 rounded-full transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2',
                    track,
                    checked ? 'bg-primary-600' : 'bg-dark-300 dark:bg-dark-600',
                    disabled && 'opacity-50 cursor-not-allowed'
                )}
            >
                <span
                    className={clsx(
                        'pointer-events-none inline-block rounded-full bg-white shadow-lg transform ring-0 transition duration-200 ease-in-out',
                        thumb,
                        checked ? translate : 'translate-x-0.5',
                        'mt-0.5 ml-0.5'
                    )}
                />
            </button>
            {label && (
                <span className="text-sm text-dark-700 dark:text-dark-300">{label}</span>
            )}
        </label>
    );
}
