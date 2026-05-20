import { UserManager, WebStorageStateStore } from "oidc-client-ts";

const oidcConfig = {
  authority: import.meta.env.VITE_OIDC_AUTHORITY || "http://localhost:8180/realms/scraperr",
  client_id: import.meta.env.VITE_OIDC_CLIENT_ID || "scraperr-frontend",
  redirect_uri: `${window.location.origin}/`,
  post_logout_redirect_uri: `${window.location.origin}/`,
  response_type: "code",
  scope: "openid profile email",
  userStore: new WebStorageStateStore({ store: window.localStorage }),
};

export const userManager = new UserManager(oidcConfig);

export async function getAccessToken(): Promise<string | null> {
  const user = await userManager.getUser();
  if (!user || user.expired) {
    return null;
  }
  return user.access_token;
}

export async function login() {
  await userManager.signinRedirect();
}

export async function logout() {
  await userManager.signoutRedirect();
}

export async function handleCallback(): Promise<void> {
  if (window.location.search.includes("code=")) {
    await userManager.signinRedirectCallback();
    window.history.replaceState({}, document.title, window.location.pathname);
  }
}
