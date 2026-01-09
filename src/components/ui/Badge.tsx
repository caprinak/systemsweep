// src/components/ui/Badge.tsx
import { clsx } from 'clsx';
import React from 'react';

interface BadgeProps {
    children: React.ReactNode;
    variant?: 'default' | 'success' | 'warning' | 'danger' | 'info';
    size?: 'sm' | 'md';
    className?: string;
}

export function Badge({
    children,
    variant = 'default',
    size = 'md',
    className,
}: BadgeProps) {
    const variants = {
        default: 'bg-dark-200 dark:bg-dark-700 text-dark-700 dark:text-dark-300',
        success: 'bg-success-100 dark:bg-success-500/20 text-success-700 dark:text-success-400',
        warning: 'bg-warning-100 dark:bg-warning-500/20 text-warning-700 dark:text-warning-400',
        danger: 'bg-danger-100 dark:bg-danger-500/20 text-danger-700 dark:text-danger-400',
        info: 'bg-primary-100 dark:bg-primary-500/20 text-primary-700 dark:text-primary-400',
    };

    const sizes = {
        sm: 'px-2 py-0.5 text-xs',
        md: 'px-2.5 py-1 text-sm',
    };

    return (
        <span
            className={clsx(
                'inline-flex items-center font-medium rounded-full',
                variants[variant],
                sizes[size],
                className
            )}
        >
            {children}
        </span>
    );
}
