// src/components/layout/Layout.tsx
import React from 'react';
import { Sidebar } from './Sidebar';
import { Header } from './Header';

interface LayoutProps {
    children: React.ReactNode;
}

export function Layout({ children }: LayoutProps) {
    return (
        <div className="h-screen flex bg-dark-50 dark:bg-dark-950">
            <Sidebar />
            <div className="flex-1 flex flex-col overflow-hidden">
                <Header />
                <main className="flex-1 overflow-y-auto p-6">
                    {children}
                </main>
            </div>
        </div>
    );
}
