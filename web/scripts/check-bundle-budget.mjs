import { existsSync, readdirSync, readFileSync, statSync } from "node:fs"
import { basename, join } from "node:path"
import { fileURLToPath } from "node:url"
import { gzipSync } from "node:zlib"

const assetsDir = fileURLToPath(new URL("../dist/assets/", import.meta.url))

const budgets = [
  {
    name: "initial-js",
    pattern: /^index-[\w-]+\.js$/,
    required: true,
    maxRawKiB: 520,
    maxGzipKiB: 160,
  },
  {
    name: "initial-css",
    pattern: /^index-[\w-]+\.css$/,
    required: true,
    maxRawKiB: 230,
    maxGzipKiB: 35,
  },
  {
    name: "admin-route-js",
    pattern: /^admin-[\w-]+\.js$/,
    required: false,
    maxRawKiB: 260,
    maxGzipKiB: 65,
  },
  {
    name: "media-route-js",
    pattern: /^media-[\w-]+\.js$/,
    required: false,
    maxRawKiB: 360,
    maxGzipKiB: 90,
  },
]

const totalJsBudget = {
  name: "total-js",
  maxRawKiB: 1_250,
  maxGzipKiB: 330,
}

if (!existsSync(assetsDir)) {
  console.error("Bundle assets were not found. Run `npm --prefix web run build` first.")
  process.exit(1)
}

const assetFiles = readdirSync(assetsDir)
  .map((file) => join(assetsDir, file))
  .filter((file) => statSync(file).isFile())

let failed = false
const rows = []

for (const budget of budgets) {
  const matches = assetFiles
    .filter((file) => budget.pattern.test(basename(file)))
    .map(readAsset)
    .sort((a, b) => b.rawKiB - a.rawKiB)

  if (matches.length === 0) {
    if (budget.required) {
      failed = true
      rows.push({ name: budget.name, file: "missing", rawKiB: 0, gzipKiB: 0, status: "FAIL" })
    } else {
      rows.push({ name: budget.name, file: "not emitted", rawKiB: 0, gzipKiB: 0, status: "OK" })
    }
    continue
  }

  const asset = matches[0]
  const overRaw = asset.rawKiB > budget.maxRawKiB
  const overGzip = asset.gzipKiB > budget.maxGzipKiB
  failed ||= overRaw || overGzip
  rows.push({
    name: budget.name,
    file: asset.file,
    rawKiB: asset.rawKiB,
    gzipKiB: asset.gzipKiB,
    maxRawKiB: budget.maxRawKiB,
    maxGzipKiB: budget.maxGzipKiB,
    status: overRaw || overGzip ? "FAIL" : "OK",
  })
}

const jsAssets = assetFiles.filter((file) => file.endsWith(".js")).map(readAsset)
const totalJs = jsAssets.reduce(
  (acc, asset) => ({
    rawKiB: acc.rawKiB + asset.rawKiB,
    gzipKiB: acc.gzipKiB + asset.gzipKiB,
  }),
  { rawKiB: 0, gzipKiB: 0 },
)
const totalJsOverRaw = totalJs.rawKiB > totalJsBudget.maxRawKiB
const totalJsOverGzip = totalJs.gzipKiB > totalJsBudget.maxGzipKiB
failed ||= totalJsOverRaw || totalJsOverGzip
rows.push({
  name: totalJsBudget.name,
  file: `${jsAssets.length} js assets`,
  rawKiB: roundKiB(totalJs.rawKiB),
  gzipKiB: roundKiB(totalJs.gzipKiB),
  maxRawKiB: totalJsBudget.maxRawKiB,
  maxGzipKiB: totalJsBudget.maxGzipKiB,
  status: totalJsOverRaw || totalJsOverGzip ? "FAIL" : "OK",
})

console.log("Nako web bundle budget")
console.log(
  [
    "budget".padEnd(18),
    "file".padEnd(38),
    "raw KiB".padStart(9),
    "gzip KiB".padStart(10),
    "limit KiB".padStart(18),
    "status".padStart(8),
  ].join("  "),
)

for (const row of rows) {
  const limit = row.maxRawKiB ? `${row.maxRawKiB}/${row.maxGzipKiB}` : "-"
  console.log(
    [
      row.name.padEnd(18),
      row.file.padEnd(38),
      formatKiB(row.rawKiB).padStart(9),
      formatKiB(row.gzipKiB).padStart(10),
      limit.padStart(18),
      row.status.padStart(8),
    ].join("  "),
  )
}

if (failed) {
  console.error("Bundle budget failed.")
  process.exit(1)
}

function readAsset(filePath) {
  const content = readFileSync(filePath)
  return {
    file: basename(filePath),
    rawKiB: roundKiB(content.byteLength / 1024),
    gzipKiB: roundKiB(gzipSync(content).byteLength / 1024),
  }
}

function roundKiB(value) {
  return Math.round(value * 100) / 100
}

function formatKiB(value) {
  return value.toFixed(2)
}
