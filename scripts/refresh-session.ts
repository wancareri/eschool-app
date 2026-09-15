#!/usr/bin/env bun
/**
 * e-school session manager — login + auto-refresh.
 *
 * Usage:
 *   bun run scripts/refresh-session.ts                     # one-shot refresh
 *   bun run scripts/refresh-session.ts --loop              # refresh every 30 min
 *   bun run scripts/refresh-session.ts --login Kuskov11 28012010Yh!  # fresh login
 *   bun run scripts/refresh-session.ts --status           # check token expiry
 */

const OAUTH_URL = "https://oauth.rios.unibel.by";
const BASE_URL = "https://diary.e-schools.by";
const CLIENT_ID = "oauth_diary_echools";
const TOKEN_FILE = `${process.env.HOME}/.eschool-tokens.json`;

// ── Token storage ──────────────────────────────────────────────────────

interface Tokens {
  access_token: string;
  refresh_token: string;
  saved_at: number;
}

async function loadTokens(): Promise<Tokens | null> {
  try {
    const f = Bun.file(TOKEN_FILE);
    if (await f.exists()) return await f.json();
  } catch {}
  return null;
}

async function saveTokens(t: Tokens): Promise<void> {
  await Bun.write(TOKEN_FILE, JSON.stringify(t, null, 2));
  console.log(`Saved to ${TOKEN_FILE}`);
}

function decodeJwtExp(token: string): Date | null {
  try {
    const p = JSON.parse(Buffer.from(token.split(".")[1], "base64url").toString());
    return new Date(p.exp * 1000);
  } catch { return null; }
}

// ── curl helper ────────────────────────────────────────────────────────

function curl(args: string[]): { code: number; stdout: string; stderr: string } {
  const r = Bun.spawnSync(["curl", ...args], { stdout: "pipe", stderr: "pipe" });
  return { code: r.exitCode, stdout: r.stdout.toString(), stderr: r.stderr.toString() };
}

function curlFollow(url: string, opts: string[] = []): { code: number; headers: string; body: string } {
  const r = curl(["-sS", "-k", "-L", "--max-time", "15", ...opts, url]);
  return { code: r.code, headers: "", body: r.stdout };
}

// ── OAuth login (7-step flow using curl with cookie jar) ───────────────

async function login(username: string, password: string): Promise<Tokens> {
  const cookieFile = `/tmp/eschool-cookies-${Date.now()}.txt`;
  const UA = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15";

  const C = (url: string, extra: string[] = []) =>
    curl(["-sS", "-k", "-c", cookieFile, "-b", cookieFile, "--max-time", "15",
      "-H", `User-Agent: ${UA}`, ...extra, url]);

  const CRedirect = (url: string, extra: string[] = []) =>
    curl(["-sS", "-k", "-c", cookieFile, "-b", cookieFile, "--max-time", "15",
      "-H", `User-Agent: ${UA}`, ...extra, url]);

  // Step 1: GET diary login/student → 302 to OAuth
  console.log("[1/7] GET diary login/student...");
  const r1 = C(`${BASE_URL}/api/v1/admin/auth/login/student`, ["-D", "-"]);
  const loc1 = r1.stdout.match(/location:\s*(.+)/i)?.[1]?.trim();
  if (!loc1) throw new Error(`Step 1: no Location. Output: ${r1.stdout.slice(0, 200)}`);

  // Step 2: GET OAuth login page → extract CSRF + ReturnUrl
  const loginPageUrl = loc1.startsWith("http") ? loc1 : `${OAUTH_URL}${loc1}`;
  console.log("[2/7] GET OAuth login page...");
  const r2 = C(loginPageUrl);
  const html = r2.stdout;
  const csrf = html.match(/name="__RequestVerificationToken"[^>]*value="([^"]+)"/)?.[1];
  const returnUrl = html.match(/name="Input\.ReturnUrl"\s+value="([^"]+)"/)?.[1]?.replace(/&amp;/g, "&");
  if (!csrf) throw new Error("CSRF not found");
  if (!returnUrl) throw new Error("ReturnUrl not found");

  // Step 3: POST credentials → 302 to callback
  const origin = loginPageUrl.match(/^https?:\/\/[^/]+/)?.[0];
  if (!origin) throw new Error("Cannot extract origin");
  console.log("[3/7] POST credentials...");
  const r3 = curl(["-sS", "-k", "-c", cookieFile, "-b", cookieFile, "--max-time", "15",
    "-H", `User-Agent: ${UA}`,
    "-H", "Content-Type: application/x-www-form-urlencoded",
    "-D", "-",
    "-X", "POST",
    "-d", `Input.ReturnUrl=${encodeURIComponent(returnUrl)}`,
    "-d", `Input.Username=${username}`,
    "-d", `Input.Password=${encodeURIComponent(password)}`,
    "-d", "Input.Button=login",
    "-d", "Input.RememberLogin=false",
    "-d", `__RequestVerificationToken=${csrf}`,
    `${origin}/Account/Login`,
  ]);
  const loc3 = r3.stdout.match(/location:\s*(.+)/i)?.[1]?.trim()?.replace(/&amp;/g, "&");
  if (!loc3) throw new Error(`Step 3: no Location. Output: ${r3.stdout.slice(0, 300)}`);

  // Step 4: GET callback → 302 to diary callback
  console.log("[4/7] GET callback...");
  const cbUrl = loc3.startsWith("http") ? loc3 : `${origin}${loc3}`;
  const r4 = C(cbUrl, ["-D", "-", "-H", `Referer: ${origin}/`]);
  const loc4 = r4.stdout.match(/location:\s*(.+)/i)?.[1]?.trim()?.replace(/&amp;/g, "&");
  if (!loc4) throw new Error(`Step 4: no Location. Output: ${r4.stdout.slice(0, 300)}`);

  // Step 5: GET diary callback → 302 to preauthorized
  console.log("[5/7] GET diary callback...");
  const diagCbUrl = loc4.startsWith("http") ? loc4 : `${BASE_URL}${loc4}`;
  const r5 = C(diagCbUrl, ["-D", "-"]);
  const loc5 = r5.stdout.match(/location:\s*(.+)/i)?.[1]?.trim()?.replace(/&amp;/g, "&");
  if (!loc5) throw new Error(`Step 5: no Location. Output: ${r5.stdout.slice(0, 300)}`);
  const uuid = loc5.match(/preauthorized\?data=([^&]+)/)?.[1];
  if (!uuid) throw new Error(`UUID not found in: ${loc5}`);

  // Step 6: GET data_for_login/{UUID}
  console.log("[6/7] GET data_for_login...");
  const r6 = C(`${BASE_URL}/api/v1/admin/auth/data_for_login/${uuid}`);
  const loginData = JSON.parse(r6.stdout);
  const profileId = loginData.profile_id as string;
  const schoolId = (loginData.schools as any[])[0].id as string;
  const kind = ((loginData.kinds as string[])[0] || "student").toLowerCase();

  // Step 7: GET auth/login?token={base64}
  console.log("[7/7] GET auth/login...");
  const tokenPayload = `${profileId}:${schoolId}:${kind}`;
  const tokenB64 = Buffer.from(tokenPayload).toString("base64");
  const r7 = C(`${BASE_URL}/api/v1/auth/login?token=${tokenB64}`);
  const authData = JSON.parse(r7.stdout);

  // Cleanup cookie file
  try { Bun.spawnSync(["rm", "-f", cookieFile]); } catch {}

  return {
    access_token: authData.auth_token || authData.access_token,
    refresh_token: authData.refresh_token,
    saved_at: Date.now(),
  };
}

// ── Token refresh ──────────────────────────────────────────────────────

async function refreshTokens(refreshToken: string): Promise<Tokens> {
  const r = curl([
    "-sS", "-k", "--max-time", "15",
    "-X", "POST",
    "-H", "Content-Type: application/x-www-form-urlencoded",
    "-d", `client_id=${CLIENT_ID}`,
    "-d", "grant_type=refresh_token",
    `-d`, `refresh_token=${refreshToken}`,
    `${OAUTH_URL}/connect/token`,
  ]);

  if (r.code !== 0 || !r.stdout.includes("auth_token")) {
    throw new Error(`Refresh failed: ${r.stdout}`);
  }

  const data = JSON.parse(r.stdout);
  return {
    access_token: data.auth_token || data.access_token,
    refresh_token: data.refresh_token || refreshToken,
    saved_at: Date.now(),
  };
}

// ── Commands ───────────────────────────────────────────────────────────

async function cmdStatus() {
  const t = await loadTokens();
  if (!t) { console.log("No tokens. Run --login first."); return; }
  const exp = decodeJwtExp(t.access_token);
  if (exp) {
    const left = exp.getTime() - Date.now();
    const mins = Math.floor(left / 60000);
    console.log(left > 0
      ? `Token valid. Expires: ${exp.toLocaleString()} (${mins}m left)`
      : `Token EXPIRED ${Math.abs(mins)}m ago.`);
  }
  console.log(`Saved: ${new Date(t.saved_at).toLocaleString()}`);
}

async function cmdRefresh() {
  const t = await loadTokens();
  if (!t) { console.error("No tokens. Run --login first."); process.exit(1); }
  const exp = decodeJwtExp(t.access_token);
  if (exp && exp.getTime() - Date.now() > 300_000) {
    console.log(`Token valid for ${Math.floor((exp.getTime() - Date.now()) / 60000)}m. Skipping.`);
    return;
  }
  console.log("Refreshing...");
  const newT = await refreshTokens(t.refresh_token);
  await saveTokens(newT);
  const newExp = decodeJwtExp(newT.access_token);
  console.log(`✓ Done. Expires: ${newExp?.toLocaleString() ?? "unknown"}`);
}

async function cmdLogin(user: string, pass: string) {
  console.log("Logging in...");
  const t = await login(user, pass);
  await saveTokens(t);
  const exp = decodeJwtExp(t.access_token);
  console.log(`✓ Logged in. Expires: ${exp?.toLocaleString() ?? "unknown"}`);
}

// ── Entry ──────────────────────────────────────────────────────────────

const args = process.argv.slice(2);
if (args.includes("--status")) {
  await cmdStatus();
} else if (args.includes("--login")) {
  const u = args[args.indexOf("--login") + 1];
  const p = args[args.indexOf("--login") + 2];
  if (!u || !p) { console.error("Usage: --login <user> <pass>"); process.exit(1); }
  await cmdLogin(u, p);
} else if (args.includes("--loop")) {
  console.log("Auto-refresh loop (30 min). Ctrl+C to stop.");
  await cmdRefresh();
  setInterval(cmdRefresh, 30 * 60 * 1000);
} else {
  await cmdRefresh();
}
