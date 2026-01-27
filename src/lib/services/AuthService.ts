import { invoke } from '@tauri-apps/api/core';

export interface SessionToken {
  token: string;
}

export interface SessionData {
  session_id: string;
  user_id: string;
  username: string;
  roles: string[];
  created_at: string;
  expires_at: string;
}

export class AuthService {
  static async login(username: string, password: string): Promise<SessionToken> {
    console.log("AuthService: Calling login for", username);
    try {
      const result = await invoke<SessionToken>('login', { username, password, userAgent: navigator.userAgent });
      console.log("AuthService: login success", result);
      return result;
    } catch (e) {
      console.error("AuthService: login failed", e);
      throw e;
    }
  }

  static async register(username: string, password: string): Promise<string> {
    console.log("AuthService: Calling register for", username);
    try {
      const result = await invoke<string>('register', { username, password });
      console.log("AuthService: register success", result);
      return result;
    } catch (e) {
      console.error("AuthService: register failed", e);
      throw e;
    }
  }

  static async logout(token: string): Promise<void> {
    await invoke('logout', { token });
  }

  static async validateSession(token: string): Promise<SessionData> {
    return await invoke('validate_session', { token });
  }

  static async renewSession(token: string): Promise<void> {
    await invoke('renew_session', { token });
  }
}
