// src/components/ui/Card.tsx
import React from 'react';
import { clsx } from 'clsx';

interface CardProps {
    children: React.ReactNode;
    className?: string;
    padding?: 'none' | 'sm' | 'md' | 'lg';
    hover?: boolean;
}

export function Card({
    children,
    className,
    padding = 'md',
    hover = false,
}: CardProps) {
    const paddings = {
        none: '',
        sm: 'p-3',
        md: 'p-4',
        lg: 'p-6',
    };

    return (
        <div
            className={clsx(
                'bg-white dark:bg-dark-900 rounded-xl border border-dark-200 dark:border-dark-800',
                paddings[padding],
                hover && 'transition-shadow hover:shadow-lg cursor-pointer',
                className
            )}
        >
            {children}
        </div>
    );
}

interface CardHeaderProps {
    children: React.ReactNode;
    className?: string;
    action?: React.ReactNode;
}

export function CardHeader({ children, className, action }: CardHeaderProps) {
    return (
        <div
            className={clsx(
                'flex items-center justify-between mb-4',
                className
            )}
        >
            <div>{children}</div>
            {action && <div>{action}</div>}
        </div>
    );
}

export function CardTitle({
    children,
    className,
}: {
    children: React.ReactNode;
    className?: string;
}) {
    return (
        <h3 className={clsx('text-lg font-semibold text-dark-900 dark:text-dark-50', className)}>
            {children}
        </h3>
    );
}

export function CardDescription({
    children,
    className,
}: {
    children: React.ReactNode;
    className?: string;
}) {
    return (
        <p className={clsx('text-sm text-dark-500 dark:text-dark-400 mt-1', className)}>
            {children}
        </p>
    );
}
