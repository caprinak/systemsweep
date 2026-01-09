// src/components/ui/Progress.tsx
import React from 'react';
import { clsx } from 'clsx';

interface ProgressProps {
    value: number;
    max?: number;
    size?: 'sm' | 'md' | 'lg';
    variant?: 'default' | 'success' | 'warning' | 'danger';
    showLabel?: boolean;
    animated?: boolean;
    className?: string;
}

export function Progress({
    value,
    max = 100,
    size = 'md',
    variant = 'default',
    showLabel = false,
    animated = false,
    className,
}: ProgressProps) {
    const percentage = Math.min(Math.max((value / max) * 100, 0), 100);

    const sizes = {
        sm: 'h-1',
        md: 'h-2',
        lg: 'h-3',
    };

    const variants = {
        default: 'bg-primary-500',
        success: 'bg-success-500',
        warning: 'bg-warning-500',
        danger: 'bg-danger-500',
    };

    return (
        <div className={className}>
            <div
                className={clsx(
                    'w-full bg-dark-200 dark:bg-dark-700 rounded-full overflow-hidden',
                    sizes[size]
                )}
            >
                <div
                    className={clsx(
                        'h-full rounded-full transition-all duration-300 ease-out',
                        variants[variant],
                        animated && 'animate-pulse'
                    )}
                    style={{ width: `${percentage}%` }}
                />
            </div>
            {showLabel && (
                <div className="mt-1 text-xs text-dark-500 dark:text-dark-400 text-right">
                    {percentage.toFixed(1)}%
                </div>
            )}
        </div>
    );
}

interface CircularProgressProps {
    value: number;
    max?: number;
    size?: number;
    strokeWidth?: number;
    variant?: 'default' | 'success' | 'warning' | 'danger';
    showLabel?: boolean;
    className?: string;
}

export function CircularProgress({
    value,
    max = 100,
    size = 120,
    strokeWidth = 8,
    variant = 'default',
    showLabel = true,
    className,
}: CircularProgressProps) {
    const percentage = Math.min(Math.max((value / max) * 100, 0), 100);
    const radius = (size - strokeWidth) / 2;
    const circumference = radius * 2 * Math.PI;
    const offset = circumference - (percentage / 100) * circumference;

    const variants = {
        default: 'text-primary-500',
        success: 'text-success-500',
        warning: 'text-warning-500',
        danger: 'text-danger-500',
    };

    return (
        <div className={clsx('relative inline-flex', className)}>
            <svg width={size} height={size} className="-rotate-90">
                <circle
                    cx={size / 2}
                    cy={size / 2}
                    r={radius}
                    fill="none"
                    stroke="currentColor"
                    strokeWidth={strokeWidth}
                    className="text-dark-200 dark:text-dark-700"
                />
                <circle
                    cx={size / 2}
                    cy={size / 2}
                    r={radius}
                    fill="none"
                    stroke="currentColor"
                    strokeWidth={strokeWidth}
                    strokeDasharray={circumference}
                    strokeDashoffset={offset}
                    strokeLinecap="round"
                    className={clsx('transition-all duration-500 ease-out', variants[variant])}
                />
            </svg>
            {showLabel && (
                <div className="absolute inset-0 flex items-center justify-center">
                    <span className="text-2xl font-bold text-dark-900 dark:text-dark-50">
                        {Math.round(percentage)}%
                    </span>
                </div>
            )}
        </div>
    );
}
