import { createSignal } from "solid-js";
import type { User } from "~/types";
import { getMe, checkAuth } from "~/lib/api";
import { handleCallback, login, logout, userManager } from "~/lib/auth";

const [user, setUser] = createSignal<User | null>(null);
const [isAuthenticated, setIsAuthenticated] = createSignal(false);
const [registrationEnabled, setRegistrationEnabled] = createSignal(true);
const [recordingsEnabled, setRecordingsEnabled] = createSignal(true);

export const authStore = {
  get user() { return user(); },
  get isAuthenticated() { return isAuthenticated(); },
  get registrationEnabled() { return registrationEnabled(); },
  get recordingsEnabled() { return recordingsEnabled(); },
};

export async function initAuth() {
  // Handle OIDC callback if present
  await handleCallback();

  const oidcUser = await userManager.getUser();
  if (oidcUser && !oidcUser.expired) {
    setIsAuthenticated(true);
    try {
      const me = await getMe();
      setUser(me);
    } catch {
      // User info not available
    }
  }

  try {
    const config = await checkAuth();
    setRegistrationEnabled(config.registration_enabled);
    setRecordingsEnabled(config.recordings_enabled);
  } catch {
    // Config check not available
  }
}

export { login, logout };
