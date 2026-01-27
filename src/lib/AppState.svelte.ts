import { invoke } from '@tauri-apps/api/core';
import { AuthService, type SessionData } from './services/AuthService';

interface AppStateType {
    authenticated: boolean;
    user?: SessionData;
    token?: string;
}

class AppStateManager {
    private state = $state<AppStateType>({
        authenticated: false
    });

    async initialize(): Promise<void> {
        // Try to recover session from localStorage
        const token = localStorage.getItem('session_token');
        if (token) {
            try {
                const session = await AuthService.validateSession(token);
                this.state.authenticated = true;
                this.state.user = session;
                this.state.token = token;
            } catch (e) {
                console.error("Session invalid", e);
                localStorage.removeItem('session_token');
                this.state.authenticated = false;
            }
        }
    }

    async login(username: string, password: string): Promise<void> {
        const tokenObj = await AuthService.login(username, password);
        const token = tokenObj.token;
        const session = await AuthService.validateSession(token);

        localStorage.setItem('session_token', token);
        this.state.authenticated = true;
        this.state.token = token;
        this.state.user = session;
    }

    async logout(): Promise<void> {
        if (this.state.token) {
            try {
                await AuthService.logout(this.state.token);
            } catch (e) {
                console.warn("Logout on backend failed", e);
            }
        }
        localStorage.removeItem('session_token');
        this.state.authenticated = false;
        this.state.user = undefined;
        this.state.token = undefined;
    }

    async register(username: string, password: string): Promise<void> {
        await AuthService.register(username, password);
        // Auto login after register
        await this.login(username, password);
    }

    isAuthenticated(): boolean {
        return this.state.authenticated;
    }

    getUser(): SessionData | undefined {
        return this.state.user;
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