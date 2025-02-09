import { invoke } from '@tauri-apps/api/core';

interface AppStateType {
    authenticated: boolean;
}

class AppStateManager {
    private state = $state<AppStateType>({
        authenticated: false
    });

    async initialize(): Promise<void> {
        await this.refreshAuthState();
    }

    async refreshAuthState(): Promise<void> {
        try {
            const isAuthenticated = await invoke<boolean>('is_authenticated');
            this.state.authenticated = isAuthenticated;
        } catch (error) {
            console.error('Failed to refresh authentication state:', error);
            this.state.authenticated = false;
        }
    }

    isAuthenticated(): boolean {
        return this.state.authenticated;
    }

    getState(): AppStateType {
        return this.state;
    }
}

// Create and initialize singleton instance
const appState = new AppStateManager();
// Export the instance and initialization promise
export const initPromise = appState.initialize().catch(error => {
    console.error('Failed to initialize app state:', error);
});

export default appState;