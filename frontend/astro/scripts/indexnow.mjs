#!/usr/bin/env node
// Submit every URL in the live sitemap to IndexNow (Bing, Yandex, Seznam,
// Naver, … — one ping fans out to all participating engines).
//
// Reads the *deployed* sitemap rather than dist/, so run it after
// `flyctl deploy` has finished. The key is public by design: engines
// verify ownership by fetching https://<host>/<key>.txt, which lives in
// public/.
//
// Usage: node scripts/indexnow.mjs [--dry-run]

const HOST = "pencilready.com";
const KEY = "f5bfa0ef0364972fa8d92e831efdadf9";
const dryRun = process.argv.includes("--dry-run");

async function locs(url) {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
    const xml = await res.text();
    return [...xml.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => m[1]);
}

// sitemap-index.xml lists sitemap-N.xml files; those list the pages.
const urls = [];
for (const sitemap of await locs(`https://${HOST}/sitemap-index.xml`)) {
    urls.push(...(await locs(sitemap)));
}

console.log(`${urls.length} URLs`);
if (dryRun) {
    for (const u of urls) console.log(`  ${u}`);
    process.exit(0);
}

const keyRes = await fetch(`https://${HOST}/${KEY}.txt`);
if (!keyRes.ok || (await keyRes.text()).trim() !== KEY) {
    console.error(`key file https://${HOST}/${KEY}.txt not served — deploy first`);
    process.exit(1);
}

const res = await fetch("https://api.indexnow.org/indexnow", {
    method: "POST",
    headers: { "Content-Type": "application/json; charset=utf-8" },
    body: JSON.stringify({
        host: HOST,
        key: KEY,
        keyLocation: `https://${HOST}/${KEY}.txt`,
        urlList: urls,
    }),
});
// 200 = accepted, 202 = accepted but key not yet verified.
console.log(`IndexNow: HTTP ${res.status} ${await res.text()}`);
if (res.status !== 200 && res.status !== 202) process.exit(1);
