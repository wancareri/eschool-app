#!/usr/bin/env npx tsx

interface EschoolConfig {
  username: string;
  password: string;
  baseUrl?: string;
  oauthUrl?: string;
}

interface AuthTokens {
  auth_token: string;
  refresh_token: string;
}

interface UserProfile {
  user_id: string;
  school_id: string;
  school_name: string;
  profile_id: string;
  person_kind: string;
  full_name: string;
  roles: string[];
}

const UA =
  "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

function decodeEntities(s: string): string {
  return s
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"');
}

class EschoolAuth {
  private baseUrl: string;
  private oauthUrl: string;
  private cookieJar: Map<string, string> = new Map();

  constructor(config: EschoolConfig) {
    this.baseUrl = config.baseUrl ?? "https://diary.e-schools.by";
    this.oauthUrl = config.oauthUrl ?? "https://oauth.rios.unibel.by";
  }

  private setCookies(headers: Headers, domain: string): void {
    for (const h of headers.getSetCookie()) {
      const [cookiePart] = h.split(";");
      const eqIdx = cookiePart.indexOf("=");
      if (eqIdx === -1) continue;
      const name = cookiePart.slice(0, eqIdx).trim();
      const value = cookiePart.slice(eqIdx + 1).trim();
      this.cookieJar.set(`${domain}:${name}`, value);
    }
  }

  private getCookies(domain: string): string {
    const cookies: string[] = [];
    for (const [key, value] of this.cookieJar) {
      if (key.startsWith(`${domain}:`)) {
        cookies.push(`${key.slice(domain.length + 1)}=${value}`);
      }
    }
    return cookies.join("; ");
  }

  private async req(
    url: string,
    init?: RequestInit
  ): Promise<Response> {
    const parsed = new URL(url);
    const domain = parsed.hostname;
    const headers = new Headers(init?.headers);
    headers.set("User-Agent", UA);
    const existingCookies = this.getCookies(domain);
    if (existingCookies) {
      headers.set("Cookie", existingCookies);
    }
    const resp = await fetch(url, { ...init, headers, redirect: "manual" });
    this.setCookies(resp.headers, domain);
    return resp;
  }

  private extractCsrfToken(html: string): string {
    const match = html.match(
      /__RequestVerificationToken.*?value="([^"]+)"/
    );
    if (!match) throw new Error("CSRF token not found in HTML");
    return match[1];
  }

  private extractReturnUrl(html: string): string {
    const match = html.match(
      /name="Input\.ReturnUrl"\s+value="([^"]+)"/
    );
    if (!match) throw new Error("ReturnUrl not found in HTML");
    return decodeEntities(match[1]);
  }

  async login(username: string, password: string): Promise<AuthTokens> {
    console.log("[1] Starting auth flow...");

    const resp1 = await this.req(
      `${this.baseUrl}/api/v1/admin/auth/login/student`
    );
    const oauthLoginUrl = resp1.headers.get("location");
    if (!oauthLoginUrl) throw new Error("No redirect from login/student");

    console.log("[2] Fetching OAuth login page...");
    const resp2 = await this.req(oauthLoginUrl);
    const loginHtml = await resp2.text();
    const csrfToken = this.extractCsrfToken(loginHtml);
    const returnUrl = this.extractReturnUrl(loginHtml);

    console.log("[3] Submitting credentials...");
    const body = new URLSearchParams({
      "Input.ReturnUrl": returnUrl,
      "Input.Username": username,
      "Input.Password": password,
      "Input.Button": "login",
      "Input.RememberLogin": "false",
      __RequestVerificationToken: csrfToken,
    });

    const oauthOrigin = new URL(oauthLoginUrl).origin;
    const resp3 = await this.req(`${oauthOrigin}/Account/Login`, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: body.toString(),
    });

    const callbackUrlRaw = resp3.headers.get("location");
    if (!callbackUrlRaw) throw new Error("No redirect after login");
    const callbackUrl = decodeEntities(callbackUrlRaw);

    console.log("[4] Following OAuth callback chain...");
    const resp4 = await this.req(
      callbackUrl.startsWith("http")
        ? callbackUrl
        : `${oauthOrigin}${callbackUrl}`
    );

    const diaryCallbackUrlRaw = resp4.headers.get("location");
    if (!diaryCallbackUrlRaw)
      throw new Error("No redirect from authorize/callback");
    const diaryCallbackUrl = decodeEntities(diaryCallbackUrlRaw);

    console.log("[5] Following diary callback...");
    const resp5 = await this.req(
      diaryCallbackUrl.startsWith("http")
        ? diaryCallbackUrl
        : `${this.baseUrl}${diaryCallbackUrl}`
    );

    const preauthRedirectRaw = resp5.headers.get("location");
    if (!preauthRedirectRaw) throw new Error("No redirect to preauthorized");
    const preauthRedirect = decodeEntities(preauthRedirectRaw);

    const uuidMatch = preauthRedirect.match(/preauthorized\?data=([^&]+)/);
    if (!uuidMatch) throw new Error("UUID not found in preauth redirect");
    const preauthUuid = uuidMatch[1];

    console.log(`[6] Preauth UUID: ${preauthUuid}`);

    console.log("[7] Getting login data...");
    const resp7 = await this.req(
      `${this.baseUrl}/api/v1/admin/auth/data_for_login/${preauthUuid}`
    );
    const loginData = (await resp7.json()) as {
      profile_id: string;
      kinds: string[];
      schools: { id: string; name: string }[];
    };

    const profileId = loginData.profile_id;
    const schoolId = loginData.schools[0]?.id;
    const personKind = loginData.kinds[0]?.toLowerCase() ?? "student";

    console.log(`    Profile: ${profileId}`);
    console.log(`    School:  ${loginData.schools[0]?.name}`);

    console.log("[8] Exchanging for JWT token...");
    const tokenPayload = `${profileId}:${schoolId}:${personKind}`;
    const tokenBase64 = Buffer.from(tokenPayload).toString("base64");
    const resp8 = await this.req(
      `${this.baseUrl}/api/v1/auth/login?token=${tokenBase64}`
    );
    const tokens = (await resp8.json()) as AuthTokens;

    console.log(`    auth_token: ${tokens.auth_token.slice(0, 50)}...`);

    return tokens;
  }

  async getProfile(token: string): Promise<UserProfile> {
    console.log("\n[9] Fetching user profile...");
    const resp = await this.req(`${this.baseUrl}/api/v1/admin/auth/me`, {
      headers: { Authorization: token },
    });
    const profile = (await resp.json()) as UserProfile;

    console.log("    User ID:   ", profile.user_id);
    console.log("    School:    ", profile.school_name);
    console.log("    Full name: ", profile.full_name);
    console.log("    Kind:      ", profile.person_kind);
    console.log("    Roles:     ", profile.roles.join(", "));
    console.log("    Profile ID:", profile.profile_id);

    return profile;
  }
}

async function main() {
  const args = process.argv.slice(2);
  const username = args[0];
  const password = args[1];

  if (!username || !password) {
    console.error(
      "Usage: npx tsx scripts/eschool-auth.ts <username> <password>"
    );
    process.exit(1);
  }

  const auth = new EschoolAuth({ username, password });

  try {
    const tokens = await auth.login(username, password);
    console.log("\n========== AUTH SUCCESS ==========");
    console.log("auth_token:    ", tokens.auth_token);
    console.log("refresh_token: ", tokens.refresh_token);
    console.log("==================================\n");

    const profile = await auth.getProfile(tokens.auth_token);
    console.log("\n========== PROFILE ==========");
    console.log(JSON.stringify(profile, null, 2));
    console.log("==============================");
  } catch (err) {
    console.error("Auth failed:", err);
    process.exit(1);
  }
}

main();
