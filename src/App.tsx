// src/App.tsx
import React from 'react';
import { Layout } from './components/layout/Layout';
import { Dashboard } from './components/views/Dashboard';
import { Scanner } from './components/views/Scanner';
import { Duplicates } from './components/views/Duplicates';
import { Startup } from './components/views/Startup';
import { SettingsView } from './components/views/Settings';
import { useAppStore } from './stores/appStore';
import { AnimatePresence, motion } from 'framer-motion';

function App() {
    const { currentView } = useAppStore();

    const renderView = () => {
        switch (currentView) {
            case 'dashboard':
                return <Dashboard />;
            case 'scanner':
                return <Scanner />;
            case 'duplicates':
                return <Duplicates />;
            case 'startup':
                return <Startup />;
            case 'settings':
                return <SettingsView />;
            case 'large-files':
                return <div className="p-10 text-center text-dark-500">Large Files Module Coming Soon</div>;
            case 'old-files':
                return <div className="p-10 text-center text-dark-500">Old Files Module Coming Soon</div>;
            case 'privacy':
                return <div className="p-10 text-center text-dark-500">Privacy Module Coming Soon</div>;
            case 'history':
                return <div className="p-10 text-center text-dark-500">History Module Coming Soon</div>;
            default:
                return <Dashboard />;
        }
    };

    return (
        <Layout>
            <AnimatePresence mode="wait">
                <motion.div
                    key={currentView}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -10 }}
                    transition={{ duration: 0.2 }}
                    className="h-full"
                >
                    {renderView()}
                </motion.div>
            </AnimatePresence>
        </Layout>
    );
}

export default App;
