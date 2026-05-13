import { readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const SEMVER_TAG = /^v(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$/

function versionFromTag(tag) {
  const match = SEMVER_TAG.exec(tag)
  if (!match) {
    throw new Error(`Expected tag to be a v-prefixed semantic version, got: ${tag || '<empty>'}`)
  }
  return match[1]
}

async function updateJsonVersion(file, version, updateRootPackage = false) {
  const data = JSON.parse(await readFile(file, 'utf8'))
  data.version = version
  if (updateRootPackage && data.packages?.['']) {
    data.packages[''].version = version
  }
  await writeFile(file, `${JSON.stringify(data, null, 2)}\n`)
}

async function updateCargoTomlVersion(file, version) {
  const content = await readFile(file, 'utf8')
  const packageVersion = /(^\[package\][\s\S]*?^version\s*=\s*)"[^"]+"/m
  if (!packageVersion.test(content)) {
    throw new Error(`Could not find [package] version in ${file}`)
  }
  const updated = content.replace(packageVersion, `$1"${version}"`)
  await writeFile(file, updated)
}

async function updateCargoLockVersion(file, packageName, version) {
  const content = await readFile(file, 'utf8')
  const newline = content.includes('\r\n') ? '\\r?\\n' : '\\n'
  const packageBlock = new RegExp(
    `(\\[\\[package\\]\\]${newline}name = "${packageName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}"${newline}version = )"[^"]+"`,
  )
  if (!packageBlock.test(content)) {
    throw new Error(`Could not find ${packageName} in ${file}`)
  }
  const updated = content.replace(packageBlock, `$1"${version}"`)
  await writeFile(file, updated)
}

export async function syncReleaseVersion({ root = process.cwd(), tag = process.env.GITHUB_REF_NAME } = {}) {
  const version = versionFromTag(tag)

  await updateJsonVersion(path.join(root, 'package.json'), version)
  await updateJsonVersion(path.join(root, 'package-lock.json'), version, true)
  await updateJsonVersion(path.join(root, 'src-tauri', 'tauri.conf.json'), version)
  await updateCargoTomlVersion(path.join(root, 'src-tauri', 'Cargo.toml'), version)
  await updateCargoLockVersion(
    path.join(root, 'src-tauri', 'Cargo.lock'),
    'gigacoder-config-assistant',
    version,
  )

  return version
}

const isMain = process.argv[1] === fileURLToPath(import.meta.url)
if (isMain) {
  syncReleaseVersion({ tag: process.argv[2] || process.env.GITHUB_REF_NAME })
    .then((version) => {
      console.log(`Synced release metadata to ${version}`)
    })
    .catch((error) => {
      console.error(error.message)
      process.exitCode = 1
    })
}
