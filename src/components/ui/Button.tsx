// src/components/ui/Button.tsx
import React from 'react';
import { clsx } from 'clsx';
import { Loader2 } from 'lucide-react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: 'primary' | 'secondary' | 'danger' | 'ghost' | 'outline';
    size?: 'sm' | 'md' | 'lg';
    isLoading?: boolean;
    leftIcon?: React.ReactNode;
    rightIcon?: React.ReactNode;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
    (
        {
            className,
            variant = 'primary',
            size = 'md',
            isLoading = false,
            leftIcon,
            rightIcon,
            disabled,
            children,
            ...props
        },
        ref
    ) => {
        const baseStyles =
            'inline-flex items-center justify-center font-medium transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none rounded-lg';

        const variants = {
            primary:
                'bg-primary-600 text-white hover:bg-primary-700 focus:ring-primary-500 shadow-sm',
            secondary:
                'bg-dark-200 dark:bg-dark-700 text-dark-900 dark:text-dark-100 hover:bg-dark-300 dark:hover:bg-dark-600 focus:ring-dark-500',
            danger:
                'bg-danger-600 text-white hover:bg-danger-700 focus:ring-danger-500 shadow-sm',
            ghost:
                'text-dark-700 dark:text-dark-300 hover:bg-dark-100 dark:hover:bg-dark-800 focus:ring-dark-500',
            outline:
                'border-2 border-dark-300 dark:border-dark-600 text-dark-700 dark:text-dark-300 hover:bg-dark-100 dark:hover:bg-dark-800 focus:ring-dark-500',
        };

        const sizes = {
            sm: 'px-3 py-1.5 text-sm gap-1.5',
            md: 'px-4 py-2 text-sm gap-2',
            lg: 'px-6 py-3 text-base gap-2',
        };

        return (
            <button
                ref={ref}
                className={clsx(baseStyles, variants[variant], sizes[size], className)}
                disabled={disabled || isLoading}
                {...props}
            >
                {isLoading ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                    leftIcon
                )}
                {children}
                {rightIcon && !isLoading && rightIcon}
            </button>
        );
    }
);

Button.displayName = 'Button';
