// src/components/layout/Header.tsx
import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
    Bell,
    Moon,
    Sun,
    Monitor,
    Search,
    Settings,
    HelpCircle,
} from 'lucide-react';
import { useAppStore } from '../../stores/appStore';
import { useTheme } from '../../hooks/useTheme';
import { Badge } from '../ui/Badge';
import { formatRelativeTime } from '../../utils/format';

export function Header() {
    const { notifications, markNotificationRead, clearNotifications } = useAppStore();
    const { theme, setTheme } = useTheme();
    const [showNotifications, setShowNotifications] = useState(false);
    const [showThemeMenu, setShowThemeMenu] = useState(false);

    const unreadCount = notifications.filter((n) => !n.read).length;

    const themeOptions = [
        { value: 'light', label: 'Light', icon: <Sun className="w-4 h-4" /> },
        { value: 'dark', label: 'Dark', icon: <Moon className="w-4 h-4" /> },
        { value: 'system', label: 'System', icon: <Monitor className="w-4 h-4" /> },
    ] as const;

    return (
        <header className="h-16 bg-white dark:bg-dark-900 border-b border-dark-200 dark:border-dark-800 flex items-center justify-between px-6">
            {/* Search */}
            <div className="flex-1 max-w-md">
                <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-dark-400" />
                    <input
                        type="text"
                        placeholder="Search files, settings..."
                        className="w-full pl-10 pr-4 py-2 bg-dark-100 dark:bg-dark-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 border-0"
                    />
                </div>
            </div>

            {/* Actions */}
            <div className="flex items-center gap-2">
                {/* Theme Toggle */}
                <div className="relative">
                    <button
                        onClick={() => setShowThemeMenu(!showThemeMenu)}
                        className="p-2 rounded-lg text-dark-500 hover:bg-dark-100 dark:hover:bg-dark-800 transition-colors"
                    >
                        {theme === 'dark' ? (
                            <Moon className="w-5 h-5" />
                        ) : theme === 'light' ? (
                            <Sun className="w-5 h-5" />
                        ) : (
                            <Monitor className="w-5 h-5" />
                        )}
                    </button>
                    <AnimatePresence>
                        {showThemeMenu && (
                            <motion.div
                                initial={{ opacity: 0, y: 10 }}
                                animate={{ opacity: 1, y: 0 }}
                                exit={{ opacity: 0, y: 10 }}
                                className="absolute right-0 mt-2 w-40 bg-white dark:bg-dark-800 rounded-lg shadow-lg border border-dark-200 dark:border-dark-700 overflow-hidden z-50"
                            >
                                {themeOptions.map((option) => (
                                    <button
                                        key={option.value}
                                        onClick={() => {
                                            setTheme(option.value);
                                            setShowThemeMenu(false);
                                        }}
                                        className={`w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-dark-100 dark:hover:bg-dark-700 transition-colors ${theme === option.value
                                                ? 'text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-500/10'
                                                : 'text-dark-700 dark:text-dark-300'
                                            }`}
                                    >
                                        {option.icon}
                                        {option.label}
                                    </button>
                                ))}
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>

                {/* Notifications */}
                <div className="relative">
                    <button
                        onClick={() => setShowNotifications(!showNotifications)}
                        className="relative p-2 rounded-lg text-dark-500 hover:bg-dark-100 dark:hover:bg-dark-800 transition-colors"
                    >
                        <Bell className="w-5 h-5" />
                        {unreadCount > 0 && (
                            <span className="absolute top-1 right-1 w-4 h-4 bg-danger-500 text-white text-xs rounded-full flex items-center justify-center">
                                {unreadCount > 9 ? '9+' : unreadCount}
                            </span>
                        )}
                    </button>
                    <AnimatePresence>
                        {showNotifications && (
                            <motion.div
                                initial={{ opacity: 0, y: 10 }}
                                animate={{ opacity: 1, y: 0 }}
                                exit={{ opacity: 0, y: 10 }}
                                className="absolute right-0 mt-2 w-80 bg-white dark:bg-dark-800 rounded-xl shadow-lg border border-dark-200 dark:border-dark-700 overflow-hidden z-50"
                            >
                                <div className="flex items-center justify-between px-4 py-3 border-b border-dark-200 dark:border-dark-700">
                                    <h3 className="font-semibold text-dark-900 dark:text-dark-50">
                                        Notifications
                                    </h3>
                                    {notifications.length > 0 && (
                                        <button
                                            onClick={clearNotifications}
                                            className="text-xs text-primary-600 hover:text-primary-700"
                                        >
                                            Clear all
                                        </button>
                                    )}
                                </div>
                                <div className="max-h-96 overflow-y-auto">
                                    {notifications.length === 0 ? (
                                        <div className="p-4 text-center text-dark-500">
                                            No notifications
                                        </div>
                                    ) : (
                                        notifications.slice(0, 10).map((notification) => (
                                            <div
                                                key={notification.id}
                                                onClick={() => markNotificationRead(notification.id)}
                                                className={`px-4 py-3 border-b border-dark-100 dark:border-dark-700 last:border-0 cursor-pointer hover:bg-dark-50 dark:hover:bg-dark-700/50 transition-colors ${!notification.read ? 'bg-primary-50/50 dark:bg-primary-500/5' : ''
                                                    }`}
                                            >
                                                <div className="flex items-start gap-3">
                                                    <Badge
                                                        variant={
                                                            notification.type === 'error'
                                                                ? 'danger'
                                                                : notification.type === 'warning'
                                                                    ? 'warning'
                                                                    : notification.type === 'success'
                                                                        ? 'success'
                                                                        : 'info'
                                                        }
                                                        size="sm"
                                                    >
                                                        {notification.type}
                                                    </Badge>
                                                    <div className="flex-1 min-w-0">
                                                        <p className="text-sm font-medium text-dark-900 dark:text-dark-50 truncate">
                                                            {notification.title}
                                                        </p>
                                                        <p className="text-xs text-dark-500 mt-0.5 line-clamp-2">
                                                            {notification.message}
                                                        </p>
                                                        <p className="text-xs text-dark-400 mt-1">
                                                            {formatRelativeTime(notification.timestamp)}
                                                        </p>
                                                    </div>
                                                </div>
                                            </div>
                                        ))
                                    )}
                                </div>
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>

                {/* Help */}
                <button className="p-2 rounded-lg text-dark-500 hover:bg-dark-100 dark:hover:bg-dark-800 transition-colors">
                    <HelpCircle className="w-5 h-5" />
                </button>
            </div>
        </header>
    );
}
