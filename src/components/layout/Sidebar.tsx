// src/components/layout/Sidebar.tsx
import React from 'react';
import { clsx } from 'clsx';
import { motion, AnimatePresence } from 'framer-motion';
import {
    LayoutDashboard,
    Search,
    Copy,
    Rocket,
    HardDrive,
    Clock,
    Settings,
    History,
    Shield,
    ChevronLeft,
    ChevronRight,
    Sparkles,
} from 'lucide-react';
import { useAppStore } from '../../stores/appStore';

interface NavItem {
    id: string;
    label: string;
    icon: React.ReactNode;
    badge?: number;
}

const navItems: NavItem[] = [
    { id: 'dashboard', label: 'Dashboard', icon: <LayoutDashboard className="w-5 h-5" /> },
    { id: 'scanner', label: 'Smart Scan', icon: <Search className="w-5 h-5" /> },
    { id: 'duplicates', label: 'Duplicates', icon: <Copy className="w-5 h-5" /> },
    { id: 'startup', label: 'Startup', icon: <Rocket className="w-5 h-5" /> },
    { id: 'large-files', label: 'Large Files', icon: <HardDrive className="w-5 h-5" /> },
    { id: 'old-files', label: 'Old Files', icon: <Clock className="w-5 h-5" /> },
    { id: 'privacy', label: 'Privacy', icon: <Shield className="w-5 h-5" /> },
    { id: 'history', label: 'History', icon: <History className="w-5 h-5" /> },
    { id: 'settings', label: 'Settings', icon: <Settings className="w-5 h-5" /> },
];

export function Sidebar() {
    const { sidebarCollapsed, setSidebarCollapsed, currentView, setCurrentView } = useAppStore();

    return (
        <motion.aside
            initial={false}
            animate={{ width: sidebarCollapsed ? 72 : 240 }}
            transition={{ duration: 0.2 }}
            className="h-full bg-white dark:bg-dark-900 border-r border-dark-200 dark:border-dark-800 flex flex-col"
        >
            {/* Logo */}
            <div className="h-16 flex items-center px-4 border-b border-dark-200 dark:border-dark-800">
                <div className="flex items-center gap-3">
                    <div className="w-10 h-10 bg-gradient-to-br from-primary-500 to-primary-700 rounded-xl flex items-center justify-center shadow-lg">
                        <Sparkles className="w-6 h-6 text-white" />
                    </div>
                    <AnimatePresence>
                        {!sidebarCollapsed && (
                            <motion.span
                                initial={{ opacity: 0, x: -10 }}
                                animate={{ opacity: 1, x: 0 }}
                                exit={{ opacity: 0, x: -10 }}
                                className="font-bold text-lg text-dark-900 dark:text-dark-50 whitespace-nowrap"
                            >
                                CleanDesk
                            </motion.span>
                        )}
                    </AnimatePresence>
                </div>
            </div>

            {/* Navigation */}
            <nav className="flex-1 py-4 px-3 space-y-1 overflow-y-auto">
                {navItems.map((item) => (
                    <button
                        key={item.id}
                        onClick={() => setCurrentView(item.id)}
                        className={clsx(
                            'w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-200',
                            currentView === item.id
                                ? 'bg-primary-100 dark:bg-primary-500/20 text-primary-700 dark:text-primary-400'
                                : 'text-dark-600 dark:text-dark-400 hover:bg-dark-100 dark:hover:bg-dark-800 hover:text-dark-900 dark:hover:text-dark-100'
                        )}
                    >
                        <span className={clsx(
                            'flex-shrink-0',
                            currentView === item.id && 'text-primary-600 dark:text-primary-400'
                        )}>
                            {item.icon}
                        </span>
                        <AnimatePresence>
                            {!sidebarCollapsed && (
                                <motion.span
                                    initial={{ opacity: 0, x: -10 }}
                                    animate={{ opacity: 1, x: 0 }}
                                    exit={{ opacity: 0, x: -10 }}
                                    className="flex-1 text-left font-medium whitespace-nowrap"
                                >
                                    {item.label}
                                </motion.span>
                            )}
                        </AnimatePresence>
                        {item.badge && !sidebarCollapsed && (
                            <span className="px-2 py-0.5 text-xs font-medium bg-danger-100 dark:bg-danger-500/20 text-danger-600 dark:text-danger-400 rounded-full">
                                {item.badge}
                            </span>
                        )}
                    </button>
                ))}
            </nav>

            {/* Collapse Toggle */}
            <div className="p-3 border-t border-dark-200 dark:border-dark-800">
                <button
                    onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
                    className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-dark-500 hover:bg-dark-100 dark:hover:bg-dark-800 transition-colors"
                >
                    {sidebarCollapsed ? (
                        <ChevronRight className="w-5 h-5" />
                    ) : (
                        <>
                            <ChevronLeft className="w-5 h-5" />
                            <span className="text-sm">Collapse</span>
                        </>
                    )}
                </button>
            </div>
        </motion.aside>
    );
}
